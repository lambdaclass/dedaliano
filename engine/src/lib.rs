pub mod types;
pub mod linalg;
pub mod element;
pub mod solver;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Solve 2D linear static analysis. JSON in → JSON out.
#[wasm_bindgen]
pub fn solve_2d(json: &str) -> Result<String, JsValue> {
    let input: types::SolverInput = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    let results = solver::linear::solve_2d(&input)
        .map_err(|e| JsValue::from_str(&e))?;
    serde_json::to_string(&results)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
}

/// Solve 3D linear static analysis. JSON in → JSON out.
#[wasm_bindgen]
pub fn solve_3d(json: &str) -> Result<String, JsValue> {
    let input: types::SolverInput3D = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    let results = solver::linear::solve_3d(&input)
        .map_err(|e| JsValue::from_str(&e))?;
    serde_json::to_string(&results)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
}

/// Solve 2D P-Delta analysis. JSON in → JSON out.
#[wasm_bindgen]
pub fn solve_pdelta_2d(json: &str, max_iter: usize, tolerance: f64) -> Result<String, JsValue> {
    let input: types::SolverInput = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    let results = solver::pdelta::solve_pdelta_2d(&input, max_iter, tolerance)
        .map_err(|e| JsValue::from_str(&e))?;
    serde_json::to_string(&results)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
}

/// Solve 2D buckling analysis. JSON in → JSON out.
#[wasm_bindgen]
pub fn solve_buckling_2d(json: &str, num_modes: usize) -> Result<String, JsValue> {
    let input: types::SolverInput = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    let results = solver::buckling::solve_buckling_2d(&input, num_modes)
        .map_err(|e| JsValue::from_str(&e))?;
    serde_json::to_string(&results)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
}

/// Solve 2D modal analysis. JSON in → JSON out.
/// densities_json: { "materialId": density_kg_m3, ... }
#[wasm_bindgen]
pub fn solve_modal_2d(json: &str, num_modes: usize) -> Result<String, JsValue> {
    let input: types::ModalInput = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    let results = solver::modal::solve_modal_2d(&input.solver, &input.densities, num_modes)
        .map_err(|e| JsValue::from_str(&e))?;
    serde_json::to_string(&results)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
}

/// Solve 2D spectral analysis. JSON in → JSON out.
#[wasm_bindgen]
pub fn solve_spectral_2d(json: &str) -> Result<String, JsValue> {
    let input: types::SpectralInput = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    let results = solver::spectral::solve_spectral_2d(&input)
        .map_err(|e| JsValue::from_str(&e))?;
    serde_json::to_string(&results)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
}

/// Solve 2D plastic analysis. JSON in → JSON out.
#[wasm_bindgen]
pub fn solve_plastic_2d(json: &str) -> Result<String, JsValue> {
    let input: types::PlasticInput = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    let results = solver::plastic::solve_plastic_2d(&input)
        .map_err(|e| JsValue::from_str(&e))?;
    serde_json::to_string(&results)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
}

/// Solve 2D moving loads analysis. JSON in → JSON out.
#[wasm_bindgen]
pub fn solve_moving_loads_2d(json: &str) -> Result<String, JsValue> {
    let input: types::MovingLoadInput = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    let results = solver::moving_loads::solve_moving_loads_2d(&input)
        .map_err(|e| JsValue::from_str(&e))?;
    serde_json::to_string(&results)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::types::*;
    use std::collections::HashMap;

    fn make_input(
        nodes: Vec<(usize, f64, f64)>,
        mats: Vec<(usize, f64, f64)>,
        secs: Vec<(usize, f64, f64)>,
        elems: Vec<(usize, &str, usize, usize, usize, usize, bool, bool)>,
        sups: Vec<(usize, usize, &str)>,
        loads: Vec<SolverLoad>,
    ) -> SolverInput {
        let mut nodes_map = HashMap::new();
        for (id, x, y) in nodes {
            nodes_map.insert(id.to_string(), SolverNode { id, x, y });
        }
        let mut mats_map = HashMap::new();
        for (id, e, nu) in mats {
            mats_map.insert(id.to_string(), SolverMaterial { id, e, nu });
        }
        let mut secs_map = HashMap::new();
        for (id, a, iz) in secs {
            secs_map.insert(id.to_string(), SolverSection { id, a, iz });
        }
        let mut elems_map = HashMap::new();
        for (id, t, ni, nj, mi, si, hs, he) in elems {
            elems_map.insert(id.to_string(), SolverElement {
                id, elem_type: t.to_string(), node_i: ni, node_j: nj,
                material_id: mi, section_id: si, hinge_start: hs, hinge_end: he,
            });
        }
        let mut sups_map = HashMap::new();
        for (id, nid, t) in sups {
            sups_map.insert(id.to_string(), SolverSupport {
                id, node_id: nid, support_type: t.to_string(),
                kx: None, ky: None, kz: None, dx: None, dy: None, drz: None, angle: None,
            });
        }
        SolverInput { nodes: nodes_map, materials: mats_map, sections: secs_map, elements: elems_map, supports: sups_map, loads }
    }

    #[test]
    fn test_simply_supported_beam() {
        let input = make_input(
            vec![(1, 0.0, 0.0), (2, 6.0, 0.0)],
            vec![(1, 200000.0, 0.3)], // E in MPa
            vec![(1, 0.15, 0.003125)], // A=0.3*0.5, Iz=0.3*0.5^3/12
            vec![(1, "frame", 1, 2, 1, 1, false, false)],
            vec![(1, 1, "pinned"), (2, 2, "rollerX")],
            vec![SolverLoad::Distributed(SolverDistributedLoad {
                element_id: 1, q_i: -10.0, q_j: -10.0, a: None, b: None,
            })],
        );
        let results = super::solver::linear::solve_2d(&input).unwrap();
        let r1 = results.reactions.iter().find(|r| r.node_id == 1).unwrap();
        let r2 = results.reactions.iter().find(|r| r.node_id == 2).unwrap();
        assert!((r1.ry - 30.0).abs() < 0.5, "R1y={}", r1.ry);
        assert!((r2.ry - 30.0).abs() < 0.5, "R2y={}", r2.ry);
    }

    #[test]
    fn test_cantilever() {
        let input = make_input(
            vec![(1, 0.0, 0.0), (2, 4.0, 0.0)],
            vec![(1, 200000.0, 0.3)],
            vec![(1, 0.15, 0.003125)],
            vec![(1, "frame", 1, 2, 1, 1, false, false)],
            vec![(1, 1, "fixed")],
            vec![SolverLoad::Nodal(SolverNodalLoad { node_id: 2, fx: 0.0, fy: -50.0, mz: 0.0 })],
        );
        let results = super::solver::linear::solve_2d(&input).unwrap();
        let r1 = results.reactions.iter().find(|r| r.node_id == 1).unwrap();
        assert!((r1.ry - 50.0).abs() < 0.5, "Ry={}", r1.ry);
        assert!((r1.mz.abs() - 200.0).abs() < 1.0, "Mz={}", r1.mz);
    }

    #[test]
    fn test_truss() {
        let input = make_input(
            vec![(1, 0.0, 0.0), (2, 4.0, 0.0), (3, 2.0, 3.0)],
            vec![(1, 200000.0, 0.3)],
            vec![(1, 0.001, 0.0)],
            vec![
                (1, "truss", 1, 2, 1, 1, false, false),
                (2, "truss", 1, 3, 1, 1, false, false),
                (3, "truss", 2, 3, 1, 1, false, false),
            ],
            vec![(1, 1, "pinned"), (2, 2, "rollerX")],
            vec![SolverLoad::Nodal(SolverNodalLoad { node_id: 3, fx: 0.0, fy: -10.0, mz: 0.0 })],
        );
        let results = super::solver::linear::solve_2d(&input).unwrap();
        let r1 = results.reactions.iter().find(|r| r.node_id == 1).unwrap();
        let r2 = results.reactions.iter().find(|r| r.node_id == 2).unwrap();
        assert!((r1.ry + r2.ry - 10.0).abs() < 0.01);
        assert!((r1.ry - 5.0).abs() < 0.01);
    }
}
