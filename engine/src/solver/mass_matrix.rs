use crate::types::*;
use crate::linalg::*;
use std::collections::HashMap;
use super::dof::DofNumbering;

/// Assemble consistent mass matrix for 2D structure.
/// densities: materialId → density in kg/m³
pub fn assemble_mass_matrix_2d(
    input: &SolverInput,
    dof_num: &DofNumbering,
    densities: &HashMap<String, f64>,
) -> Vec<f64> {
    let n = dof_num.n_total;
    let mut m_global = vec![0.0; n * n];

    for elem in input.elements.values() {
        let node_i = input.nodes.values().find(|nd| nd.id == elem.node_i).unwrap();
        let node_j = input.nodes.values().find(|nd| nd.id == elem.node_j).unwrap();
        let sec = input.sections.values().find(|s| s.id == elem.section_id).unwrap();

        let density = densities.get(&elem.material_id.to_string()).copied().unwrap_or(0.0);
        if density <= 0.0 {
            continue;
        }

        let dx = node_j.x - node_i.x;
        let dy = node_j.y - node_i.y;
        let l = (dx * dx + dy * dy).sqrt();
        let cos = dx / l;
        let sin = dy / l;

        // rho*A in consistent units: density is kg/m³, A is m²
        // Mass = rho*A*L in kg. Convert to kN·s²/m (tonnes): divide by 1000
        let rho_a = density * sec.a / 1000.0; // tonnes/m

        if elem.elem_type == "truss" {
            // Consistent truss mass: rhoAL/6 * [[2,1],[1,2]] per direction
            let m_local = truss_consistent_mass(rho_a, l);
            let truss_dofs = [
                dof_num.global_dof(elem.node_i, 0).unwrap(),
                dof_num.global_dof(elem.node_i, 1).unwrap(),
                dof_num.global_dof(elem.node_j, 0).unwrap(),
                dof_num.global_dof(elem.node_j, 1).unwrap(),
            ];
            for i in 0..4 {
                for j in 0..4 {
                    m_global[truss_dofs[i] * n + truss_dofs[j]] += m_local[i * 4 + j];
                }
            }
        } else {
            let m_local = frame_consistent_mass(rho_a, l, elem.hinge_start, elem.hinge_end);
            let t = crate::element::frame_transform_2d(cos, sin);
            let m_glob = transform_stiffness(&m_local, &t, 6);

            let elem_dofs = dof_num.element_dofs(elem.node_i, elem.node_j);
            let ndof = elem_dofs.len();
            for i in 0..ndof {
                for j in 0..ndof {
                    m_global[elem_dofs[i] * n + elem_dofs[j]] += m_glob[i * ndof + j];
                }
            }
        }
    }

    m_global
}

/// Consistent mass matrix for 2D frame element (6×6 local).
/// rho_a: mass per unit length (tonnes/m = kN·s²/m²)
fn frame_consistent_mass(rho_a: f64, l: f64, hinge_start: bool, hinge_end: bool) -> Vec<f64> {
    let m = rho_a * l / 420.0;
    let mut mat = vec![0.0; 36];

    if !hinge_start && !hinge_end {
        // Standard consistent mass (no hinges)
        // Axial: [140, 0, 0, 70, 0, 0; ...]
        mat[0 * 6 + 0] = 140.0 * m;
        mat[0 * 6 + 3] = 70.0 * m;
        mat[3 * 6 + 0] = 70.0 * m;
        mat[3 * 6 + 3] = 140.0 * m;

        // Transverse:
        mat[1 * 6 + 1] = 156.0 * m;
        mat[1 * 6 + 2] = 22.0 * l * m;
        mat[1 * 6 + 4] = 54.0 * m;
        mat[1 * 6 + 5] = -13.0 * l * m;

        mat[2 * 6 + 1] = 22.0 * l * m;
        mat[2 * 6 + 2] = 4.0 * l * l * m;
        mat[2 * 6 + 4] = 13.0 * l * m;
        mat[2 * 6 + 5] = -3.0 * l * l * m;

        mat[4 * 6 + 1] = 54.0 * m;
        mat[4 * 6 + 2] = 13.0 * l * m;
        mat[4 * 6 + 4] = 156.0 * m;
        mat[4 * 6 + 5] = -22.0 * l * m;

        mat[5 * 6 + 1] = -13.0 * l * m;
        mat[5 * 6 + 2] = -3.0 * l * l * m;
        mat[5 * 6 + 4] = -22.0 * l * m;
        mat[5 * 6 + 5] = 4.0 * l * l * m;
    } else {
        // Simplified: lumped mass for hinged elements
        let total_mass = rho_a * l;
        let half = total_mass / 2.0;
        mat[0 * 6 + 0] = half;
        mat[1 * 6 + 1] = half;
        mat[3 * 6 + 3] = half;
        mat[4 * 6 + 4] = half;
    }

    mat
}

/// Consistent mass matrix for 2D truss element (4×4 global).
fn truss_consistent_mass(rho_a: f64, l: f64) -> [f64; 16] {
    let m = rho_a * l / 6.0;
    let mut mat = [0.0; 16];
    // [[2,0,1,0],[0,2,0,1],[1,0,2,0],[0,1,0,2]] * m
    mat[0 * 4 + 0] = 2.0 * m;
    mat[0 * 4 + 2] = 1.0 * m;
    mat[1 * 4 + 1] = 2.0 * m;
    mat[1 * 4 + 3] = 1.0 * m;
    mat[2 * 4 + 0] = 1.0 * m;
    mat[2 * 4 + 2] = 2.0 * m;
    mat[3 * 4 + 1] = 1.0 * m;
    mat[3 * 4 + 3] = 2.0 * m;
    mat
}

/// Assemble consistent mass matrix for 3D structure.
pub fn assemble_mass_matrix_3d(
    input: &SolverInput3D,
    dof_num: &DofNumbering,
    densities: &HashMap<String, f64>,
) -> Vec<f64> {
    let n = dof_num.n_total;
    let mut m_global = vec![0.0; n * n];
    let left_hand = input.left_hand.unwrap_or(false);

    for elem in input.elements.values() {
        let node_i = input.nodes.values().find(|nd| nd.id == elem.node_i).unwrap();
        let node_j = input.nodes.values().find(|nd| nd.id == elem.node_j).unwrap();
        let sec = input.sections.values().find(|s| s.id == elem.section_id).unwrap();

        let density = densities.get(&elem.material_id.to_string()).copied().unwrap_or(0.0);
        if density <= 0.0 { continue; }

        let dx = node_j.x - node_i.x;
        let dy = node_j.y - node_i.y;
        let dz = node_j.z - node_i.z;
        let l = (dx * dx + dy * dy + dz * dz).sqrt();
        let rho_a = density * sec.a / 1000.0; // tonnes/m

        if elem.elem_type == "truss" {
            // 3D truss: M = ρAL/6 * [[2I₃, I₃],[I₃, 2I₃]]
            let m = rho_a * l / 6.0;
            let truss_dofs: Vec<usize> = (0..3).map(|i| dof_num.global_dof(elem.node_i, i).unwrap())
                .chain((0..3).map(|i| dof_num.global_dof(elem.node_j, i).unwrap()))
                .collect();
            for i in 0..3 {
                m_global[truss_dofs[i] * n + truss_dofs[i]] += 2.0 * m;
                m_global[truss_dofs[i+3] * n + truss_dofs[i+3]] += 2.0 * m;
                m_global[truss_dofs[i] * n + truss_dofs[i+3]] += m;
                m_global[truss_dofs[i+3] * n + truss_dofs[i]] += m;
            }
        } else {
            let m_local = frame_consistent_mass_3d(rho_a, sec.a, sec.iy, sec.iz, l,
                elem.hinge_start, elem.hinge_end);

            let (ex, ey, ez) = crate::element::compute_local_axes_3d(
                node_i.x, node_i.y, node_i.z,
                node_j.x, node_j.y, node_j.z,
                elem.local_yx, elem.local_yy, elem.local_yz,
                elem.roll_angle, left_hand,
            );
            let t = crate::element::frame_transform_3d(&ex, &ey, &ez);
            let m_glob = transform_stiffness(&m_local, &t, 12);

            let elem_dofs = dof_num.element_dofs(elem.node_i, elem.node_j);
            let ndof = elem_dofs.len();
            for i in 0..ndof {
                for j in 0..ndof {
                    m_global[elem_dofs[i] * n + elem_dofs[j]] += m_glob[i * ndof + j];
                }
            }
        }
    }
    m_global
}

/// Consistent mass matrix for 3D frame element (12×12 local).
fn frame_consistent_mass_3d(rho_a: f64, a: f64, iy: f64, iz: f64, l: f64,
    hinge_start: bool, hinge_end: bool) -> Vec<f64> {
    let mut mat = vec![0.0; 144];

    if hinge_start || hinge_end {
        // Lumped mass for hinged elements
        let half = rho_a * l / 2.0;
        for i in 0..3 { // Translational DOFs only
            mat[i * 12 + i] = half;
            mat[(i + 6) * 12 + (i + 6)] = half;
        }
        return mat;
    }

    let m = rho_a * l / 420.0;

    // Axial (DOFs 0, 6)
    mat[0 * 12 + 0] = 140.0 * m;
    mat[0 * 12 + 6] = 70.0 * m;
    mat[6 * 12 + 0] = 70.0 * m;
    mat[6 * 12 + 6] = 140.0 * m;

    // Torsional rotary inertia (DOFs 3, 9): ρ·Ip·L/6 × [[2,1],[1,2]]
    // Ip ≈ Iy + Iz (polar moment approximation)
    let rho_ip = rho_a * (iy + iz) / a;
    let m_tor = rho_ip * l / 6.0;
    mat[3 * 12 + 3] = 2.0 * m_tor;
    mat[3 * 12 + 9] = m_tor;
    mat[9 * 12 + 3] = m_tor;
    mat[9 * 12 + 9] = 2.0 * m_tor;

    // Y-Z bending (DOFs 1, 5, 7, 11) — same as 2D transverse
    mat[1 * 12 + 1] = 156.0 * m;
    mat[1 * 12 + 5] = 22.0 * l * m;
    mat[1 * 12 + 7] = 54.0 * m;
    mat[1 * 12 + 11] = -13.0 * l * m;

    mat[5 * 12 + 1] = 22.0 * l * m;
    mat[5 * 12 + 5] = 4.0 * l * l * m;
    mat[5 * 12 + 7] = 13.0 * l * m;
    mat[5 * 12 + 11] = -3.0 * l * l * m;

    mat[7 * 12 + 1] = 54.0 * m;
    mat[7 * 12 + 5] = 13.0 * l * m;
    mat[7 * 12 + 7] = 156.0 * m;
    mat[7 * 12 + 11] = -22.0 * l * m;

    mat[11 * 12 + 1] = -13.0 * l * m;
    mat[11 * 12 + 5] = -3.0 * l * l * m;
    mat[11 * 12 + 7] = -22.0 * l * m;
    mat[11 * 12 + 11] = 4.0 * l * l * m;

    // X-Z bending (DOFs 2, 4, 8, 10) — same pattern, sign flip on θy coupling
    mat[2 * 12 + 2] = 156.0 * m;
    mat[2 * 12 + 4] = -22.0 * l * m;
    mat[2 * 12 + 8] = 54.0 * m;
    mat[2 * 12 + 10] = 13.0 * l * m;

    mat[4 * 12 + 2] = -22.0 * l * m;
    mat[4 * 12 + 4] = 4.0 * l * l * m;
    mat[4 * 12 + 8] = -13.0 * l * m;
    mat[4 * 12 + 10] = -3.0 * l * l * m;

    mat[8 * 12 + 2] = 54.0 * m;
    mat[8 * 12 + 4] = -13.0 * l * m;
    mat[8 * 12 + 8] = 156.0 * m;
    mat[8 * 12 + 10] = 22.0 * l * m;

    mat[10 * 12 + 2] = 13.0 * l * m;
    mat[10 * 12 + 4] = -3.0 * l * l * m;
    mat[10 * 12 + 8] = 22.0 * l * m;
    mat[10 * 12 + 10] = 4.0 * l * l * m;

    mat
}

/// Compute total mass of 3D structure (in tonnes = kN·s²/m).
pub fn compute_total_mass_3d(
    input: &SolverInput3D,
    densities: &HashMap<String, f64>,
) -> f64 {
    let mut total = 0.0;
    for elem in input.elements.values() {
        let sec = input.sections.values().find(|s| s.id == elem.section_id).unwrap();
        let node_i = input.nodes.values().find(|nd| nd.id == elem.node_i).unwrap();
        let node_j = input.nodes.values().find(|nd| nd.id == elem.node_j).unwrap();
        let density = densities.get(&elem.material_id.to_string()).copied().unwrap_or(0.0);
        let dx = node_j.x - node_i.x;
        let dy = node_j.y - node_i.y;
        let dz = node_j.z - node_i.z;
        let l = (dx * dx + dy * dy + dz * dz).sqrt();
        total += density * sec.a * l / 1000.0;
    }
    total
}

/// Compute total mass of structure (in tonnes = kN·s²/m).
pub fn compute_total_mass(
    input: &SolverInput,
    densities: &HashMap<String, f64>,
) -> f64 {
    let mut total = 0.0;
    for elem in input.elements.values() {
        let sec = input.sections.values().find(|s| s.id == elem.section_id).unwrap();
        let node_i = input.nodes.values().find(|nd| nd.id == elem.node_i).unwrap();
        let node_j = input.nodes.values().find(|nd| nd.id == elem.node_j).unwrap();
        let density = densities.get(&elem.material_id.to_string()).copied().unwrap_or(0.0);
        let dx = node_j.x - node_i.x;
        let dy = node_j.y - node_i.y;
        let l = (dx * dx + dy * dy).sqrt();
        total += density * sec.a * l / 1000.0; // tonnes
    }
    total
}
