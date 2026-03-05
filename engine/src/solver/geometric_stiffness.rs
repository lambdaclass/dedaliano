use crate::types::*;
use crate::linalg::*;
use super::dof::DofNumbering;

/// Add geometric stiffness to the global stiffness matrix based on current axial forces.
/// Used by P-Delta and Buckling analyses.
pub fn add_geometric_stiffness_2d(
    input: &SolverInput,
    dof_num: &DofNumbering,
    u: &[f64],
    k_global: &mut [f64],
) {
    let n = dof_num.n_total;

    for elem in input.elements.values() {
        if elem.elem_type == "truss" {
            // Truss geometric stiffness
            add_truss_kg_2d(input, dof_num, elem, u, k_global, n);
            continue;
        }

        let node_i = input.nodes.values().find(|n| n.id == elem.node_i).unwrap();
        let node_j = input.nodes.values().find(|n| n.id == elem.node_j).unwrap();
        let mat = input.materials.values().find(|m| m.id == elem.material_id).unwrap();
        let sec = input.sections.values().find(|s| s.id == elem.section_id).unwrap();

        let dx = node_j.x - node_i.x;
        let dy = node_j.y - node_i.y;
        let l = (dx * dx + dy * dy).sqrt();
        let cos = dx / l;
        let sin = dy / l;
        let e = mat.e * 1000.0;

        // Get current axial force from displacements
        let elem_dofs = dof_num.element_dofs(elem.node_i, elem.node_j);
        let u_global: Vec<f64> = elem_dofs.iter().map(|&d| u[d]).collect();
        let t = crate::element::frame_transform_2d(cos, sin);
        let u_local = transform_displacement(&u_global, &t, 6);

        // Axial deformation → axial force
        let axial_force = e * sec.a / l * (u_local[3] - u_local[0]);

        // Geometric stiffness matrix (Przemieniecki formulation)
        let p = axial_force;
        let coeff = p / (30.0 * l);

        // Local geometric stiffness (transverse DOFs: v1, θ1, v2, θ2 → indices 1,2,4,5)
        let mut k_g_local = vec![0.0; 36]; // 6×6
        k_g_local[1 * 6 + 1] = 36.0 * coeff;
        k_g_local[1 * 6 + 2] = 3.0 * l * coeff;
        k_g_local[1 * 6 + 4] = -36.0 * coeff;
        k_g_local[1 * 6 + 5] = 3.0 * l * coeff;

        k_g_local[2 * 6 + 1] = 3.0 * l * coeff;
        k_g_local[2 * 6 + 2] = 4.0 * l * l * coeff;
        k_g_local[2 * 6 + 4] = -3.0 * l * coeff;
        k_g_local[2 * 6 + 5] = -l * l * coeff;

        k_g_local[4 * 6 + 1] = -36.0 * coeff;
        k_g_local[4 * 6 + 2] = -3.0 * l * coeff;
        k_g_local[4 * 6 + 4] = 36.0 * coeff;
        k_g_local[4 * 6 + 5] = -3.0 * l * coeff;

        k_g_local[5 * 6 + 1] = 3.0 * l * coeff;
        k_g_local[5 * 6 + 2] = -l * l * coeff;
        k_g_local[5 * 6 + 4] = -3.0 * l * coeff;
        k_g_local[5 * 6 + 5] = 4.0 * l * l * coeff;

        // Transform to global
        let k_g_global = transform_stiffness(&k_g_local, &t, 6);

        // Add to global matrix
        let ndof = elem_dofs.len();
        for i in 0..ndof {
            for j in 0..ndof {
                k_global[elem_dofs[i] * n + elem_dofs[j]] += k_g_global[i * ndof + j];
            }
        }
    }
}

fn add_truss_kg_2d(
    input: &SolverInput,
    dof_num: &DofNumbering,
    elem: &SolverElement,
    u: &[f64],
    k_global: &mut [f64],
    n: usize,
) {
    let node_i = input.nodes.values().find(|nd| nd.id == elem.node_i).unwrap();
    let node_j = input.nodes.values().find(|nd| nd.id == elem.node_j).unwrap();
    let mat = input.materials.values().find(|m| m.id == elem.material_id).unwrap();
    let sec = input.sections.values().find(|s| s.id == elem.section_id).unwrap();

    let dx = node_j.x - node_i.x;
    let dy = node_j.y - node_i.y;
    let l = (dx * dx + dy * dy).sqrt();
    let cos = dx / l;
    let sin = dy / l;
    let e = mat.e * 1000.0;

    let ui = [
        dof_num.global_dof(elem.node_i, 0).map(|d| u[d]).unwrap_or(0.0),
        dof_num.global_dof(elem.node_i, 1).map(|d| u[d]).unwrap_or(0.0),
    ];
    let uj = [
        dof_num.global_dof(elem.node_j, 0).map(|d| u[d]).unwrap_or(0.0),
        dof_num.global_dof(elem.node_j, 1).map(|d| u[d]).unwrap_or(0.0),
    ];
    let delta = (uj[0] - ui[0]) * cos + (uj[1] - ui[1]) * sin;
    let axial_force = e * sec.a / l * delta;

    // Truss geometric stiffness in global: P/L * [[s²,-cs,-s²,cs],[-cs,c²,cs,-c²],...]
    // where c=cos, s=sin
    let p_over_l = axial_force / l;
    let cc = cos * cos;
    let ss = sin * sin;
    let cs = cos * sin;

    // 4×4 matrix for DOFs: [ux_i, uy_i, ux_j, uy_j]
    let k_g = [
        ss * p_over_l, -cs * p_over_l, -ss * p_over_l, cs * p_over_l,
        -cs * p_over_l, cc * p_over_l, cs * p_over_l, -cc * p_over_l,
        -ss * p_over_l, cs * p_over_l, ss * p_over_l, -cs * p_over_l,
        cs * p_over_l, -cc * p_over_l, -cs * p_over_l, cc * p_over_l,
    ];

    let truss_dofs = [
        dof_num.global_dof(elem.node_i, 0).unwrap(),
        dof_num.global_dof(elem.node_i, 1).unwrap(),
        dof_num.global_dof(elem.node_j, 0).unwrap(),
        dof_num.global_dof(elem.node_j, 1).unwrap(),
    ];

    for i in 0..4 {
        for j in 0..4 {
            k_global[truss_dofs[i] * n + truss_dofs[j]] += k_g[i * 4 + j];
        }
    }
}

/// Build geometric stiffness matrix from element forces (no displacement needed).
/// Used by buckling analysis which already has element forces from linear solve.
pub fn build_kg_from_forces_2d(
    input: &SolverInput,
    dof_num: &DofNumbering,
    element_forces: &[ElementForces],
) -> Vec<f64> {
    let n = dof_num.n_total;
    let mut k_g = vec![0.0; n * n];

    for ef in element_forces {
        let elem = input.elements.values().find(|e| e.id == ef.element_id).unwrap();
        let node_i = input.nodes.values().find(|nd| nd.id == elem.node_i).unwrap();
        let node_j = input.nodes.values().find(|nd| nd.id == elem.node_j).unwrap();

        let dx = node_j.x - node_i.x;
        let dy = node_j.y - node_i.y;
        let l = (dx * dx + dy * dy).sqrt();
        let cos = dx / l;
        let sin = dy / l;

        // Use average axial force (negative = compression)
        let axial_force = (ef.n_start + ef.n_end) / 2.0;

        if elem.elem_type == "truss" {
            let p_over_l = axial_force / l;
            let cc = cos * cos;
            let ss = sin * sin;
            let cs = cos * sin;
            let kg_local = [
                ss * p_over_l, -cs * p_over_l, -ss * p_over_l, cs * p_over_l,
                -cs * p_over_l, cc * p_over_l, cs * p_over_l, -cc * p_over_l,
                -ss * p_over_l, cs * p_over_l, ss * p_over_l, -cs * p_over_l,
                cs * p_over_l, -cc * p_over_l, -cs * p_over_l, cc * p_over_l,
            ];
            let truss_dofs = [
                dof_num.global_dof(elem.node_i, 0).unwrap(),
                dof_num.global_dof(elem.node_i, 1).unwrap(),
                dof_num.global_dof(elem.node_j, 0).unwrap(),
                dof_num.global_dof(elem.node_j, 1).unwrap(),
            ];
            for i in 0..4 {
                for j in 0..4 {
                    k_g[truss_dofs[i] * n + truss_dofs[j]] += kg_local[i * 4 + j];
                }
            }
        } else {
            let t = crate::element::frame_transform_2d(cos, sin);
            let p = axial_force;
            let coeff = p / (30.0 * l);

            let mut kg_local = vec![0.0; 36];
            kg_local[1 * 6 + 1] = 36.0 * coeff;
            kg_local[1 * 6 + 2] = 3.0 * l * coeff;
            kg_local[1 * 6 + 4] = -36.0 * coeff;
            kg_local[1 * 6 + 5] = 3.0 * l * coeff;
            kg_local[2 * 6 + 1] = 3.0 * l * coeff;
            kg_local[2 * 6 + 2] = 4.0 * l * l * coeff;
            kg_local[2 * 6 + 4] = -3.0 * l * coeff;
            kg_local[2 * 6 + 5] = -l * l * coeff;
            kg_local[4 * 6 + 1] = -36.0 * coeff;
            kg_local[4 * 6 + 2] = -3.0 * l * coeff;
            kg_local[4 * 6 + 4] = 36.0 * coeff;
            kg_local[4 * 6 + 5] = -3.0 * l * coeff;
            kg_local[5 * 6 + 1] = 3.0 * l * coeff;
            kg_local[5 * 6 + 2] = -l * l * coeff;
            kg_local[5 * 6 + 4] = -3.0 * l * coeff;
            kg_local[5 * 6 + 5] = 4.0 * l * l * coeff;

            let kg_global = transform_stiffness(&kg_local, &t, 6);
            let elem_dofs = dof_num.element_dofs(elem.node_i, elem.node_j);
            let ndof = elem_dofs.len();
            for i in 0..ndof {
                for j in 0..ndof {
                    k_g[elem_dofs[i] * n + elem_dofs[j]] += kg_global[i * ndof + j];
                }
            }
        }
    }
    k_g
}
