/// Validation: Soil-Structure Interaction (Pure Formula Verification)
///
/// References:
///   - Hetenyi, "Beams on Elastic Foundation", University of Michigan Press, 1946
///   - Vesic, "Bending of Beams Resting on Isotropic Elastic Solid", ASCE, 1961
///   - Biot, "Bending of an Infinite Beam on an Elastic Foundation", ASME, 1937
///   - Terzaghi, "Evaluation of Coefficients of Subgrade Reaction", Geotechnique, 1955
///   - Bowles, "Foundation Analysis and Design", 5th Ed., McGraw-Hill
///   - Das, "Principles of Foundation Engineering", 9th Ed.
///   - Meyerhof, "The Ultimate Bearing Capacity of Foundations", Geotechnique, 1951
///
/// Tests verify soil-structure interaction formulas without calling the solver.
///
/// Tests:
///   1. Winkler spring modulus and subgrade reaction
///   2. Vesic formula for subgrade reaction modulus
///   3. Biot's relative stiffness parameter
///   4. Beam on elastic foundation: characteristic length
///   5. Lateral earth pressure: Rankine active and passive
///   6. Lateral earth pressure: Coulomb with wall friction
///   7. Bearing capacity: Terzaghi and Meyerhof
///   8. Settlement of footings on elastic half-space

use std::f64::consts::PI;

// ================================================================
// 1. Winkler Spring Modulus and Subgrade Reaction
// ================================================================
//
// The Winkler model represents soil as independent linear springs.
// Subgrade reaction modulus k_s (kN/m^3) relates contact pressure
// to settlement:
//   q = k_s * delta
//
// For a beam of width B:
//   Spring stiffness per unit length: k = k_s * B  (kN/m per m)
//
// Terzaghi (1955) suggested values:
//   Loose sand:    k_s = 4800-16000 kN/m^3
//   Medium sand:   k_s = 9600-80000 kN/m^3
//   Dense sand:    k_s = 64000-128000 kN/m^3
//   Stiff clay:    k_s = 12000-24000 kN/m^3
//
// Reference: Bowles, "Foundation Analysis and Design", Table 9-1

#[test]
fn validation_ssi_winkler_spring_modulus() {
    // Contact pressure and settlement relationship
    let ks_medium_sand: f64 = 40_000.0; // kN/m^3
    let delta: f64 = 0.010; // m, 10 mm settlement
    let q: f64 = ks_medium_sand * delta;
    let q_expected: f64 = 400.0; // kPa
    assert!(
        (q - q_expected).abs() / q_expected < 1e-12,
        "Contact pressure: computed={:.2} kPa, expected={:.2} kPa",
        q, q_expected
    );

    // Spring stiffness per unit length for beam width B
    let b: f64 = 0.6; // m, beam width
    let k_per_m: f64 = ks_medium_sand * b;
    let k_per_m_expected: f64 = 24_000.0; // kN/m per m
    assert!(
        (k_per_m - k_per_m_expected).abs() / k_per_m_expected < 1e-12,
        "Spring stiffness: computed={:.0}, expected={:.0} kN/m per m",
        k_per_m, k_per_m_expected
    );

    // FEM nodal spring: tributary length * k_per_m
    let elem_len: f64 = 0.5; // m
    let k_nodal_interior: f64 = k_per_m * elem_len;
    let k_nodal_interior_expected: f64 = 12_000.0; // kN/m
    assert!(
        (k_nodal_interior - k_nodal_interior_expected).abs() / k_nodal_interior_expected < 1e-12,
        "Nodal spring: computed={:.0}, expected={:.0} kN/m",
        k_nodal_interior, k_nodal_interior_expected
    );

    // End node gets half the tributary length
    let k_nodal_end: f64 = k_per_m * elem_len / 2.0;
    assert!(
        (k_nodal_end - k_nodal_interior / 2.0).abs() < 1e-10,
        "End spring is half of interior: {:.0} vs {:.0}",
        k_nodal_end, k_nodal_interior / 2.0
    );
}

// ================================================================
// 2. Vesic Formula for Subgrade Reaction Modulus
// ================================================================
//
// Vesic (1961) proposed a formula relating the subgrade modulus
// to the soil elastic modulus and beam stiffness:
//
//   k_s = 0.65 * (E_s * B^4 / (E_b * I_b))^(1/12) * E_s / (1 - nu_s^2)
//
// Simplified form (for practical use):
//   k_s ≈ E_s / (B * (1 - nu_s^2))
//
// where:
//   E_s = soil elastic modulus
//   nu_s = soil Poisson's ratio
//   B = foundation width
//   E_b = beam elastic modulus
//   I_b = beam moment of inertia
//
// Reference: Vesic, ASCE J. Soil Mech., 1961

#[test]
fn validation_ssi_vesic_subgrade_modulus() {
    let e_s: f64 = 50_000.0;  // kPa, soil elastic modulus (medium dense sand)
    let nu_s: f64 = 0.30;     // Poisson's ratio
    let b: f64 = 1.0;         // m, foundation width

    // Simplified Vesic formula
    let ks_simple: f64 = e_s / (b * (1.0 - nu_s * nu_s));
    let ks_simple_expected: f64 = 50_000.0 / (1.0 * 0.91); // ≈ 54945 kN/m^3
    assert!(
        (ks_simple - ks_simple_expected).abs() / ks_simple_expected < 1e-10,
        "Vesic simplified: computed={:.0}, expected={:.0} kN/m^3",
        ks_simple, ks_simple_expected
    );

    // Full Vesic formula
    let e_b: f64 = 200_000_000.0; // kPa = 200 GPa (steel)
    let i_b: f64 = 1e-4;          // m^4
    let ratio: f64 = (e_s * b.powi(4) / (e_b * i_b)).powf(1.0 / 12.0);
    let ks_full: f64 = 0.65 * ratio * e_s / (1.0 - nu_s * nu_s);

    // Full formula should be positive
    assert!(
        ks_full > 0.0,
        "Vesic full: ks = {:.0} kN/m^3",
        ks_full
    );

    // Wider foundation: k_s should decrease (inversely related to B)
    let b2: f64 = 2.0; // double the width
    let ks_wider: f64 = e_s / (b2 * (1.0 - nu_s * nu_s));
    assert!(
        ks_wider < ks_simple,
        "Wider foundation: ks({:.1}m)={:.0} < ks({:.1}m)={:.0}",
        b2, ks_wider, b, ks_simple
    );

    // Ratio should be B1/B2
    let ks_ratio: f64 = ks_wider / ks_simple;
    let ks_ratio_expected: f64 = b / b2; // = 0.5
    assert!(
        (ks_ratio - ks_ratio_expected).abs() < 1e-10,
        "Width ratio: computed={:.4}, expected={:.4}",
        ks_ratio, ks_ratio_expected
    );
}

// ================================================================
// 3. Biot's Relative Stiffness Parameter
// ================================================================
//
// Biot (1937) defined a relative stiffness parameter that
// characterizes the beam-soil interaction:
//
//   K_r = E_b * I_b / (E_s * L^3)
//
// where L = beam length.
//
// For K_r > 0.5: beam is "rigid" (uniform pressure distribution)
// For K_r < 0.01: beam is "flexible" (pressure varies significantly)
//
// The characteristic length of the elastic foundation:
//   lambda = (k_s * B / (4 * E_b * I_b))^(1/4)
//
// Reference: Biot, J. Applied Mechanics, ASME, 1937

#[test]
fn validation_ssi_biot_relative_stiffness() {
    let e_b: f64 = 30_000_000.0; // kPa = 30 GPa (concrete)
    let i_b: f64 = 4.5e-3;       // m^4 (typical strip footing 0.3m x 1.0m)
    let e_s: f64 = 20_000.0;     // kPa (soft soil)
    let l: f64 = 10.0;           // m, beam length

    // Biot's relative stiffness
    let k_r: f64 = e_b * i_b / (e_s * l * l * l);
    // = 30e6 * 4.5e-3 / (20000 * 1000)
    // = 135000 / 20000000 = 0.00675
    let k_r_expected: f64 = 30_000_000.0 * 4.5e-3 / (20_000.0 * 1000.0);
    assert!(
        (k_r - k_r_expected).abs() / k_r_expected < 1e-10,
        "Biot K_r: computed={:.6}, expected={:.6}",
        k_r, k_r_expected
    );

    // K_r < 0.01 => flexible beam
    assert!(
        k_r < 0.01,
        "Flexible beam: K_r={:.6} < 0.01",
        k_r
    );

    // For a much stiffer beam (thicker section)
    let i_b_stiff: f64 = 0.1; // m^4 (very stiff)
    let k_r_stiff: f64 = e_b * i_b_stiff / (e_s * l * l * l);
    // = 30e6 * 0.1 / 20e6 = 3e6/20e6 = 0.15
    assert!(
        k_r_stiff > 0.1,
        "Rigid beam: K_r={:.4} > 0.1",
        k_r_stiff
    );

    // Verify: stiffer beam always has higher K_r
    assert!(
        k_r_stiff > k_r,
        "Stiffer beam: K_r={:.6} > {:.6}",
        k_r_stiff, k_r
    );

    // Softer soil also increases K_r (beam relatively more rigid)
    let e_s_soft: f64 = 5000.0; // kPa
    let k_r_soft_soil: f64 = e_b * i_b / (e_s_soft * l * l * l);
    assert!(
        k_r_soft_soil > k_r,
        "Softer soil: K_r={:.6} > {:.6}",
        k_r_soft_soil, k_r
    );
}

// ================================================================
// 4. Beam on Elastic Foundation: Characteristic Length
// ================================================================
//
// The characteristic length (or relative stiffness factor) for
// a beam on a Winkler foundation:
//
//   lambda = (k * B / (4 * EI))^(1/4)
//
// Key quantities:
//   Characteristic length: L_c = pi / lambda
//   Deflection under point load P at x=0:
//     y(0) = P * lambda / (2 * k * B)
//   Maximum bending moment:
//     M(0) = P / (4 * lambda)
//
// If beam length L > L_c: "long beam" (infinite beam solution applies)
// If beam length L < L_c: "short beam" (finite beam corrections needed)
//
// Reference: Hetenyi, "Beams on Elastic Foundation", Ch. 3

#[test]
fn validation_ssi_characteristic_length() {
    let e_b: f64 = 200_000_000.0; // kPa = 200 GPa
    let i_b: f64 = 8.33e-5;       // m^4 (HEB200 approx)
    let ks: f64 = 30_000.0;       // kN/m^3
    let b: f64 = 0.2;             // m, flange width

    // Characteristic parameter lambda
    let lambda: f64 = (ks * b / (4.0 * e_b * i_b)).powf(0.25);
    let lambda_expected: f64 = (30_000.0_f64 * 0.2 / (4.0 * 200_000_000.0 * 8.33e-5)).powf(0.25);
    assert!(
        (lambda - lambda_expected).abs() / lambda_expected < 1e-10,
        "Lambda: computed={:.6}, expected={:.6}",
        lambda, lambda_expected
    );

    // Characteristic length
    let l_c: f64 = PI / lambda;
    assert!(
        l_c > 0.0,
        "Characteristic length: {:.4} m",
        l_c
    );

    // Deflection under point load
    let p: f64 = 100.0; // kN
    let y_0: f64 = p * lambda / (2.0 * ks * b);
    assert!(
        y_0 > 0.0,
        "Deflection at load: {:.6} m",
        y_0
    );

    // Maximum bending moment
    let m_0: f64 = p / (4.0 * lambda);
    assert!(
        m_0 > 0.0,
        "Maximum moment at load: {:.4} kN-m",
        m_0
    );

    // Verify relationship: doubling EI reduces lambda by factor 2^(-1/4)
    let i_b2: f64 = 2.0 * i_b;
    let lambda2: f64 = (ks * b / (4.0 * e_b * i_b2)).powf(0.25);
    let ratio: f64 = lambda2 / lambda;
    let ratio_expected: f64 = 2.0_f64.powf(-0.25);
    assert!(
        (ratio - ratio_expected).abs() / ratio_expected < 1e-10,
        "Lambda ratio for 2x EI: computed={:.6}, expected={:.6}",
        ratio, ratio_expected
    );

    // Stiffer foundation (larger ks) increases lambda
    let ks2: f64 = 2.0 * ks;
    let lambda_stiffer: f64 = (ks2 * b / (4.0 * e_b * i_b)).powf(0.25);
    assert!(
        lambda_stiffer > lambda,
        "Stiffer soil: lambda={:.6} > {:.6}",
        lambda_stiffer, lambda
    );
}

// ================================================================
// 5. Lateral Earth Pressure: Rankine Active and Passive
// ================================================================
//
// Rankine earth pressure coefficients:
//   K_a = (1 - sin(phi)) / (1 + sin(phi)) = tan^2(45 - phi/2)
//   K_p = (1 + sin(phi)) / (1 - sin(phi)) = tan^2(45 + phi/2)
//
// where phi = soil friction angle.
//
// Active pressure at depth z: sigma_a = K_a * gamma * z - 2*c*sqrt(K_a)
// Passive pressure at depth z: sigma_p = K_p * gamma * z + 2*c*sqrt(K_p)
//
// Note: K_a * K_p = 1
//
// Reference: Das, "Principles of Foundation Engineering", Ch. 7

#[test]
fn validation_ssi_rankine_earth_pressure() {
    let phi_deg: f64 = 30.0; // degrees, friction angle
    let phi_rad: f64 = phi_deg * PI / 180.0;
    let gamma: f64 = 18.0;   // kN/m^3, unit weight
    let c: f64 = 0.0;        // kPa, cohesion (granular soil)
    let z: f64 = 5.0;        // m, depth

    // Rankine active coefficient
    let ka: f64 = (1.0 - phi_rad.sin()) / (1.0 + phi_rad.sin());
    let ka_expected: f64 = (45.0_f64 - phi_deg / 2.0).to_radians().tan().powi(2);
    assert!(
        (ka - ka_expected).abs() / ka_expected < 1e-10,
        "K_a: computed={:.6}, expected={:.6}",
        ka, ka_expected
    );

    // For phi=30: K_a = 1/3
    let ka_30: f64 = 1.0 / 3.0;
    assert!(
        (ka - ka_30).abs() / ka_30 < 1e-10,
        "K_a(30) = 1/3: computed={:.6}",
        ka
    );

    // Rankine passive coefficient
    let kp: f64 = (1.0 + phi_rad.sin()) / (1.0 - phi_rad.sin());
    let kp_expected: f64 = 3.0; // for phi=30
    assert!(
        (kp - kp_expected).abs() / kp_expected < 1e-10,
        "K_p: computed={:.6}, expected={:.6}",
        kp, kp_expected
    );

    // K_a * K_p = 1
    let product: f64 = ka * kp;
    assert!(
        (product - 1.0).abs() < 1e-10,
        "K_a * K_p = {:.6} (should be 1.0)",
        product
    );

    // Active pressure at depth z
    let sigma_a: f64 = ka * gamma * z - 2.0 * c * ka.sqrt();
    let sigma_a_expected: f64 = (1.0 / 3.0) * 18.0 * 5.0; // = 30 kPa
    assert!(
        (sigma_a - sigma_a_expected).abs() / sigma_a_expected < 1e-10,
        "Active pressure: computed={:.2} kPa, expected={:.2} kPa",
        sigma_a, sigma_a_expected
    );

    // Passive pressure at depth z
    let sigma_p: f64 = kp * gamma * z + 2.0 * c * kp.sqrt();
    let sigma_p_expected: f64 = 3.0 * 18.0 * 5.0; // = 270 kPa
    assert!(
        (sigma_p - sigma_p_expected).abs() / sigma_p_expected < 1e-10,
        "Passive pressure: computed={:.2} kPa, expected={:.2} kPa",
        sigma_p, sigma_p_expected
    );

    // Passive > Active
    assert!(
        sigma_p > sigma_a,
        "Passive ({:.2}) > Active ({:.2})",
        sigma_p, sigma_a
    );
}

// ================================================================
// 6. Lateral Earth Pressure: Coulomb with Wall Friction
// ================================================================
//
// Coulomb's active earth pressure coefficient with wall friction:
//   K_a = sin^2(alpha + phi) /
//         [sin^2(alpha) * sin(alpha - delta) *
//          (1 + sqrt(sin(phi + delta)*sin(phi - beta) /
//                    (sin(alpha - delta)*sin(alpha + beta))))^2]
//
// For vertical wall (alpha=90), horizontal backfill (beta=0):
//   K_a = cos^2(phi) /
//         [1 + sqrt(sin(phi + delta)*sin(phi) / cos(delta))]^2
//
// where:
//   phi = soil friction angle
//   delta = wall friction angle
//   alpha = wall inclination from horizontal (90 for vertical)
//   beta = backfill slope
//
// Reference: Das, "Principles of Foundation Engineering", Ch. 7

#[test]
fn validation_ssi_coulomb_earth_pressure() {
    let phi_deg: f64 = 35.0;
    let phi_rad: f64 = phi_deg * PI / 180.0;
    let delta_deg: f64 = 20.0; // wall friction angle
    let delta_rad: f64 = delta_deg * PI / 180.0;
    let alpha_deg: f64 = 90.0; // vertical wall
    let alpha_rad: f64 = alpha_deg * PI / 180.0;
    let beta_deg: f64 = 0.0;   // horizontal backfill
    let beta_rad: f64 = beta_deg * PI / 180.0;

    // General Coulomb formula
    let num: f64 = (alpha_rad + phi_rad).sin().powi(2);
    let inner_sqrt: f64 = ((phi_rad + delta_rad).sin() * (phi_rad - beta_rad).sin()
        / ((alpha_rad - delta_rad).sin() * (alpha_rad + beta_rad).sin()))
    .sqrt();
    let denom: f64 =
        alpha_rad.sin().powi(2) * (alpha_rad - delta_rad).sin() * (1.0 + inner_sqrt).powi(2);
    let ka_coulomb: f64 = num / denom;

    // Should be positive and less than 1
    assert!(
        ka_coulomb > 0.0 && ka_coulomb < 1.0,
        "Coulomb K_a should be in (0, 1): {:.6}",
        ka_coulomb
    );

    // Compare with Rankine (delta = 0)
    let ka_rankine: f64 = (1.0 - phi_rad.sin()) / (1.0 + phi_rad.sin());
    // Coulomb with wall friction should give lower K_a (wall friction is favorable)
    assert!(
        ka_coulomb < ka_rankine,
        "Coulomb K_a ({:.4}) < Rankine K_a ({:.4}) with wall friction",
        ka_coulomb, ka_rankine
    );

    // Verify: when delta = 0, Coulomb reduces to Rankine for vertical wall
    let delta_zero: f64 = 0.0;
    let inner_sqrt_0: f64 = ((phi_rad + delta_zero).sin() * (phi_rad - beta_rad).sin()
        / ((alpha_rad - delta_zero).sin() * (alpha_rad + beta_rad).sin()))
    .sqrt();
    let denom_0: f64 =
        alpha_rad.sin().powi(2) * (alpha_rad - delta_zero).sin() * (1.0 + inner_sqrt_0).powi(2);
    let _ka_coulomb_0: f64 = num / denom_0;

    // For vertical wall with no friction, Coulomb = Rankine
    // The numerator changes too when delta=0
    let num_0: f64 = (alpha_rad + phi_rad).sin().powi(2);
    let ka_coulomb_exact_0: f64 = num_0 / denom_0;

    // Both should be close to Rankine
    assert!(
        (ka_coulomb_exact_0 - ka_rankine).abs() < 0.02,
        "Coulomb(delta=0)={:.4} ≈ Rankine={:.4}",
        ka_coulomb_exact_0, ka_rankine
    );
}

// ================================================================
// 7. Bearing Capacity: Terzaghi and Meyerhof
// ================================================================
//
// Terzaghi's bearing capacity for strip footing:
//   q_ult = c * N_c + q * N_q + 0.5 * gamma * B * N_gamma
//
// where N_c, N_q, N_gamma are bearing capacity factors:
//   N_q = e^(pi*tan(phi)) * tan^2(45 + phi/2)
//   N_c = (N_q - 1) * cot(phi)
//   N_gamma = 2 * (N_q + 1) * tan(phi)  (Meyerhof approximation)
//
// For phi = 30:
//   N_q = 18.40, N_c = 30.14, N_gamma = 22.40 (Meyerhof)
//
// Reference: Meyerhof (1951), Geotechnique

#[test]
fn validation_ssi_bearing_capacity() {
    let phi_deg: f64 = 30.0;
    let phi_rad: f64 = phi_deg * PI / 180.0;
    let c: f64 = 10.0;       // kPa, cohesion
    let gamma: f64 = 18.0;   // kN/m^3, unit weight
    let b: f64 = 2.0;        // m, footing width
    let d_f: f64 = 1.5;      // m, depth of footing
    let q: f64 = gamma * d_f; // kPa, overburden pressure

    // Bearing capacity factors (Meyerhof)
    let n_q: f64 = (PI * phi_rad.tan()).exp() * (45.0_f64.to_radians() + phi_rad / 2.0).tan().powi(2);
    let n_q_expected: f64 = 18.40;
    assert!(
        (n_q - n_q_expected).abs() / n_q_expected < 0.01,
        "N_q: computed={:.2}, expected={:.2}",
        n_q, n_q_expected
    );

    let n_c: f64 = (n_q - 1.0) / phi_rad.tan();
    let n_c_expected: f64 = 30.14;
    assert!(
        (n_c - n_c_expected).abs() / n_c_expected < 0.02,
        "N_c: computed={:.2}, expected={:.2}",
        n_c, n_c_expected
    );

    let n_gamma: f64 = 2.0 * (n_q + 1.0) * phi_rad.tan();
    let n_gamma_expected: f64 = 22.40;
    assert!(
        (n_gamma - n_gamma_expected).abs() / n_gamma_expected < 0.02,
        "N_gamma: computed={:.2}, expected={:.2}",
        n_gamma, n_gamma_expected
    );

    // Ultimate bearing capacity (strip footing)
    let q_ult: f64 = c * n_c + q * n_q + 0.5 * gamma * b * n_gamma;
    assert!(
        q_ult > 0.0,
        "Ultimate bearing capacity: {:.2} kPa",
        q_ult
    );

    // Each term should be positive
    let term_c: f64 = c * n_c;
    let term_q: f64 = q * n_q;
    let term_gamma: f64 = 0.5 * gamma * b * n_gamma;
    assert!(
        term_c > 0.0 && term_q > 0.0 && term_gamma > 0.0,
        "All terms positive: c={:.2}, q={:.2}, gamma={:.2}",
        term_c, term_q, term_gamma
    );

    // Factor of safety
    let fs: f64 = q_ult / q;
    assert!(
        fs > 1.0,
        "Factor of safety: {:.2} (should be > 1)",
        fs
    );
}

// ================================================================
// 8. Settlement of Footings on Elastic Half-Space
// ================================================================
//
// Immediate settlement of a rectangular footing (B x L) on an
// elastic half-space (Boussinesq theory):
//
//   s = q * B * (1 - nu^2) * I_w / E_s
//
// where:
//   q = net bearing pressure
//   B = footing width
//   nu = soil Poisson's ratio
//   E_s = soil elastic modulus
//   I_w = influence factor depending on L/B and rigidity
//
// Influence factors (Steinbrenner/Mayne-Poulos):
//   Square (L/B=1): I_w = 0.56 (flexible center), 0.88 (rigid)
//   Rectangle (L/B=2): I_w = 0.77 (flexible center), 1.12 (rigid)
//   Strip (L/B->inf): I_w -> 1.0 (flexible), 1.53 (rigid)
//
// Reference: Bowles, "Foundation Analysis and Design", Ch. 5

#[test]
fn validation_ssi_footing_settlement() {
    let q: f64 = 150.0;       // kPa, net bearing pressure
    let b: f64 = 3.0;         // m, footing width
    let nu: f64 = 0.3;        // Poisson's ratio
    let e_s: f64 = 40_000.0;  // kPa, soil modulus (medium dense sand)

    // Square rigid footing (L/B = 1)
    let iw_square: f64 = 0.88;
    let s_square: f64 = q * b * (1.0 - nu * nu) * iw_square / e_s;
    let s_square_mm: f64 = s_square * 1000.0;
    // = 150 * 3 * 0.91 * 0.88 / 40000 * 1000
    let s_square_expected_mm: f64 = 150.0 * 3.0 * 0.91 * 0.88 / 40_000.0 * 1000.0;
    assert!(
        (s_square_mm - s_square_expected_mm).abs() / s_square_expected_mm < 1e-10,
        "Square footing settlement: computed={:.2} mm, expected={:.2} mm",
        s_square_mm, s_square_expected_mm
    );

    // Rectangular footing (L/B = 2)
    let iw_rect: f64 = 1.12;
    let s_rect: f64 = q * b * (1.0 - nu * nu) * iw_rect / e_s;
    let s_rect_mm: f64 = s_rect * 1000.0;

    // Rectangular footing settles more than square (larger I_w)
    assert!(
        s_rect_mm > s_square_mm,
        "Rectangular ({:.2} mm) > Square ({:.2} mm)",
        s_rect_mm, s_square_mm
    );

    // Settlement increases linearly with footing width
    let b2: f64 = 6.0; // double the width
    let s_wider: f64 = q * b2 * (1.0 - nu * nu) * iw_square / e_s;
    let ratio: f64 = s_wider / s_square;
    let ratio_expected: f64 = b2 / b; // = 2.0
    assert!(
        (ratio - ratio_expected).abs() / ratio_expected < 1e-10,
        "Width ratio: computed={:.4}, expected={:.4}",
        ratio, ratio_expected
    );

    // Settlement decreases with increasing soil modulus
    let e_s_stiff: f64 = 80_000.0;
    let s_stiff: f64 = q * b * (1.0 - nu * nu) * iw_square / e_s_stiff;
    assert!(
        s_stiff < s_square,
        "Stiffer soil: {:.4} m < {:.4} m",
        s_stiff, s_square
    );

    // Settlement ratio equals inverse modulus ratio
    let mod_ratio: f64 = s_stiff / s_square;
    let mod_ratio_expected: f64 = e_s / e_s_stiff;
    assert!(
        (mod_ratio - mod_ratio_expected).abs() / mod_ratio_expected < 1e-10,
        "Modulus ratio: computed={:.4}, expected={:.4}",
        mod_ratio, mod_ratio_expected
    );
}
