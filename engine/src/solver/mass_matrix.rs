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
