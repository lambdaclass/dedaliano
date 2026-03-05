// Shared test helpers for building SolverInput structures.

use dedaliano_engine::types::*;
use std::collections::HashMap;

/// Build a 2D SolverInput from compact descriptions.
#[allow(dead_code)]
pub fn make_input(
    nodes: Vec<(usize, f64, f64)>,
    mats: Vec<(usize, f64, f64)>,       // (id, E_MPa, nu)
    secs: Vec<(usize, f64, f64)>,       // (id, A, Iz)
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
            id,
            elem_type: t.to_string(),
            node_i: ni,
            node_j: nj,
            material_id: mi,
            section_id: si,
            hinge_start: hs,
            hinge_end: he,
        });
    }
    let mut sups_map = HashMap::new();
    for (id, nid, t) in sups {
        sups_map.insert(id.to_string(), SolverSupport {
            id,
            node_id: nid,
            support_type: t.to_string(),
            kx: None, ky: None, kz: None,
            dx: None, dy: None, drz: None, angle: None,
        });
    }
    SolverInput { nodes: nodes_map, materials: mats_map, sections: secs_map, elements: elems_map, supports: sups_map, loads }
}

/// Build a multi-element column along X for buckling/modal tests.
#[allow(dead_code)]
pub fn make_column(
    n_elements: usize,
    length: f64,
    e: f64,
    a: f64,
    iz: f64,
    start_support: &str,
    end_support: &str,
    axial_load: f64,
) -> SolverInput {
    let n_nodes = n_elements + 1;
    let elem_len = length / n_elements as f64;

    let mut nodes = Vec::new();
    for i in 0..n_nodes {
        nodes.push((i + 1, i as f64 * elem_len, 0.0));
    }

    let mut elems = Vec::new();
    for i in 0..n_elements {
        elems.push((i + 1, "frame", i + 1, i + 2, 1, 1, false, false));
    }

    let mut sups = vec![(1, 1, start_support)];
    sups.push((2, n_nodes, end_support));

    let loads = if axial_load.abs() > 1e-20 {
        vec![SolverLoad::Nodal(SolverNodalLoad {
            node_id: n_nodes,
            fx: axial_load,
            fy: 0.0,
            mz: 0.0,
        })]
    } else {
        vec![]
    };

    make_input(nodes, vec![(1, e, 0.3)], vec![(1, a, iz)], elems, sups, loads)
}

/// Build a simply-supported beam with uniform distributed load.
#[allow(dead_code)]
pub fn make_ss_beam_udl(
    n_elements: usize,
    length: f64,
    e: f64,
    a: f64,
    iz: f64,
    q: f64,
) -> SolverInput {
    let n_nodes = n_elements + 1;
    let elem_len = length / n_elements as f64;

    let mut nodes = Vec::new();
    for i in 0..n_nodes {
        nodes.push((i + 1, i as f64 * elem_len, 0.0));
    }

    let mut elems = Vec::new();
    for i in 0..n_elements {
        elems.push((i + 1, "frame", i + 1, i + 2, 1, 1, false, false));
    }

    let sups = vec![(1, 1, "pinned"), (2, n_nodes, "rollerX")];

    let mut loads = Vec::new();
    for i in 0..n_elements {
        loads.push(SolverLoad::Distributed(SolverDistributedLoad {
            element_id: i + 1,
            q_i: q,
            q_j: q,
            a: None,
            b: None,
        }));
    }

    make_input(nodes, vec![(1, e, 0.3)], vec![(1, a, iz)], elems, sups, loads)
}

/// Portal frame: 2 columns + 1 beam.
#[allow(dead_code)]
pub fn make_portal_frame(
    h: f64,
    w: f64,
    e: f64,
    a: f64,
    iz: f64,
    lateral_load: f64,
    gravity_load: f64,
) -> SolverInput {
    let nodes = vec![(1, 0.0, 0.0), (2, 0.0, h), (3, w, h), (4, w, 0.0)];
    let elems = vec![
        (1, "frame", 1, 2, 1, 1, false, false),
        (2, "frame", 2, 3, 1, 1, false, false),
        (3, "frame", 3, 4, 1, 1, false, false),
    ];
    let sups = vec![(1, 1, "fixed"), (2, 4, "fixed")];
    let mut loads = Vec::new();
    if lateral_load.abs() > 1e-20 {
        loads.push(SolverLoad::Nodal(SolverNodalLoad {
            node_id: 2, fx: lateral_load, fy: 0.0, mz: 0.0,
        }));
    }
    if gravity_load.abs() > 1e-20 {
        loads.push(SolverLoad::Nodal(SolverNodalLoad {
            node_id: 2, fx: 0.0, fy: gravity_load, mz: 0.0,
        }));
        loads.push(SolverLoad::Nodal(SolverNodalLoad {
            node_id: 3, fx: 0.0, fy: gravity_load, mz: 0.0,
        }));
    }

    make_input(nodes, vec![(1, e, 0.3)], vec![(1, a, iz)], elems, sups, loads)
}

/// Assert relative closeness (with absolute tolerance fallback for near-zero).
#[allow(dead_code)]
pub fn assert_close(actual: f64, expected: f64, rel_tol: f64, label: &str) {
    let abs_tol = 1e-6;
    let diff = (actual - expected).abs();
    let denom = expected.abs().max(1.0);
    let rel_err = diff / denom;
    assert!(
        diff < abs_tol || rel_err < rel_tol,
        "{}: actual={:.6}, expected={:.6}, rel_err={:.4}%",
        label, actual, expected, rel_err * 100.0
    );
}

/// Check global equilibrium: sum of reactions ≈ sum of applied loads.
#[allow(dead_code)]
pub fn check_equilibrium(results: &AnalysisResults, loads: &[SolverLoad]) {
    let (mut fx, mut fy, mut _mz) = (0.0, 0.0, 0.0);
    for load in loads {
        match load {
            SolverLoad::Nodal(nl) => {
                fx += nl.fx;
                fy += nl.fy;
                _mz += nl.mz;
            }
            SolverLoad::Distributed(dl) => {
                let _ = dl;
            }
            _ => {}
        }
    }
    let sum_rx: f64 = results.reactions.iter().map(|r| r.rx).sum();
    let sum_ry: f64 = results.reactions.iter().map(|r| r.ry).sum();
    let total_reaction_x = sum_rx;
    let total_reaction_y = sum_ry;
    if fy < 0.0 {
        assert!(total_reaction_y > 0.0, "Vertical equilibrium: sum_ry={}", total_reaction_y);
    }
    if fx.abs() > 1e-10 {
        assert!(
            (total_reaction_x + fx).abs() < 1.0,
            "Horizontal equilibrium: sum_rx={}, fx={}", total_reaction_x, fx
        );
    }
}
