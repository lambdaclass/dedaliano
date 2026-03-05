use crate::types::*;
use crate::solver::linear::solve_2d;
use crate::postprocess::diagrams::compute_diagram_value_at;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ==================== Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfluenceLinePoint {
    pub x: f64,
    pub y: f64,
    pub element_id: usize,
    pub t: f64,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfluenceLineResult {
    pub quantity: String,
    pub target_node_id: Option<usize>,
    pub target_element_id: Option<usize>,
    pub target_position: f64,
    pub points: Vec<InfluenceLinePoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfluenceLineInput {
    pub solver: SolverInput,
    pub quantity: String,     // "Ry", "Rx", "Mz", "V", "M"
    #[serde(default)]
    pub target_node_id: Option<usize>,
    #[serde(default)]
    pub target_element_id: Option<usize>,
    #[serde(default = "default_target_position")]
    pub target_position: f64,
    #[serde(default = "default_n_points")]
    pub n_points_per_element: usize,
}

fn default_target_position() -> f64 { 0.5 }
fn default_n_points() -> usize { 20 }

// ==================== Influence Line Computation ====================

/// Compute influence line: move unit load P=1 (downward) across all elements.
pub fn compute_influence_line(input: &InfluenceLineInput) -> Result<InfluenceLineResult, String> {
    if input.solver.nodes.len() < 2 {
        return Err("Need at least 2 nodes".into());
    }
    if input.solver.elements.is_empty() {
        return Err("Need at least 1 element".into());
    }
    if input.solver.supports.is_empty() {
        return Err("Need at least 1 support".into());
    }

    // Build base input (no loads)
    let base = SolverInput {
        nodes: input.solver.nodes.clone(),
        materials: input.solver.materials.clone(),
        sections: input.solver.sections.clone(),
        elements: input.solver.elements.clone(),
        supports: input.solver.supports.clone(),
        loads: Vec::new(),
    };

    // Pre-compute node positions
    let node_pos: HashMap<usize, (f64, f64)> = input.solver.nodes.values()
        .map(|n| (n.id, (n.x, n.y)))
        .collect();

    let mut points = Vec::new();

    for elem in input.solver.elements.values() {
        let (nix, niy) = *node_pos.get(&elem.node_i).unwrap();
        let (njx, njy) = *node_pos.get(&elem.node_j).unwrap();
        let dx = njx - nix;
        let dy = njy - niy;
        let l = (dx * dx + dy * dy).sqrt();
        if l < 1e-6 { continue; }

        let cos_theta = dx / l;
        let sin_theta = dy / l;

        for k in 0..=input.n_points_per_element {
            let t = k as f64 / input.n_points_per_element as f64;
            let a = t * l;
            let wx = nix + t * dx;
            let wy = niy + t * dy;

            // Unit load P=1 downward → perpendicular component
            let p_perp = -cos_theta;
            let p_axial = -sin_theta;

            let mut loads: Vec<SolverLoad> = Vec::new();

            if p_perp.abs() > 1e-10 {
                loads.push(SolverLoad::PointOnElement(SolverPointLoadOnElement {
                    element_id: elem.id,
                    a,
                    p: p_perp,
                    px: None,
                    mz: None,
                }));
            }

            if p_axial.abs() > 1e-10 {
                let fi = p_axial * (1.0 - t);
                let fj = p_axial * t;
                loads.push(SolverLoad::Nodal(SolverNodalLoad {
                    node_id: elem.node_i,
                    fx: fi * cos_theta,
                    fy: fi * sin_theta,
                    mz: 0.0,
                }));
                loads.push(SolverLoad::Nodal(SolverNodalLoad {
                    node_id: elem.node_j,
                    fx: fj * cos_theta,
                    fy: fj * sin_theta,
                    mz: 0.0,
                }));
            }

            let trial_input = SolverInput {
                loads,
                ..base.clone()
            };

            let value = match solve_2d(&trial_input) {
                Ok(result) => {
                    extract_value(&input.quantity, input.target_node_id, input.target_element_id, input.target_position, &result)
                }
                Err(_) => 0.0,
            };

            points.push(InfluenceLinePoint {
                x: wx,
                y: wy,
                element_id: elem.id,
                t,
                value,
            });
        }
    }

    Ok(InfluenceLineResult {
        quantity: input.quantity.clone(),
        target_node_id: input.target_node_id,
        target_element_id: input.target_element_id,
        target_position: input.target_position,
        points,
    })
}

fn extract_value(
    quantity: &str,
    target_node_id: Option<usize>,
    target_element_id: Option<usize>,
    target_position: f64,
    result: &AnalysisResults,
) -> f64 {
    match quantity {
        "Ry" | "Rx" | "Mz" => {
            if let Some(node_id) = target_node_id {
                if let Some(reaction) = result.reactions.iter().find(|r| r.node_id == node_id) {
                    match quantity {
                        "Ry" => reaction.ry,
                        "Rx" => reaction.rx,
                        "Mz" => reaction.mz,
                        _ => 0.0,
                    }
                } else {
                    0.0
                }
            } else {
                0.0
            }
        }
        "V" | "M" => {
            if let Some(elem_id) = target_element_id {
                if let Some(forces) = result.element_forces.iter().find(|f| f.element_id == elem_id) {
                    let kind = if quantity == "V" { "shear" } else { "moment" };
                    compute_diagram_value_at(kind, target_position, forces)
                } else {
                    0.0
                }
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}
