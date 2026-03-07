use crate::types::*;
use crate::linalg::*;
use crate::element::*;
use super::dof::DofNumbering;
use super::assembly;

/// Solve a 2D frame using co-rotational large displacement analysis.
///
/// This wraps the existing linear element formulations, rebuilding
/// transformations from deformed geometry at each Newton-Raphson iteration.
/// Uses incremental-iterative loading with Newton-Raphson inner loops.
pub fn solve_corotational_2d(
    input: &SolverInput,
    max_iter: usize,
    tolerance: f64,
    n_increments: usize,
) -> Result<CorotationalResult, String> {
    let dof_num = DofNumbering::build_2d(input);

    if dof_num.n_free == 0 {
        return Err("No free DOFs — all nodes are fully restrained".into());
    }

    let n = dof_num.n_total;
    let nf = dof_num.n_free;

    // Get the full external load vector from linear assembly (reference load)
    let asm = assembly::assemble_2d(input, &dof_num);
    let f_total = asm.f.clone();

    // Global displacement vector (starts at zero)
    let mut u_full = vec![0.0; n];

    let mut total_iterations = 0;
    let mut converged = true;

    // Incremental-iterative procedure
    for increment in 1..=n_increments {
        let load_factor = increment as f64 / n_increments as f64;

        // Scaled external forces for this increment
        let f_ext: Vec<f64> = f_total.iter().map(|&f| load_factor * f).collect();

        // Newton-Raphson inner loop
        let mut nr_converged = false;
        for _iter in 0..max_iter {
            total_iterations += 1;

            // Compute internal forces and tangent stiffness from co-rotational formulation
            let mut f_int = vec![0.0; n];
            let mut k_t = vec![0.0; n * n];

            assemble_corotational(input, &dof_num, &u_full, &mut f_int, &mut k_t);

            // Add spring stiffness contributions to K_T and f_int
            add_spring_contributions(input, &dof_num, &u_full, &mut f_int, &mut k_t);

            // Residual R = F_ext - f_int
            let mut residual = vec![0.0; n];
            for i in 0..n {
                residual[i] = f_ext[i] - f_int[i];
            }

            // Check convergence on free DOFs only
            let mut r_free_norm_sq = 0.0;
            let mut f_ext_free_norm_sq = 0.0;
            for i in 0..nf {
                r_free_norm_sq += residual[i] * residual[i];
                f_ext_free_norm_sq += f_ext[i] * f_ext[i];
            }
            let r_free_norm = r_free_norm_sq.sqrt();
            let f_ext_free_norm = f_ext_free_norm_sq.sqrt();

            let rel_error = if f_ext_free_norm > 1e-30 {
                r_free_norm / f_ext_free_norm
            } else {
                r_free_norm
            };

            if rel_error < tolerance {
                nr_converged = true;
                break;
            }

            // Solve K_T * delta_u = R for free DOFs
            let free_idx: Vec<usize> = (0..nf).collect();
            let k_ff = extract_submatrix(&k_t, n, &free_idx, &free_idx);
            let r_f: Vec<f64> = residual[..nf].to_vec();

            let delta_u_f = solve_free_dofs(&k_ff, &r_f, nf)?;

            // Update displacements (free DOFs only)
            for i in 0..nf {
                u_full[i] += delta_u_f[i];
            }
        }

        if !nr_converged {
            converged = false;
            break;
        }
    }

    // Find max displacement magnitude
    let max_displacement = compute_max_displacement(&dof_num, &u_full);

    // Build final results using existing infrastructure
    let results = build_final_results(input, &dof_num, &u_full)?;

    Ok(CorotationalResult {
        results,
        iterations: total_iterations,
        converged,
        load_increments: n_increments,
        max_displacement,
    })
}

/// Assemble co-rotational internal forces and tangent stiffness for all elements.
fn assemble_corotational(
    input: &SolverInput,
    dof_num: &DofNumbering,
    u_full: &[f64],
    f_int: &mut [f64],
    k_t: &mut [f64],
) {
    let n = dof_num.n_total;

    for elem in input.elements.values() {
        let node_i = input.nodes.values().find(|nd| nd.id == elem.node_i).unwrap();
        let node_j = input.nodes.values().find(|nd| nd.id == elem.node_j).unwrap();
        let mat = input.materials.values().find(|m| m.id == elem.material_id).unwrap();
        let sec = input.sections.values().find(|s| s.id == elem.section_id).unwrap();

        let e = mat.e * 1000.0; // MPa -> kN/m^2

        if elem.elem_type == "truss" {
            assemble_truss_corotational(
                dof_num, elem, node_i, node_j, e, sec.a,
                u_full, f_int, k_t, n,
            );
        } else {
            assemble_frame_corotational(
                dof_num, elem, node_i, node_j, e, sec.a, sec.iz,
                u_full, f_int, k_t, n,
            );
        }
    }
}

/// Co-rotational treatment for a 2D truss element.
fn assemble_truss_corotational(
    dof_num: &DofNumbering,
    elem: &SolverElement,
    node_i: &SolverNode,
    node_j: &SolverNode,
    e: f64,
    a: f64,
    u_full: &[f64],
    f_int: &mut [f64],
    k_t: &mut [f64],
    n: usize,
) {
    let dx0 = node_j.x - node_i.x;
    let dy0 = node_j.y - node_i.y;
    let l0 = (dx0 * dx0 + dy0 * dy0).sqrt();

    let u_xi = dof_num.global_dof(elem.node_i, 0).map(|d| u_full[d]).unwrap_or(0.0);
    let u_yi = dof_num.global_dof(elem.node_i, 1).map(|d| u_full[d]).unwrap_or(0.0);
    let u_xj = dof_num.global_dof(elem.node_j, 0).map(|d| u_full[d]).unwrap_or(0.0);
    let u_yj = dof_num.global_dof(elem.node_j, 1).map(|d| u_full[d]).unwrap_or(0.0);

    let dx_n = (node_j.x + u_xj) - (node_i.x + u_xi);
    let dy_n = (node_j.y + u_yj) - (node_i.y + u_yi);
    let l_n = (dx_n * dx_n + dy_n * dy_n).sqrt();

    if l_n < 1e-15 {
        return;
    }

    let c = dx_n / l_n;
    let s = dy_n / l_n;

    let d_axial = l_n - l0;
    let axial_force = e * a / l0 * d_axial;

    // Internal force: f = N * r, where r = [-c, -s, c, s]
    let r = [-c, -s, c, s];

    let truss_dofs = [
        dof_num.global_dof(elem.node_i, 0).unwrap(),
        dof_num.global_dof(elem.node_i, 1).unwrap(),
        dof_num.global_dof(elem.node_j, 0).unwrap(),
        dof_num.global_dof(elem.node_j, 1).unwrap(),
    ];

    for i in 0..4 {
        f_int[truss_dofs[i]] += axial_force * r[i];
    }

    // Tangent: K = (EA/L0) * r*r^T + (N/L_n) * (z_mat - r*r^T)
    let ea_l0 = e * a / l0;
    let n_over_l = axial_force / l_n;

    for i in 0..4 {
        for j in 0..4 {
            let k_mat = ea_l0 * r[i] * r[j];
            // z_ij for the block-structured identity
            let sign_i = if i < 2 { -1.0 } else { 1.0 };
            let sign_j = if j < 2 { -1.0 } else { 1.0 };
            let delta = if i % 2 == j % 2 { 1.0 } else { 0.0 };
            let z_ij = sign_i * sign_j * delta;
            let k_geo = n_over_l * (z_ij - r[i] * r[j]);

            k_t[truss_dofs[i] * n + truss_dofs[j]] += k_mat + k_geo;
        }
    }
}

/// Co-rotational treatment for a 2D frame element using the B-matrix formulation.
///
/// Natural deformations: d_n = [d_axial, theta_i_local, theta_j_local]
/// B matrix (3x6): maps 6 global element DOFs to 3 natural deformations.
/// Internal force: f_global = B^T * q, where q = K_nat * d_n.
/// Tangent: K_T = B^T * K_nat * B + K_geo (from Crisfield).
fn assemble_frame_corotational(
    dof_num: &DofNumbering,
    elem: &SolverElement,
    node_i: &SolverNode,
    node_j: &SolverNode,
    e: f64,
    a: f64,
    iz: f64,
    u_full: &[f64],
    f_int: &mut [f64],
    k_t: &mut [f64],
    n: usize,
) {
    let dx0 = node_j.x - node_i.x;
    let dy0 = node_j.y - node_i.y;
    let l0 = (dx0 * dx0 + dy0 * dy0).sqrt();
    let alpha_0 = dy0.atan2(dx0);

    let elem_dofs = dof_num.element_dofs(elem.node_i, elem.node_j);
    let u_elem: Vec<f64> = elem_dofs.iter().map(|&d| u_full[d]).collect();

    // Deformed geometry
    let dx_n = (node_j.x + u_elem[3]) - (node_i.x + u_elem[0]);
    let dy_n = (node_j.y + u_elem[4]) - (node_i.y + u_elem[1]);
    let l_n = (dx_n * dx_n + dy_n * dy_n).sqrt();

    if l_n < 1e-15 {
        return;
    }

    let alpha_n = dy_n.atan2(dx_n);
    let c = dx_n / l_n;
    let s = dy_n / l_n;

    let beta = alpha_n - alpha_0;

    // Natural deformations
    let d_axial = l_n - l0;
    let theta_i = u_elem[2] - beta;
    let theta_j = u_elem[5] - beta;

    // B matrix (3x6): derivative of natural deformations w.r.t. global DOFs
    // Row 0: d(d_axial)/d(u) = [-c, -s, 0, c, s, 0]
    // Row 1: d(theta_i)/d(u) = -d(beta)/d(u) + [0,0,1,0,0,0]
    //   d(beta)/d(u) = d(alpha_n)/d(u) = [s/l_n, -c/l_n, 0, -s/l_n, c/l_n, 0]
    // Row 2: d(theta_j)/d(u) = -d(beta)/d(u) + [0,0,0,0,0,1]
    let b: [[f64; 6]; 3] = [
        [-c, -s, 0.0, c, s, 0.0],
        [-s / l_n, c / l_n, 1.0, s / l_n, -c / l_n, 0.0],
        [-s / l_n, c / l_n, 0.0, s / l_n, -c / l_n, 1.0],
    ];

    // Natural stiffness K_nat (3x3) = P^T * K_local * P
    // where P maps natural to full local: d_local = P * d_natural
    // P = [[1,0,0],[0,0,0],[0,1,0],[-1,0,0],[0,0,0],[0,0,1]]
    let k_local = frame_local_stiffness_2d(e, a, iz, l0, elem.hinge_start, elem.hinge_end, 0.0);

    let p_mat: [[f64; 3]; 6] = [
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
    ];

    let mut k_nat = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            let mut val = 0.0;
            for r in 0..6 {
                for cc in 0..6 {
                    val += p_mat[r][i] * k_local[r * 6 + cc] * p_mat[cc][j];
                }
            }
            k_nat[i][j] = val;
        }
    }

    // Natural internal forces: q = K_nat * d_natural
    let d_natural = [d_axial, theta_i, theta_j];
    let mut q = [0.0; 3]; // [N, Mi, Mj]
    for i in 0..3 {
        for j in 0..3 {
            q[i] += k_nat[i][j] * d_natural[j];
        }
    }

    // Global internal forces: f_int = B^T * q
    let ndof = 6;
    for i in 0..ndof {
        let mut fi = 0.0;
        for k in 0..3 {
            fi += b[k][i] * q[k];
        }
        f_int[elem_dofs[i]] += fi;
    }

    // --- Tangent stiffness ---

    // Part 1: B^T * K_nat * B (6x6)
    let mut knb = [[0.0; 6]; 3];
    for i in 0..3 {
        for j in 0..ndof {
            let mut val = 0.0;
            for k in 0..3 {
                val += k_nat[i][k] * b[k][j];
            }
            knb[i][j] = val;
        }
    }
    let mut k_mat = [[0.0; 6]; 6];
    for i in 0..ndof {
        for j in 0..ndof {
            let mut val = 0.0;
            for k in 0..3 {
                val += b[k][i] * knb[k][j];
            }
            k_mat[i][j] = val;
        }
    }

    // Part 2: Geometric stiffness from derivative of B
    // Reference: Crisfield Vol 1 (17.33), also de Souza Neto et al.
    //
    // r = [-c, -s, 0, c, s, 0] (unit vector along element axis)
    // z = [s, -c, 0, -s, c, 0] (perpendicular)
    //
    // K_geo = (N/l_n) * z*z^T + ((Mi+Mj)/l_n^2) * (z*r^T + r*z^T)
    let nn = q[0]; // axial force
    let mi = q[1]; // moment at i
    let mj = q[2]; // moment at j

    let r_vec = [-c, -s, 0.0, c, s, 0.0];
    let z_vec = [s, -c, 0.0, -s, c, 0.0];

    let n_over_ln = nn / l_n;
    let m_sum_over_ln2 = (mi + mj) / (l_n * l_n);

    let mut k_geo = [[0.0; 6]; 6];
    for i in 0..ndof {
        for j in 0..ndof {
            k_geo[i][j] = n_over_ln * z_vec[i] * z_vec[j]
                + m_sum_over_ln2 * (z_vec[i] * r_vec[j] + r_vec[i] * z_vec[j]);
        }
    }

    // Assemble: K_mat + K_geo
    for i in 0..ndof {
        for j in 0..ndof {
            k_t[elem_dofs[i] * n + elem_dofs[j]] += k_mat[i][j] + k_geo[i][j];
        }
    }
}

/// Subtract fixed-end forces (FEF) from element loads in local coordinates.
fn subtract_element_fef(
    input: &SolverInput,
    elem: &SolverElement,
    l: f64,
    e: f64,
    _a: f64,
    _iz: f64,
    f_local: &mut [f64; 6],
) {
    let sec = input.sections.values().find(|s| s.id == elem.section_id).unwrap();

    for load in &input.loads {
        match load {
            SolverLoad::Distributed(dl) if dl.element_id == elem.id => {
                let a_dist = dl.a.unwrap_or(0.0);
                let b_dist = dl.b.unwrap_or(l);
                let is_full = (a_dist.abs() < 1e-12) && ((b_dist - l).abs() < 1e-12);

                let mut fef = if is_full {
                    fef_distributed_2d(dl.q_i, dl.q_j, l)
                } else {
                    fef_partial_distributed_2d(dl.q_i, dl.q_j, a_dist, b_dist, l)
                };
                adjust_fef_for_hinges(&mut fef, l, elem.hinge_start, elem.hinge_end);

                for i in 0..6 {
                    f_local[i] -= fef[i];
                }
            }
            SolverLoad::PointOnElement(pl) if pl.element_id == elem.id => {
                let px = pl.px.unwrap_or(0.0);
                let mz = pl.mz.unwrap_or(0.0);
                let mut fef = fef_point_load_2d(pl.p, px, mz, pl.a, l);
                adjust_fef_for_hinges(&mut fef, l, elem.hinge_start, elem.hinge_end);

                for i in 0..6 {
                    f_local[i] -= fef[i];
                }
            }
            SolverLoad::Thermal(tl) if tl.element_id == elem.id => {
                let alpha = 12e-6;
                let h = if sec.a > 1e-15 {
                    (12.0 * sec.iz / sec.a).sqrt()
                } else {
                    0.1
                };
                let mut fef = fef_thermal_2d(
                    e, sec.a, sec.iz, l,
                    tl.dt_uniform, tl.dt_gradient, alpha, h,
                );
                adjust_fef_for_hinges(&mut fef, l, elem.hinge_start, elem.hinge_end);

                for i in 0..6 {
                    f_local[i] -= fef[i];
                }
            }
            _ => {}
        }
    }
}

/// Add spring stiffness and internal forces from springs.
fn add_spring_contributions(
    input: &SolverInput,
    dof_num: &DofNumbering,
    u_full: &[f64],
    f_int: &mut [f64],
    k_t: &mut [f64],
) {
    let n = dof_num.n_total;

    for sup in input.supports.values() {
        if sup.support_type != "spring" {
            continue;
        }

        if let Some(kx) = sup.kx {
            if kx > 0.0 {
                if let Some(&d) = dof_num.map.get(&(sup.node_id, 0)) {
                    k_t[d * n + d] += kx;
                    f_int[d] += kx * u_full[d];
                }
            }
        }
        if let Some(ky) = sup.ky {
            if ky > 0.0 {
                if let Some(&d) = dof_num.map.get(&(sup.node_id, 1)) {
                    k_t[d * n + d] += ky;
                    f_int[d] += ky * u_full[d];
                }
            }
        }
        if let Some(kz) = sup.kz {
            if kz > 0.0 && dof_num.dofs_per_node >= 3 {
                if let Some(&d) = dof_num.map.get(&(sup.node_id, 2)) {
                    k_t[d * n + d] += kz;
                    f_int[d] += kz * u_full[d];
                }
            }
        }
    }
}

/// Solve the free-DOF system K_ff * x = r using dense Cholesky with LU fallback.
fn solve_free_dofs(k_ff: &[f64], r_f: &[f64], nf: usize) -> Result<Vec<f64>, String> {
    let mut k_work = k_ff.to_vec();
    match cholesky_solve(&mut k_work, r_f, nf) {
        Some(u) => Ok(u),
        None => {
            let mut k_work = k_ff.to_vec();
            let mut r_work = r_f.to_vec();
            lu_solve(&mut k_work, &mut r_work, nf)
                .ok_or_else(|| {
                    "Singular tangent stiffness — structure may be a mechanism \
                     or load increment too large"
                        .to_string()
                })
        }
    }
}

/// Find the maximum displacement magnitude across all nodes.
fn compute_max_displacement(dof_num: &DofNumbering, u_full: &[f64]) -> f64 {
    let mut max_disp = 0.0f64;
    for &node_id in &dof_num.node_order {
        let ux = dof_num.global_dof(node_id, 0).map(|d| u_full[d]).unwrap_or(0.0);
        let uy = dof_num.global_dof(node_id, 1).map(|d| u_full[d]).unwrap_or(0.0);
        let disp = (ux * ux + uy * uy).sqrt();
        if disp > max_disp {
            max_disp = disp;
        }
    }
    max_disp
}

/// Build final AnalysisResults from the converged displacement field.
fn build_final_results(
    input: &SolverInput,
    dof_num: &DofNumbering,
    u_full: &[f64],
) -> Result<AnalysisResults, String> {
    let n = dof_num.n_total;
    let nf = dof_num.n_free;

    let displacements = super::linear::build_displacements_2d(dof_num, u_full);

    // Reactions: co-rotational internal forces at restrained DOFs minus external loads
    let asm = assembly::assemble_2d(input, dof_num);
    let nr = n - nf;
    let f_r: Vec<f64> = (nf..n).map(|i| asm.f[i]).collect();

    let mut f_int_corot = vec![0.0; n];
    let mut k_dummy = vec![0.0; n * n];
    assemble_corotational(input, dof_num, u_full, &mut f_int_corot, &mut k_dummy);
    add_spring_contributions(input, dof_num, u_full, &mut f_int_corot, &mut k_dummy);

    let mut reactions_vec = vec![0.0; nr];
    for i in 0..nr {
        reactions_vec[i] = f_int_corot[nf + i] - asm.f[nf + i];
    }

    let mut reactions = super::linear::build_reactions_2d(
        input, dof_num, &reactions_vec, &f_r, nf, u_full,
    );
    reactions.sort_by_key(|r| r.node_id);

    let mut element_forces = compute_corotational_forces(input, dof_num, u_full);
    element_forces.sort_by_key(|ef| ef.element_id);

    Ok(AnalysisResults {
        displacements,
        reactions,
        element_forces,
    })
}

/// Compute element internal forces using the co-rotational formulation.
fn compute_corotational_forces(
    input: &SolverInput,
    dof_num: &DofNumbering,
    u_full: &[f64],
) -> Vec<ElementForces> {
    let mut forces = Vec::new();

    for elem in input.elements.values() {
        let node_i = input.nodes.values().find(|nd| nd.id == elem.node_i).unwrap();
        let node_j = input.nodes.values().find(|nd| nd.id == elem.node_j).unwrap();
        let mat = input.materials.values().find(|m| m.id == elem.material_id).unwrap();
        let sec = input.sections.values().find(|s| s.id == elem.section_id).unwrap();

        let e = mat.e * 1000.0;

        if elem.elem_type == "truss" {
            let dx0 = node_j.x - node_i.x;
            let dy0 = node_j.y - node_i.y;
            let l0 = (dx0 * dx0 + dy0 * dy0).sqrt();

            let u_xi = dof_num.global_dof(elem.node_i, 0).map(|d| u_full[d]).unwrap_or(0.0);
            let u_yi = dof_num.global_dof(elem.node_i, 1).map(|d| u_full[d]).unwrap_or(0.0);
            let u_xj = dof_num.global_dof(elem.node_j, 0).map(|d| u_full[d]).unwrap_or(0.0);
            let u_yj = dof_num.global_dof(elem.node_j, 1).map(|d| u_full[d]).unwrap_or(0.0);

            let dx_n = (node_j.x + u_xj) - (node_i.x + u_xi);
            let dy_n = (node_j.y + u_yj) - (node_i.y + u_yi);
            let l_n = (dx_n * dx_n + dy_n * dy_n).sqrt();

            let d_axial = l_n - l0;
            let n_axial = e * sec.a / l0 * d_axial;

            forces.push(ElementForces {
                element_id: elem.id,
                n_start: n_axial,
                n_end: n_axial,
                v_start: 0.0,
                v_end: 0.0,
                m_start: 0.0,
                m_end: 0.0,
                length: l_n,
                q_i: 0.0,
                q_j: 0.0,
                point_loads: Vec::new(),
                distributed_loads: Vec::new(),
                hinge_start: false,
                hinge_end: false,
            });
        } else {
            let dx0 = node_j.x - node_i.x;
            let dy0 = node_j.y - node_i.y;
            let l0 = (dx0 * dx0 + dy0 * dy0).sqrt();
            let alpha_0 = dy0.atan2(dx0);

            let elem_dofs = dof_num.element_dofs(elem.node_i, elem.node_j);
            let u_elem: Vec<f64> = elem_dofs.iter().map(|&d| u_full[d]).collect();

            let dx_n = (node_j.x + u_elem[3]) - (node_i.x + u_elem[0]);
            let dy_n = (node_j.y + u_elem[4]) - (node_i.y + u_elem[1]);
            let l_n = (dx_n * dx_n + dy_n * dy_n).sqrt();
            let alpha_n = dy_n.atan2(dx_n);

            let beta = alpha_n - alpha_0;
            let d_axial = l_n - l0;
            let theta_i_local = u_elem[2] - beta;
            let theta_j_local = u_elem[5] - beta;

            // Co-rotational local deformation: node i at origin, node j displaced axially
            let d_local = [0.0, 0.0, theta_i_local, d_axial, 0.0, theta_j_local];

            let k_local = frame_local_stiffness_2d(
                e, sec.a, sec.iz, l0, elem.hinge_start, elem.hinge_end, 0.0,
            );

            // f_local = K_local * d_local
            let mut f_local = [0.0; 6];
            for i in 0..6 {
                for j in 0..6 {
                    f_local[i] += k_local[i * 6 + j] * d_local[j];
                }
            }

            // Subtract FEF for output (element forces = K*u - FEF)
            subtract_element_fef(input, elem, l0, e, sec.a, sec.iz, &mut f_local);

            // Collect load metadata
            let (mut total_qi, mut total_qj) = (0.0, 0.0);
            let mut point_loads_info = Vec::new();
            let mut dist_loads_info = Vec::new();

            for load in &input.loads {
                match load {
                    SolverLoad::Distributed(dl) if dl.element_id == elem.id => {
                        let a_dist = dl.a.unwrap_or(0.0);
                        let b_dist = dl.b.unwrap_or(l0);
                        let is_full = (a_dist.abs() < 1e-12) && ((b_dist - l0).abs() < 1e-12);
                        if is_full {
                            total_qi += dl.q_i;
                            total_qj += dl.q_j;
                        }
                        dist_loads_info.push(DistributedLoadInfo {
                            q_i: dl.q_i,
                            q_j: dl.q_j,
                            a: a_dist,
                            b: b_dist,
                        });
                    }
                    SolverLoad::PointOnElement(pl) if pl.element_id == elem.id => {
                        point_loads_info.push(PointLoadInfo {
                            a: pl.a,
                            p: pl.p,
                            px: pl.px,
                            mz: pl.mz,
                        });
                    }
                    _ => {}
                }
            }

            // Sign convention: internal forces from member perspective
            forces.push(ElementForces {
                element_id: elem.id,
                n_start: -f_local[0],
                n_end: f_local[3],
                v_start: f_local[1],
                v_end: -f_local[4],
                m_start: f_local[2],
                m_end: -f_local[5],
                length: l_n,
                q_i: total_qi,
                q_j: total_qj,
                point_loads: point_loads_info,
                distributed_loads: dist_loads_info,
                hinge_start: elem.hinge_start,
                hinge_end: elem.hinge_end,
            });
        }
    }

    forces
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn cantilever_input(length: f64, load_fy: f64) -> SolverInput {
        let mut nodes = HashMap::new();
        nodes.insert("1".into(), SolverNode { id: 1, x: 0.0, y: 0.0 });
        nodes.insert("2".into(), SolverNode { id: 2, x: length, y: 0.0 });

        let mut materials = HashMap::new();
        materials.insert("1".into(), SolverMaterial { id: 1, e: 200.0, nu: 0.3 });

        let mut sections = HashMap::new();
        sections.insert("1".into(), SolverSection { id: 1, a: 0.01, iz: 1e-4, as_y: None });

        let mut elements = HashMap::new();
        elements.insert("1".into(), SolverElement {
            id: 1,
            elem_type: "frame".into(),
            node_i: 1,
            node_j: 2,
            material_id: 1,
            section_id: 1,
            hinge_start: false,
            hinge_end: false,
        });

        let mut supports = HashMap::new();
        supports.insert("1".into(), SolverSupport {
            id: 1,
            node_id: 1,
            support_type: "fixed".into(),
            kx: None, ky: None, kz: None,
            dx: None, dy: None, drz: None,
            angle: None,
        });

        let loads = vec![
            SolverLoad::Nodal(SolverNodalLoad {
                node_id: 2,
                fx: 0.0,
                fy: load_fy,
                mz: 0.0,
            }),
        ];

        SolverInput { nodes, materials, sections, elements, supports, loads }
    }

    #[test]
    fn test_corotational_small_load_matches_linear() {
        // With a very small load, co-rotational should closely match linear
        // E=200 MPa -> E_actual=200000 kN/m^2, A=0.01 m^2, Iz=1e-4 m^4, L=3m
        // Use a very small load so geometric effects are negligible
        let input = cantilever_input(3.0, -0.001);

        let linear = super::super::linear::solve_2d(&input).unwrap();
        let corot = solve_corotational_2d(&input, 50, 1e-6, 1).unwrap();

        assert!(corot.converged, "Should converge");

        let lin_d = linear.displacements.iter().find(|d| d.node_id == 2).unwrap();
        let cor_d = corot.results.displacements.iter().find(|d| d.node_id == 2).unwrap();

        // For very small loads, uy should match closely
        let rel_tol_uy = (lin_d.uy - cor_d.uy).abs() / lin_d.uy.abs().max(1e-15);
        assert!(
            rel_tol_uy < 1e-3,
            "uy relative mismatch too large: linear={}, corot={}, rel={}",
            lin_d.uy, cor_d.uy, rel_tol_uy
        );

        // ux: linear gives 0, co-rotational gives a small shortening
        // For small loads this should be negligible
        assert!(
            cor_d.ux.abs() < 1e-6,
            "ux should be negligible for small load, got {}",
            cor_d.ux
        );
    }

    #[test]
    fn test_corotational_converges_with_increments() {
        let input = cantilever_input(3.0, -50.0);

        let corot = solve_corotational_2d(&input, 100, 1e-6, 10).unwrap();
        assert!(corot.converged, "Should converge with 10 increments");
        assert!(corot.max_displacement > 0.0, "Should have nonzero displacement");
        assert!(corot.iterations > 0, "Should take some iterations");
    }

    #[test]
    fn test_corotational_axial_truss() {
        let mut nodes = HashMap::new();
        nodes.insert("1".into(), SolverNode { id: 1, x: 0.0, y: 0.0 });
        nodes.insert("2".into(), SolverNode { id: 2, x: 5.0, y: 0.0 });

        let mut materials = HashMap::new();
        materials.insert("1".into(), SolverMaterial { id: 1, e: 200.0, nu: 0.3 });

        let mut sections = HashMap::new();
        sections.insert("1".into(), SolverSection { id: 1, a: 0.01, iz: 0.0, as_y: None });

        let mut elements = HashMap::new();
        elements.insert("1".into(), SolverElement {
            id: 1,
            elem_type: "truss".into(),
            node_i: 1,
            node_j: 2,
            material_id: 1,
            section_id: 1,
            hinge_start: false,
            hinge_end: false,
        });

        let mut supports = HashMap::new();
        supports.insert("1".into(), SolverSupport {
            id: 1,
            node_id: 1,
            support_type: "pinned".into(),
            kx: None, ky: None, kz: None,
            dx: None, dy: None, drz: None,
            angle: None,
        });
        supports.insert("2".into(), SolverSupport {
            id: 2,
            node_id: 2,
            support_type: "rollerX".into(),
            kx: None, ky: None, kz: None,
            dx: None, dy: None, drz: None,
            angle: None,
        });

        let loads = vec![
            SolverLoad::Nodal(SolverNodalLoad {
                node_id: 2,
                fx: 100.0,
                fy: 0.0,
                mz: 0.0,
            }),
        ];

        let input = SolverInput { nodes, materials, sections, elements, supports, loads };
        let corot = solve_corotational_2d(&input, 50, 1e-8, 1).unwrap();

        assert!(corot.converged);
        let ef = &corot.results.element_forces[0];
        assert!(
            (ef.n_start - 100.0).abs() < 0.1,
            "Axial force should be ~100 kN, got {}",
            ef.n_start
        );
    }

    #[test]
    fn test_no_free_dofs_error() {
        let mut nodes = HashMap::new();
        nodes.insert("1".into(), SolverNode { id: 1, x: 0.0, y: 0.0 });
        nodes.insert("2".into(), SolverNode { id: 2, x: 3.0, y: 0.0 });

        let mut materials = HashMap::new();
        materials.insert("1".into(), SolverMaterial { id: 1, e: 200.0, nu: 0.3 });

        let mut sections = HashMap::new();
        sections.insert("1".into(), SolverSection { id: 1, a: 0.01, iz: 1e-4, as_y: None });

        let mut elements = HashMap::new();
        elements.insert("1".into(), SolverElement {
            id: 1,
            elem_type: "frame".into(),
            node_i: 1,
            node_j: 2,
            material_id: 1,
            section_id: 1,
            hinge_start: false,
            hinge_end: false,
        });

        let mut supports = HashMap::new();
        supports.insert("1".into(), SolverSupport {
            id: 1, node_id: 1, support_type: "fixed".into(),
            kx: None, ky: None, kz: None,
            dx: None, dy: None, drz: None, angle: None,
        });
        supports.insert("2".into(), SolverSupport {
            id: 2, node_id: 2, support_type: "fixed".into(),
            kx: None, ky: None, kz: None,
            dx: None, dy: None, drz: None, angle: None,
        });

        let input = SolverInput {
            nodes, materials, sections, elements, supports,
            loads: vec![],
        };

        let result = solve_corotational_2d(&input, 50, 1e-8, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_corotational_two_element_frame() {
        let mut nodes = HashMap::new();
        nodes.insert("1".into(), SolverNode { id: 1, x: 0.0, y: 0.0 });
        nodes.insert("2".into(), SolverNode { id: 2, x: 3.0, y: 0.0 });
        nodes.insert("3".into(), SolverNode { id: 3, x: 3.0, y: 3.0 });

        let mut materials = HashMap::new();
        materials.insert("1".into(), SolverMaterial { id: 1, e: 200.0, nu: 0.3 });

        let mut sections = HashMap::new();
        sections.insert("1".into(), SolverSection { id: 1, a: 0.01, iz: 1e-4, as_y: None });

        let mut elements = HashMap::new();
        elements.insert("1".into(), SolverElement {
            id: 1, elem_type: "frame".into(),
            node_i: 1, node_j: 2, material_id: 1, section_id: 1,
            hinge_start: false, hinge_end: false,
        });
        elements.insert("2".into(), SolverElement {
            id: 2, elem_type: "frame".into(),
            node_i: 2, node_j: 3, material_id: 1, section_id: 1,
            hinge_start: false, hinge_end: false,
        });

        let mut supports = HashMap::new();
        supports.insert("1".into(), SolverSupport {
            id: 1, node_id: 1, support_type: "fixed".into(),
            kx: None, ky: None, kz: None,
            dx: None, dy: None, drz: None, angle: None,
        });
        supports.insert("2".into(), SolverSupport {
            id: 2, node_id: 3, support_type: "pinned".into(),
            kx: None, ky: None, kz: None,
            dx: None, dy: None, drz: None, angle: None,
        });

        let loads = vec![
            SolverLoad::Nodal(SolverNodalLoad {
                node_id: 2, fx: -5.0, fy: -5.0, mz: 0.0,
            }),
        ];

        let input = SolverInput { nodes, materials, sections, elements, supports, loads };
        let corot = solve_corotational_2d(&input, 100, 1e-6, 5).unwrap();
        assert!(corot.converged, "Two-element frame should converge");
        assert_eq!(corot.results.element_forces.len(), 2);
    }
}
