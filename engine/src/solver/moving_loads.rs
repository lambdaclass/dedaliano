use crate::types::*;
use std::collections::HashMap;

/// Moving loads analysis result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MovingLoadEnvelope {
    pub elements: HashMap<String, ElementEnvelope>,
    pub train: LoadTrain,
    pub path: Vec<PathSegment>,
    pub num_positions: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementEnvelope {
    pub m_max_pos: f64,
    pub m_max_neg: f64,
    pub v_max_pos: f64,
    pub v_max_neg: f64,
    pub n_max_pos: f64,
    pub n_max_neg: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathSegment {
    pub element_id: usize,
    pub start_pos: f64,
    pub end_pos: f64,
    pub length: f64,
    pub cos: f64,
    pub sin: f64,
}

/// Solve 2D moving loads analysis (envelope computation).
pub fn solve_moving_loads_2d(input: &MovingLoadInput) -> Result<MovingLoadEnvelope, String> {
    let solver_input = &input.solver;
    let train = &input.train;
    let step = input.step.unwrap_or(0.25);

    // Build load path
    let path = build_load_path(solver_input, input.path_element_ids.as_deref())?;
    if path.is_empty() {
        return Err("No load path found".into());
    }

    let total_length: f64 = path.iter().map(|s| s.length).sum();
    let max_offset: f64 = train.axles.iter().map(|a| a.offset).fold(0.0, f64::max);

    // Initialize envelopes
    let mut envelopes: HashMap<String, ElementEnvelope> = HashMap::new();
    for elem in solver_input.elements.values() {
        envelopes.insert(elem.id.to_string(), ElementEnvelope {
            m_max_pos: 0.0, m_max_neg: 0.0,
            v_max_pos: 0.0, v_max_neg: 0.0,
            n_max_pos: 0.0, n_max_neg: 0.0,
        });
    }

    // Step through positions
    let start_pos = -max_offset;
    let end_pos = total_length;
    let mut pos = start_pos;
    let mut num_positions = 0;

    while pos <= end_pos + 1e-10 {
        num_positions += 1;

        // Build loads for this position
        let mut loads = base_loads(solver_input);

        for axle in &train.axles {
            let axle_pos = pos + axle.offset;
            if axle_pos < -1e-10 || axle_pos > total_length + 1e-10 {
                continue;
            }

            // Find which segment this axle is on
            if let Some((seg, local_pos)) = find_segment(&path, axle_pos) {
                // Decompose weight into perpendicular component (point on element)
                // and axial component (nodal load in element direction)
                let perp_force = -axle.weight; // Downward force

                // Add as point load on element in local Y direction
                loads.push(SolverLoad::PointOnElement(SolverPointLoadOnElement {
                    element_id: seg.element_id,
                    a: local_pos,
                    p: perp_force * seg.cos.powi(2).max(0.0).sqrt().copysign(1.0),
                    px: None,
                    mz: None,
                }));

                // For vertical loads on non-horizontal members, add axial component as well
                // Simplified: project downward force onto element axes
                if seg.sin.abs() > 1e-6 {
                    // Perpendicular component (transverse to element)
                    let p_perp = -axle.weight * seg.cos;
                    // Axial component
                    let p_axial = -axle.weight * seg.sin;

                    // Replace the simple load with proper decomposition
                    loads.pop(); // Remove the one we just added
                    loads.push(SolverLoad::PointOnElement(SolverPointLoadOnElement {
                        element_id: seg.element_id,
                        a: local_pos,
                        p: p_perp,
                        px: Some(p_axial),
                        mz: None,
                    }));
                }
            }
        }

        // Solve with these loads
        let mut modified_input = solver_input.clone();
        modified_input.loads = loads;

        if let Ok(results) = super::linear::solve_2d(&modified_input) {
            // Update envelopes
            for ef in &results.element_forces {
                if let Some(env) = envelopes.get_mut(&ef.element_id.to_string()) {
                    let m_max = ef.m_start.max(ef.m_end);
                    let m_min = ef.m_start.min(ef.m_end);
                    let v_max = ef.v_start.max(ef.v_end);
                    let v_min = ef.v_start.min(ef.v_end);
                    let n_max = ef.n_start.max(ef.n_end);
                    let n_min = ef.n_start.min(ef.n_end);

                    env.m_max_pos = env.m_max_pos.max(m_max);
                    env.m_max_neg = env.m_max_neg.min(m_min);
                    env.v_max_pos = env.v_max_pos.max(v_max);
                    env.v_max_neg = env.v_max_neg.min(v_min);
                    env.n_max_pos = env.n_max_pos.max(n_max);
                    env.n_max_neg = env.n_max_neg.min(n_min);
                }
            }
        }

        pos += step;
    }

    Ok(MovingLoadEnvelope {
        elements: envelopes,
        train: train.clone(),
        path,
        num_positions,
    })
}

fn build_load_path(
    input: &SolverInput,
    path_element_ids: Option<&[usize]>,
) -> Result<Vec<PathSegment>, String> {
    let mut path = Vec::new();
    let mut cum_pos = 0.0;

    let elem_ids: Vec<usize> = if let Some(ids) = path_element_ids {
        ids.to_vec()
    } else {
        // Auto-detect: find connected chain of elements
        auto_detect_path(input)?
    };

    for eid in &elem_ids {
        let elem = input.elements.values().find(|e| e.id == *eid)
            .ok_or_else(|| format!("Element {} not found", eid))?;
        let ni = input.nodes.values().find(|n| n.id == elem.node_i).unwrap();
        let nj = input.nodes.values().find(|n| n.id == elem.node_j).unwrap();
        let dx = nj.x - ni.x;
        let dy = nj.y - ni.y;
        let l = (dx * dx + dy * dy).sqrt();
        let cos = dx / l;
        let sin = dy / l;

        path.push(PathSegment {
            element_id: *eid,
            start_pos: cum_pos,
            end_pos: cum_pos + l,
            length: l,
            cos,
            sin,
        });
        cum_pos += l;
    }

    Ok(path)
}

fn auto_detect_path(input: &SolverInput) -> Result<Vec<usize>, String> {
    // Find a chain: start from leftmost supported node, traverse connected elements
    let mut elem_list: Vec<&SolverElement> = input.elements.values().collect();
    elem_list.sort_by(|a, b| {
        let na = input.nodes.values().find(|n| n.id == a.node_i).unwrap();
        let nb = input.nodes.values().find(|n| n.id == b.node_i).unwrap();
        na.x.partial_cmp(&nb.x).unwrap()
    });

    // Simple: just return elements sorted by their start node X coordinate
    Ok(elem_list.iter().map(|e| e.id).collect())
}

fn find_segment<'a>(path: &'a [PathSegment], global_pos: f64) -> Option<(&'a PathSegment, f64)> {
    for seg in path {
        if global_pos >= seg.start_pos - 1e-10 && global_pos <= seg.end_pos + 1e-10 {
            let local = (global_pos - seg.start_pos).max(0.0).min(seg.length);
            return Some((seg, local));
        }
    }
    None
}

fn base_loads(input: &SolverInput) -> Vec<SolverLoad> {
    // Keep existing permanent loads (nodal and distributed, but not moving point loads)
    input.loads.iter().filter(|l| {
        matches!(l, SolverLoad::Nodal(_) | SolverLoad::Distributed(_) | SolverLoad::Thermal(_))
    }).cloned().collect()
}
