/// Validation tests for Timoshenko beam implementation.
///
/// Timoshenko beam theory accounts for shear deformation in addition to bending.
/// The shear parameter phi = 12*E*I / (G*As*L^2) modifies the stiffness matrix.
/// For deep beams (small L/d), shear deformation is significant and Timoshenko
/// deflections exceed Euler-Bernoulli deflections. For slender beams (large L/d),
/// both theories converge.
///
/// Key analytical results:
///   Cantilever tip deflection:  delta_T = PL^3/(3EI) + PL/(kAG)
///   SS midspan deflection:      delta_T = 5qL^4/(384EI) * (1 + 12.5*phi/L_ratio)
///   Fixed-fixed midspan:        delta_T = PL^3/(192EI) * (1 + phi)
///
/// Tests run the solver twice (without/with asY) and compare.

mod helpers;

use dedaliano_engine::solver::linear;
use dedaliano_engine::types::*;
use helpers::*;
use std::collections::HashMap;

// Steel properties
const E: f64 = 200_000.0;   // MPa
const NU: f64 = 0.3;
const G: f64 = E / (2.0 * (1.0 + NU)); // ~76923 MPa

// Rectangular section 300mm deep x 150mm wide
const B_RECT: f64 = 0.150;  // m
const D_RECT: f64 = 0.300;  // m
const A_RECT: f64 = B_RECT * D_RECT;                          // 0.045 m^2
const IZ_RECT: f64 = B_RECT * D_RECT * D_RECT * D_RECT / 12.0; // 3.375e-4 m^4
const KAPPA: f64 = 5.0 / 6.0;                                  // shear correction factor
const AS_Y_RECT: f64 = KAPPA * A_RECT;                         // 0.0375 m^2

/// Helper: Build a cantilever beam (fixed at left, free at right) with tip load P.
/// Returns SolverInput. When as_y is Some, Timoshenko beam is activated.
fn make_cantilever_tip_load(
    n_elements: usize,
    length: f64,
    a: f64,
    iz: f64,
    as_y: Option<f64>,
    p: f64,
) -> SolverInput {
    let n_nodes = n_elements + 1;
    let elem_len = length / n_elements as f64;

    let mut nodes_map = HashMap::new();
    for i in 0..n_nodes {
        nodes_map.insert(
            (i + 1).to_string(),
            SolverNode { id: i + 1, x: i as f64 * elem_len, y: 0.0 },
        );
    }

    let mut mats_map = HashMap::new();
    mats_map.insert("1".to_string(), SolverMaterial { id: 1, e: E, nu: NU });

    let mut secs_map = HashMap::new();
    secs_map.insert("1".to_string(), SolverSection { id: 1, a, iz, as_y });

    let mut elems_map = HashMap::new();
    for i in 0..n_elements {
        elems_map.insert(
            (i + 1).to_string(),
            SolverElement {
                id: i + 1,
                elem_type: "frame".to_string(),
                node_i: i + 1,
                node_j: i + 2,
                material_id: 1,
                section_id: 1,
                hinge_start: false,
                hinge_end: false,
            },
        );
    }

    let mut sups_map = HashMap::new();
    sups_map.insert("1".to_string(), SolverSupport {
        id: 1, node_id: 1, support_type: "fixed".to_string(),
        kx: None, ky: None, kz: None, dx: None, dy: None, drz: None, angle: None,
    });

    let loads = vec![SolverLoad::Nodal(SolverNodalLoad {
        node_id: n_nodes, fx: 0.0, fy: -p, mz: 0.0,
    })];

    SolverInput { nodes: nodes_map, materials: mats_map, sections: secs_map, elements: elems_map, supports: sups_map, loads }
}

/// Helper: Build a simply-supported beam with UDL.
/// Returns SolverInput. When as_y is Some, Timoshenko beam is activated.
fn make_ss_beam_udl_timo(
    n_elements: usize,
    length: f64,
    a: f64,
    iz: f64,
    as_y: Option<f64>,
    q: f64,
) -> SolverInput {
    let n_nodes = n_elements + 1;
    let elem_len = length / n_elements as f64;

    let mut nodes_map = HashMap::new();
    for i in 0..n_nodes {
        nodes_map.insert(
            (i + 1).to_string(),
            SolverNode { id: i + 1, x: i as f64 * elem_len, y: 0.0 },
        );
    }

    let mut mats_map = HashMap::new();
    mats_map.insert("1".to_string(), SolverMaterial { id: 1, e: E, nu: NU });

    let mut secs_map = HashMap::new();
    secs_map.insert("1".to_string(), SolverSection { id: 1, a, iz, as_y });

    let mut elems_map = HashMap::new();
    for i in 0..n_elements {
        elems_map.insert(
            (i + 1).to_string(),
            SolverElement {
                id: i + 1,
                elem_type: "frame".to_string(),
                node_i: i + 1,
                node_j: i + 2,
                material_id: 1,
                section_id: 1,
                hinge_start: false,
                hinge_end: false,
            },
        );
    }

    let mut sups_map = HashMap::new();
    sups_map.insert("1".to_string(), SolverSupport {
        id: 1, node_id: 1, support_type: "pinned".to_string(),
        kx: None, ky: None, kz: None, dx: None, dy: None, drz: None, angle: None,
    });
    sups_map.insert("2".to_string(), SolverSupport {
        id: 2, node_id: n_nodes, support_type: "rollerX".to_string(),
        kx: None, ky: None, kz: None, dx: None, dy: None, drz: None, angle: None,
    });

    let loads: Vec<SolverLoad> = (0..n_elements)
        .map(|i| SolverLoad::Distributed(SolverDistributedLoad {
            element_id: i + 1, q_i: q, q_j: q, a: None, b: None,
        }))
        .collect();

    SolverInput { nodes: nodes_map, materials: mats_map, sections: secs_map, elements: elems_map, supports: sups_map, loads }
}

/// Helper: Build a fixed-fixed beam with midspan point load.
fn make_fixed_fixed_midspan_load(
    n_elements: usize,
    length: f64,
    a: f64,
    iz: f64,
    as_y: Option<f64>,
    p: f64,
) -> SolverInput {
    let n_nodes = n_elements + 1;
    let elem_len = length / n_elements as f64;

    let mut nodes_map = HashMap::new();
    for i in 0..n_nodes {
        nodes_map.insert(
            (i + 1).to_string(),
            SolverNode { id: i + 1, x: i as f64 * elem_len, y: 0.0 },
        );
    }

    let mut mats_map = HashMap::new();
    mats_map.insert("1".to_string(), SolverMaterial { id: 1, e: E, nu: NU });

    let mut secs_map = HashMap::new();
    secs_map.insert("1".to_string(), SolverSection { id: 1, a, iz, as_y });

    let mut elems_map = HashMap::new();
    for i in 0..n_elements {
        elems_map.insert(
            (i + 1).to_string(),
            SolverElement {
                id: i + 1,
                elem_type: "frame".to_string(),
                node_i: i + 1,
                node_j: i + 2,
                material_id: 1,
                section_id: 1,
                hinge_start: false,
                hinge_end: false,
            },
        );
    }

    let mut sups_map = HashMap::new();
    sups_map.insert("1".to_string(), SolverSupport {
        id: 1, node_id: 1, support_type: "fixed".to_string(),
        kx: None, ky: None, kz: None, dx: None, dy: None, drz: None, angle: None,
    });
    sups_map.insert("2".to_string(), SolverSupport {
        id: 2, node_id: n_nodes, support_type: "fixed".to_string(),
        kx: None, ky: None, kz: None, dx: None, dy: None, drz: None, angle: None,
    });

    let mid_node = n_elements / 2 + 1;
    let loads = vec![SolverLoad::Nodal(SolverNodalLoad {
        node_id: mid_node, fx: 0.0, fy: -p, mz: 0.0,
    })];

    SolverInput { nodes: nodes_map, materials: mats_map, sections: secs_map, elements: elems_map, supports: sups_map, loads }
}

/// Helper: Build a propped cantilever (fixed at left, roller at right) with UDL.
fn make_propped_cantilever_udl(
    n_elements: usize,
    length: f64,
    a: f64,
    iz: f64,
    as_y: Option<f64>,
    q: f64,
) -> SolverInput {
    let n_nodes = n_elements + 1;
    let elem_len = length / n_elements as f64;

    let mut nodes_map = HashMap::new();
    for i in 0..n_nodes {
        nodes_map.insert(
            (i + 1).to_string(),
            SolverNode { id: i + 1, x: i as f64 * elem_len, y: 0.0 },
        );
    }

    let mut mats_map = HashMap::new();
    mats_map.insert("1".to_string(), SolverMaterial { id: 1, e: E, nu: NU });

    let mut secs_map = HashMap::new();
    secs_map.insert("1".to_string(), SolverSection { id: 1, a, iz, as_y });

    let mut elems_map = HashMap::new();
    for i in 0..n_elements {
        elems_map.insert(
            (i + 1).to_string(),
            SolverElement {
                id: i + 1,
                elem_type: "frame".to_string(),
                node_i: i + 1,
                node_j: i + 2,
                material_id: 1,
                section_id: 1,
                hinge_start: false,
                hinge_end: false,
            },
        );
    }

    let mut sups_map = HashMap::new();
    sups_map.insert("1".to_string(), SolverSupport {
        id: 1, node_id: 1, support_type: "fixed".to_string(),
        kx: None, ky: None, kz: None, dx: None, dy: None, drz: None, angle: None,
    });
    sups_map.insert("2".to_string(), SolverSupport {
        id: 2, node_id: n_nodes, support_type: "rollerX".to_string(),
        kx: None, ky: None, kz: None, dx: None, dy: None, drz: None, angle: None,
    });

    let loads: Vec<SolverLoad> = (0..n_elements)
        .map(|i| SolverLoad::Distributed(SolverDistributedLoad {
            element_id: i + 1, q_i: q, q_j: q, a: None, b: None,
        }))
        .collect();

    SolverInput { nodes: nodes_map, materials: mats_map, sections: secs_map, elements: elems_map, supports: sups_map, loads }
}


// ================================================================
// 1. Deep Cantilever (L/d = 2): Timoshenko deflection > EB
// ================================================================
//
// Cantilever with tip load P:
//   EB:   delta = PL^3 / (3EI)
//   Timo: delta = PL^3 / (3EI) + PL / (kAG)
//
// For L/d = 2: L = 0.6m, d = 0.3m.
// phi = 12*E*I / (G*As*L^2) is large for deep beams.

#[test]
fn deep_cantilever_l_over_d_2() {
    let l = 2.0 * D_RECT; // 0.6m
    let p = 100.0;         // kN
    let n = 8;

    // E in kN/m^2 for structural solver
    let e_kn = E * 1000.0;
    let g_kn = G * 1000.0;

    // Analytical Euler-Bernoulli
    let delta_eb = p * l.powi(3) / (3.0 * e_kn * IZ_RECT);

    // Analytical Timoshenko: adds shear deflection
    let delta_shear = p * l / (AS_Y_RECT * g_kn);
    let delta_timo = delta_eb + delta_shear;

    // Run solver without asY (Euler-Bernoulli)
    let input_eb = make_cantilever_tip_load(n, l, A_RECT, IZ_RECT, None, p);
    let results_eb = linear::solve_2d(&input_eb).unwrap();
    let tip_eb = results_eb.displacements.iter()
        .find(|d| d.node_id == n + 1).unwrap().uy.abs();

    // Run solver with asY (Timoshenko)
    let input_timo = make_cantilever_tip_load(n, l, A_RECT, IZ_RECT, Some(AS_Y_RECT), p);
    let results_timo = linear::solve_2d(&input_timo).unwrap();
    let tip_timo = results_timo.displacements.iter()
        .find(|d| d.node_id == n + 1).unwrap().uy.abs();

    // Timoshenko deflection must exceed EB deflection
    assert!(
        tip_timo > tip_eb,
        "Deep cantilever: Timoshenko ({:.6e}) should exceed EB ({:.6e})",
        tip_timo, tip_eb
    );

    // Check EB result against analytical
    let eb_error = (tip_eb - delta_eb).abs() / delta_eb;
    assert!(
        eb_error < 0.02,
        "EB deflection: computed={:.6e}, analytical={:.6e}, error={:.2}%",
        tip_eb, delta_eb, eb_error * 100.0
    );

    // Check Timoshenko result against analytical
    let timo_error = (tip_timo - delta_timo).abs() / delta_timo;
    assert!(
        timo_error < 0.02,
        "Timo deflection: computed={:.6e}, analytical={:.6e}, error={:.2}%",
        tip_timo, delta_timo, timo_error * 100.0
    );

    // The shear contribution should be significant for L/d = 2
    let shear_ratio = delta_shear / delta_eb;
    assert!(
        shear_ratio > 0.10,
        "Shear contribution should be >10% for L/d=2: ratio={:.2}%",
        shear_ratio * 100.0
    );
}

// ================================================================
// 2. Simply-Supported Deep Beam (L/d = 3): Midspan deflection
// ================================================================
//
// SS beam with UDL q:
//   EB midspan deflection:   delta = 5*q*L^4 / (384*EI)
//   Timoshenko adds shear:   delta_T = delta_EB * (1 + beta)
//   where beta = 48*EI/(5*kAG*L^2) for UDL on SS beam
//
// For deep beam, beta is significant.

#[test]
fn ss_deep_beam_l_over_d_3() {
    let l = 3.0 * D_RECT; // 0.9m
    let q: f64 = -50.0;    // kN/m downward
    let n = 10;

    let e_kn = E * 1000.0;
    let g_kn = G * 1000.0;

    // Analytical EB midspan deflection
    let delta_eb = 5.0 * q.abs() * l.powi(4) / (384.0 * e_kn * IZ_RECT);

    // Shear correction factor for SS beam with UDL
    let beta = 48.0 * e_kn * IZ_RECT / (5.0 * AS_Y_RECT * g_kn * l * l);
    let delta_timo = delta_eb * (1.0 + beta);

    // Run solver without asY (EB)
    let input_eb = make_ss_beam_udl_timo(n, l, A_RECT, IZ_RECT, None, q);
    let results_eb = linear::solve_2d(&input_eb).unwrap();
    let mid_node = n / 2 + 1;
    let mid_eb = results_eb.displacements.iter()
        .find(|d| d.node_id == mid_node).unwrap().uy.abs();

    // Run solver with asY (Timoshenko)
    let input_timo = make_ss_beam_udl_timo(n, l, A_RECT, IZ_RECT, Some(AS_Y_RECT), q);
    let results_timo = linear::solve_2d(&input_timo).unwrap();
    let mid_timo = results_timo.displacements.iter()
        .find(|d| d.node_id == mid_node).unwrap().uy.abs();

    // Timoshenko must exceed EB
    assert!(
        mid_timo > mid_eb,
        "SS deep beam: Timoshenko ({:.6e}) should exceed EB ({:.6e})",
        mid_timo, mid_eb
    );

    // Check against analytical EB
    let eb_error = (mid_eb - delta_eb).abs() / delta_eb;
    assert!(
        eb_error < 0.02,
        "SS EB deflection: computed={:.6e}, analytical={:.6e}, error={:.2}%",
        mid_eb, delta_eb, eb_error * 100.0
    );

    // Check that Timoshenko deflection is within tolerance of analytical
    let timo_error = (mid_timo - delta_timo).abs() / delta_timo;
    assert!(
        timo_error < 0.05,
        "SS Timo deflection: computed={:.6e}, analytical={:.6e}, error={:.2}%",
        mid_timo, delta_timo, timo_error * 100.0
    );
}

// ================================================================
// 3. Slender Beam Recovery (L/d = 20): Timoshenko ~ EB
// ================================================================
//
// For L/d = 20, phi is very small and shear deformation is negligible.
// Both theories should agree within 1%.

#[test]
fn slender_beam_l_over_d_20() {
    let l = 20.0 * D_RECT; // 6.0m
    let p = 50.0;           // kN
    let n = 10;

    // Run solver without asY (EB)
    let input_eb = make_cantilever_tip_load(n, l, A_RECT, IZ_RECT, None, p);
    let results_eb = linear::solve_2d(&input_eb).unwrap();
    let tip_eb = results_eb.displacements.iter()
        .find(|d| d.node_id == n + 1).unwrap().uy.abs();

    // Run solver with asY (Timoshenko)
    let input_timo = make_cantilever_tip_load(n, l, A_RECT, IZ_RECT, Some(AS_Y_RECT), p);
    let results_timo = linear::solve_2d(&input_timo).unwrap();
    let tip_timo = results_timo.displacements.iter()
        .find(|d| d.node_id == n + 1).unwrap().uy.abs();

    // They should be very close — within 1%
    let rel_diff = (tip_timo - tip_eb).abs() / tip_eb;
    assert!(
        rel_diff < 0.01,
        "Slender beam (L/d=20): EB={:.6e}, Timo={:.6e}, diff={:.4}%",
        tip_eb, tip_timo, rel_diff * 100.0
    );

    // Verify both are still reasonable (analytical EB check)
    let e_kn = E * 1000.0;
    let delta_analytical = p * l.powi(3) / (3.0 * e_kn * IZ_RECT);
    let error = (tip_eb - delta_analytical).abs() / delta_analytical;
    assert!(
        error < 0.02,
        "Slender cantilever EB vs analytical: error={:.4}%",
        error * 100.0
    );
}

// ================================================================
// 4. Fixed-Fixed Deep Beam (L/d = 3): Midspan deflection
// ================================================================
//
// Fixed-fixed beam with central point load P:
//   EB:   delta = PL^3 / (192*EI)
//   Timo: delta = PL^3 / (192*EI) + PL / (4*kAG)
//         = PL^3/(192*EI) * (1 + 48*EI/(kAG*L^2))
//         = PL^3/(192*EI) * (1 + 4*phi)
//
// For deep beam, the shear term 4*phi is large.

#[test]
fn fixed_fixed_deep_beam_l_over_d_3() {
    let l = 3.0 * D_RECT; // 0.9m
    let p = 100.0;         // kN
    let n = 10;

    let e_kn = E * 1000.0;
    let g_kn = G * 1000.0;

    // Analytical Euler-Bernoulli: central point load on fixed-fixed
    let delta_eb = p * l.powi(3) / (192.0 * e_kn * IZ_RECT);

    // Timoshenko correction
    let phi = 12.0 * e_kn * IZ_RECT / (g_kn * AS_Y_RECT * l * l);
    let delta_timo = delta_eb * (1.0 + 4.0 * phi);

    // Run solver without asY (EB)
    let input_eb = make_fixed_fixed_midspan_load(n, l, A_RECT, IZ_RECT, None, p);
    let results_eb = linear::solve_2d(&input_eb).unwrap();
    let mid_node = n / 2 + 1;
    let mid_eb = results_eb.displacements.iter()
        .find(|d| d.node_id == mid_node).unwrap().uy.abs();

    // Run solver with asY (Timoshenko)
    let input_timo = make_fixed_fixed_midspan_load(n, l, A_RECT, IZ_RECT, Some(AS_Y_RECT), p);
    let results_timo = linear::solve_2d(&input_timo).unwrap();
    let mid_timo = results_timo.displacements.iter()
        .find(|d| d.node_id == mid_node).unwrap().uy.abs();

    // Timoshenko deflection must exceed EB
    assert!(
        mid_timo > mid_eb,
        "Fixed-fixed deep beam: Timo ({:.6e}) should exceed EB ({:.6e})",
        mid_timo, mid_eb
    );

    // Check EB result
    let eb_error = (mid_eb - delta_eb).abs() / delta_eb;
    assert!(
        eb_error < 0.02,
        "Fixed-fixed EB: computed={:.6e}, analytical={:.6e}, error={:.2}%",
        mid_eb, delta_eb, eb_error * 100.0
    );

    // Check Timoshenko result
    let timo_error = (mid_timo - delta_timo).abs() / delta_timo;
    assert!(
        timo_error < 0.05,
        "Fixed-fixed Timo: computed={:.6e}, analytical={:.6e}, error={:.2}%",
        mid_timo, delta_timo, timo_error * 100.0
    );

    // The shear contribution should be very significant for fixed-fixed
    let shear_amplification = 4.0 * phi;
    assert!(
        shear_amplification > 0.5,
        "Shear amplification 4*phi={:.3} should be >0.5 for L/d=3 fixed-fixed",
        shear_amplification
    );
}

// ================================================================
// 5. Cantilever with Point Load: Tip deflection comparison
// ================================================================
//
// L/d = 5 (moderate depth). Direct comparison of solver outputs.
// Analytical: delta_EB = PL^3/(3EI), delta_shear = PL/(kAG).

#[test]
fn cantilever_point_load_comparison() {
    let l = 5.0 * D_RECT; // 1.5m
    let p = 80.0;          // kN
    let n = 10;

    let e_kn = E * 1000.0;
    let g_kn = G * 1000.0;

    let delta_eb_analytical = p * l.powi(3) / (3.0 * e_kn * IZ_RECT);
    let delta_shear_analytical = p * l / (AS_Y_RECT * g_kn);

    // Run EB
    let input_eb = make_cantilever_tip_load(n, l, A_RECT, IZ_RECT, None, p);
    let results_eb = linear::solve_2d(&input_eb).unwrap();
    let tip_eb = results_eb.displacements.iter()
        .find(|d| d.node_id == n + 1).unwrap().uy.abs();

    // Run Timoshenko
    let input_timo = make_cantilever_tip_load(n, l, A_RECT, IZ_RECT, Some(AS_Y_RECT), p);
    let results_timo = linear::solve_2d(&input_timo).unwrap();
    let tip_timo = results_timo.displacements.iter()
        .find(|d| d.node_id == n + 1).unwrap().uy.abs();

    // Check that the solver's difference equals the analytical shear deflection
    let solver_diff = tip_timo - tip_eb;
    assert!(
        solver_diff > 0.0,
        "Timoshenko deflection must exceed EB"
    );

    // The extra deflection from the solver should match the analytical shear term
    let diff_error = (solver_diff - delta_shear_analytical).abs() / delta_shear_analytical;
    assert!(
        diff_error < 0.05,
        "Shear increment: solver_diff={:.6e}, analytical={:.6e}, error={:.2}%",
        solver_diff, delta_shear_analytical, diff_error * 100.0
    );

    // Also verify the EB deflection itself
    let eb_error = (tip_eb - delta_eb_analytical).abs() / delta_eb_analytical;
    assert!(
        eb_error < 0.02,
        "EB tip deflection: computed={:.6e}, analytical={:.6e}, error={:.2}%",
        tip_eb, delta_eb_analytical, eb_error * 100.0
    );
}

// ================================================================
// 6. Propped Cantilever Deep Beam: Reaction distribution changes
// ================================================================
//
// Propped cantilever (fixed + roller) with UDL.
// EB reactions: R_roller = 3qL/8, R_fixed = 5qL/8.
// With Timoshenko, shear deformation modifies the stiffness distribution
// and thus the reactions change. For deep beams, the change is measurable.

#[test]
fn propped_cantilever_deep_beam() {
    let l = 3.0 * D_RECT; // 0.9m, L/d = 3
    let q: f64 = -40.0;    // kN/m downward
    let n = 10;

    let total_load = q.abs() * l;

    // EB analytical reactions
    let r_roller_eb_analytical = 3.0 * q.abs() * l / 8.0;
    let r_fixed_eb_analytical = 5.0 * q.abs() * l / 8.0;

    // Run EB solver
    let input_eb = make_propped_cantilever_udl(n, l, A_RECT, IZ_RECT, None, q);
    let results_eb = linear::solve_2d(&input_eb).unwrap();

    let mut reactions_eb = results_eb.reactions.clone();
    reactions_eb.sort_by_key(|r| r.node_id);
    let r_fixed_eb = reactions_eb.iter().find(|r| r.node_id == 1).unwrap().ry;
    let r_roller_eb = reactions_eb.iter().find(|r| r.node_id == n + 1).unwrap().ry;

    // Verify EB reactions match analytical
    assert_close(r_roller_eb, r_roller_eb_analytical, 0.01, "EB roller reaction");
    assert_close(r_fixed_eb, r_fixed_eb_analytical, 0.01, "EB fixed reaction");

    // Run Timoshenko solver
    let input_timo = make_propped_cantilever_udl(n, l, A_RECT, IZ_RECT, Some(AS_Y_RECT), q);
    let results_timo = linear::solve_2d(&input_timo).unwrap();

    let mut reactions_timo = results_timo.reactions.clone();
    reactions_timo.sort_by_key(|r| r.node_id);
    let _r_fixed_timo = reactions_timo.iter().find(|r| r.node_id == 1).unwrap().ry;
    let r_roller_timo = reactions_timo.iter().find(|r| r.node_id == n + 1).unwrap().ry;

    // Equilibrium must hold for both
    let sum_eb: f64 = reactions_eb.iter().map(|r| r.ry).sum();
    let sum_timo: f64 = reactions_timo.iter().map(|r| r.ry).sum();
    assert_close(sum_eb, total_load, 1e-4, "EB equilibrium");
    assert_close(sum_timo, total_load, 1e-4, "Timo equilibrium");

    // Reactions should differ between EB and Timoshenko for deep beam
    // The propped cantilever is indeterminate, so shear deformation
    // redistributes the reactions.
    let roller_diff = (r_roller_timo - r_roller_eb).abs();
    assert!(
        roller_diff > 0.01,
        "Propped cantilever: reactions should differ. EB roller={:.4}, Timo roller={:.4}",
        r_roller_eb, r_roller_timo
    );
}

// ================================================================
// 7. 3D Deep Beam: Timoshenko with as_y and as_z
// ================================================================
//
// 3D cantilever beam along X-axis, tip load in Y direction.
// Verifies the 3D solver produces larger deflection with Timoshenko.

#[test]
fn deep_beam_3d() {
    let l = 2.0 * D_RECT; // 0.6m, L/d = 2
    let p = 100.0;         // kN in Y direction
    let n = 8;

    let e_kn = E * 1000.0;
    let g_kn = G * 1000.0;

    // For 3D, need Iy and J in addition to Iz
    let iy = D_RECT * B_RECT.powi(3) / 12.0; // weak axis
    // Approximate J for rectangular: simplified Saint-Venant formula
    // Use simpler formula: J = a*b^3/3 * (1 - 0.63*a/b) where a < b
    let a_dim = B_RECT.min(D_RECT);  // shorter dimension
    let b_dim = B_RECT.max(D_RECT);  // longer dimension
    let j_rect = a_dim * b_dim.powi(3) / 3.0 * (1.0 - 0.63 * a_dim / b_dim);

    // Analytical EB deflection (bending about Z-axis for Y load)
    let delta_eb_analytical = p * l.powi(3) / (3.0 * e_kn * IZ_RECT);
    let delta_shear = p * l / (AS_Y_RECT * g_kn);

    // EB solver (no shear areas)
    let n_nodes = n + 1;
    let elem_len = l / n as f64;
    let nodes: Vec<_> = (0..n_nodes).map(|i| (i + 1, i as f64 * elem_len, 0.0, 0.0)).collect();
    let elems: Vec<_> = (0..n).map(|i| (i + 1, "frame", i + 1, i + 2, 1, 1)).collect();

    // Fixed support: all 6 DOFs restrained
    let fixed_dofs = vec![true, true, true, true, true, true];
    let sups_eb = vec![(1_usize, fixed_dofs.clone())];

    let loads = vec![SolverLoad3D::Nodal(SolverNodalLoad3D {
        node_id: n_nodes, fx: 0.0, fy: -p, fz: 0.0, mx: 0.0, my: 0.0, mz: 0.0, bw: None,
    })];

    // EB: no as_y, no as_z
    let mut input_eb = make_3d_input(
        nodes.clone(), vec![(1, E, NU)], vec![(1, A_RECT, iy, IZ_RECT, j_rect)],
        elems.clone(), sups_eb, loads.clone(),
    );

    let results_eb = linear::solve_3d(&input_eb).unwrap();
    let tip_eb = results_eb.displacements.iter()
        .find(|d| d.node_id == n_nodes).unwrap().uy.abs();

    // Timoshenko: set as_y and as_z in the section
    let as_z = KAPPA * A_RECT; // same for rectangular section
    for sec in input_eb.sections.values_mut() {
        sec.as_y = Some(AS_Y_RECT);
        sec.as_z = Some(as_z);
    }
    // Need to rebuild sups since make_3d_input consumed sups
    let sups_timo = vec![(1_usize, fixed_dofs.clone())];
    let input_timo = make_3d_input(
        nodes.clone(), vec![(1, E, NU)], vec![(1, A_RECT, iy, IZ_RECT, j_rect)],
        elems.clone(), sups_timo, loads.clone(),
    );
    // Manually set as_y/as_z
    let mut input_timo = input_timo;
    for sec in input_timo.sections.values_mut() {
        sec.as_y = Some(AS_Y_RECT);
        sec.as_z = Some(as_z);
    }

    let results_timo = linear::solve_3d(&input_timo).unwrap();
    let tip_timo = results_timo.displacements.iter()
        .find(|d| d.node_id == n_nodes).unwrap().uy.abs();

    // Timoshenko must exceed EB
    assert!(
        tip_timo > tip_eb,
        "3D deep beam: Timo ({:.6e}) should exceed EB ({:.6e})",
        tip_timo, tip_eb
    );

    // Check EB result is reasonable
    let eb_error = (tip_eb - delta_eb_analytical).abs() / delta_eb_analytical;
    assert!(
        eb_error < 0.05,
        "3D EB deflection: computed={:.6e}, analytical={:.6e}, error={:.2}%",
        tip_eb, delta_eb_analytical, eb_error * 100.0
    );

    // The extra deflection should approximately match shear analytical
    let diff = tip_timo - tip_eb;
    assert!(
        diff > delta_shear * 0.5,
        "3D shear increment ({:.6e}) should be at least 50% of analytical ({:.6e})",
        diff, delta_shear
    );
}

// ================================================================
// 8. Deep I-Beam: Realistic I-section where A_web << A_total
// ================================================================
//
// I-section: the shear area is approximately the web area (d_web * t_web),
// which is much smaller than the total area. This amplifies Timoshenko effects.
//
// Section: HEB 300-like (simplified)
//   Total A = 0.0149 m^2, Iz = 2.517e-4 m^4
//   Web:  d_web=0.260m, t_web=0.011m => A_web=0.00286 m^2
//   Shear area As = A_web = 0.00286 m^2 (much less than A_total)

#[test]
fn deep_i_beam() {
    let a_total = 0.0149;          // m^2 (HEB 300 approximate)
    let iz_i = 2.517e-4;           // m^4
    let d_beam = 0.300;            // m depth
    let a_web = 0.260 * 0.011;     // 0.00286 m^2
    let as_y_i = a_web;            // shear area ~ web area for I-beams

    let l: f64 = 3.0 * d_beam; // 0.9m, L/d = 3
    let p = 100.0;
    let n = 10;

    let e_kn = E * 1000.0;
    let g_kn = G * 1000.0;

    // Analytical
    let delta_eb = p * l.powi(3) / (3.0 * e_kn * iz_i);
    let delta_shear = p * l / (as_y_i * g_kn);
    let shear_ratio = delta_shear / delta_eb;

    // Shear/bending ratio should be very large for I-beam (small As)
    assert!(
        shear_ratio > 0.3,
        "I-beam shear ratio should be large: {:.2}%",
        shear_ratio * 100.0
    );

    // Run EB solver
    let input_eb = make_cantilever_tip_load(n, l, a_total, iz_i, None, p);
    let results_eb = linear::solve_2d(&input_eb).unwrap();
    let tip_eb = results_eb.displacements.iter()
        .find(|d| d.node_id == n + 1).unwrap().uy.abs();

    // Run Timoshenko solver
    let input_timo = make_cantilever_tip_load(n, l, a_total, iz_i, Some(as_y_i), p);
    let results_timo = linear::solve_2d(&input_timo).unwrap();
    let tip_timo = results_timo.displacements.iter()
        .find(|d| d.node_id == n + 1).unwrap().uy.abs();

    // Timoshenko deflection must be significantly larger than EB
    let amplification = tip_timo / tip_eb;
    assert!(
        amplification > 1.3,
        "I-beam: Timoshenko amplification ({:.3}x) should be >1.3x for deep I-beam",
        amplification
    );

    // Check EB accuracy
    let eb_error = (tip_eb - delta_eb).abs() / delta_eb;
    assert!(
        eb_error < 0.02,
        "I-beam EB: computed={:.6e}, analytical={:.6e}, error={:.2}%",
        tip_eb, delta_eb, eb_error * 100.0
    );

    // Check that the combined Timoshenko deflection is accurate
    let delta_timo = delta_eb + delta_shear;
    let timo_error = (tip_timo - delta_timo).abs() / delta_timo;
    assert!(
        timo_error < 0.02,
        "I-beam Timo: computed={:.6e}, analytical={:.6e}, error={:.2}%",
        tip_timo, delta_timo, timo_error * 100.0
    );
}
