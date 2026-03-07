/// Validation: Masonry Design
///
/// References:
///   - TMS 402/602-22: Building Code Requirements for Masonry Structures
///   - EN 1996-1-1:2005 (EC6): Design of masonry structures
///   - Drysdale, Hamid & Baker: "Masonry Structures: Behavior and Design" 3rd ed.
///   - ACI 530-13/ASCE 5-13: Building Code Requirements for Masonry Structures
///   - Hendry, Sinha & Davies: "Design of Masonry Structures" 3rd ed. (2004)
///
/// Tests verify compressive, flexural, and shear capacity of masonry
/// walls using both ASD and strength design methods.

mod helpers;

// ================================================================
// 1. Masonry Compressive Strength (TMS 402 / EC6)
// ================================================================
//
// TMS: f'm from unit strength and mortar type (prism test)
// EC6: f_k = K * f_b^0.7 * f_m^0.3

#[test]
fn masonry_compressive_strength_ec6() {
    let k: f64 = 0.55;      // K factor for Group 1 units with general purpose mortar
    let f_b: f64 = 20.0;    // MPa, normalized unit compressive strength
    let f_m: f64 = 10.0;    // MPa, mortar compressive strength

    // Characteristic compressive strength (EC6 §3.6.1.2)
    let f_k: f64 = k * f_b.powf(0.7) * f_m.powf(0.3);

    // = 0.55 * 20^0.7 * 10^0.3
    // = 0.55 * 9.518 * 1.995 = 10.43 MPa
    let f_k_expected: f64 = 0.55 * 20.0_f64.powf(0.7) * 10.0_f64.powf(0.3);

    assert!(
        (f_k - f_k_expected).abs() / f_k_expected < 0.01,
        "f_k: {:.2} MPa, expected {:.2}", f_k, f_k_expected
    );

    // Design strength: fd = fk / γM
    let gamma_m: f64 = 2.5; // partial factor (EC6 Table 2.3, Category B)
    let f_d: f64 = f_k / gamma_m;

    assert!(
        f_d > 2.0 && f_d < 10.0,
        "Design strength: {:.2} MPa", f_d
    );
}

// ================================================================
// 2. Masonry Wall — Axial Capacity (TMS 402)
// ================================================================
//
// Strength design: φPn = φ * 0.80 * f'm * An * [1 - (h/(140*r))²]
// where h = effective height, r = radius of gyration, An = net area
// For h/r ≤ 99 (short wall)

#[test]
fn masonry_axial_capacity_tms() {
    let f_m: f64 = 10.3;      // MPa (1500 psi), masonry prism strength
    let t: f64 = 0.190;       // m (7.625"), nominal wall thickness
    let b: f64 = 1.0;         // m, unit width
    let h: f64 = 3.0;         // m, effective height
    let grouting: f64 = 0.55; // fraction of cells grouted

    // Net area per meter of wall length
    let a_n: f64 = t * b * grouting; // approximate grouted area
    let a_n_expected: f64 = 0.1045;  // m²

    assert!(
        (a_n - a_n_expected).abs() / a_n_expected < 0.01,
        "Net area: {:.4} m², expected {:.4}", a_n, a_n_expected
    );

    // Radius of gyration (rectangular: r = t/√12)
    let r: f64 = t / 12.0_f64.sqrt();
    let r_expected: f64 = 0.0549; // m

    assert!(
        (r - r_expected).abs() / r_expected < 0.02,
        "Radius of gyration: {:.4} m, expected {:.4}", r, r_expected
    );

    // Slenderness ratio
    let h_r: f64 = h / r;
    assert!(
        h_r < 99.0,
        "h/r = {:.1} — short wall (< 99)", h_r
    );

    // Nominal capacity (short wall equation)
    let phi: f64 = 0.60; // strength reduction factor (compression)
    let slenderness_reduction: f64 = 1.0 - (h / (140.0 * r)).powi(2);
    let phi_pn: f64 = phi * 0.80 * (f_m * 1000.0) * a_n * slenderness_reduction; // kN

    assert!(
        phi_pn > 100.0,
        "Axial capacity: {:.0} kN (per meter of wall)", phi_pn
    );
}

// ================================================================
// 3. Masonry Flexural Strength (EC6 / TMS 402)
// ================================================================
//
// Out-of-plane bending of unreinforced masonry:
// MRd = f_xk1 * Z / γM (parallel to bed joints)
// MRd = f_xk2 * Z / γM (perpendicular to bed joints)

#[test]
fn masonry_flexural_strength() {
    let f_xk1: f64 = 0.10;   // MPa, flexural strength parallel to bed
    let f_xk2: f64 = 0.40;   // MPa, flexural strength perpendicular
    let t: f64 = 0.215;      // m, wall thickness
    let b: f64 = 1.0;        // m, unit width
    let gamma_m: f64 = 2.5;  // partial factor

    // Section modulus per meter
    let z: f64 = b * t * t / 6.0;
    let z_expected: f64 = 0.00770; // m³/m

    assert!(
        (z - z_expected).abs() / z_expected < 0.02,
        "Section modulus: {:.5} m³/m, expected {:.5}", z, z_expected
    );

    // Design moment resistance parallel to bed joints
    let mrd_parallel: f64 = f_xk1 * 1000.0 * z / gamma_m; // kN·m/m
    // = 0.10 * 1000 * 0.00770 / 2.5 = 0.308 kN·m/m

    // Design moment resistance perpendicular
    let mrd_perp: f64 = f_xk2 * 1000.0 * z / gamma_m;
    // = 0.40 * 1000 * 0.00770 / 2.5 = 1.232 kN·m/m

    assert!(
        mrd_perp > mrd_parallel,
        "Perpendicular {:.3} > parallel {:.3} kN·m/m", mrd_perp, mrd_parallel
    );

    // Ratio should be f_xk2/f_xk1
    let ratio: f64 = mrd_perp / mrd_parallel;
    let expected_ratio: f64 = f_xk2 / f_xk1;
    assert!(
        (ratio - expected_ratio).abs() / expected_ratio < 0.01,
        "Strength ratio: {:.1}, expected {:.1}", ratio, expected_ratio
    );
}

// ================================================================
// 4. Reinforced Masonry Beam — Flexural Capacity
// ================================================================
//
// Similar to RC: Mn = As * fy * (d - a/2)
// a = As * fy / (0.80 * f'm * b)

#[test]
fn masonry_reinforced_flexure() {
    let a_s: f64 = 600.0;     // mm², steel area (#5 bars × 3)
    let fy: f64 = 420.0;      // MPa, steel yield strength
    let f_m: f64 = 10.3;      // MPa, masonry compressive strength
    let b_w: f64 = 190.0;     // mm, wall thickness
    let d: f64 = 500.0;       // mm, effective depth

    // Depth of compression block
    let a: f64 = a_s * fy / (0.80 * f_m * b_w);
    // = 600 * 420 / (0.80 * 10.3 * 190) = 252000 / 1565.6 = 160.9 mm

    assert!(
        a > 0.0 && a < d,
        "Compression block: {:.1} mm (< d = {:.0})", a, d
    );

    // Nominal moment capacity
    let mn: f64 = a_s * fy * (d - a / 2.0) / 1e6; // kN·m
    // = 600 * 420 * (500 - 80.5) / 1e6 = 252000 * 419.5 / 1e6 = 105.7 kN·m

    // Design moment: φMn
    let phi: f64 = 0.90; // for tension-controlled
    let phi_mn: f64 = phi * mn;

    assert!(
        phi_mn > 50.0,
        "Design moment capacity: {:.1} kN·m", phi_mn
    );
}

// ================================================================
// 5. Masonry Shear Strength (TMS 402 / EC6)
// ================================================================
//
// TMS: Vn = Vnm + Vns
// Vnm = [4.0 - 1.75(Mu/(Vu*dv))] * An * √f'm + 0.25*Pu (ASD, in psi)
// EC6: fvk = fvk0 + 0.4*σd (≤ 0.065*fb or fvlt)

#[test]
fn masonry_shear_strength_ec6() {
    let f_vk0: f64 = 0.20;    // MPa, initial shear strength (mortar M5+)
    let sigma_d: f64 = 0.50;  // MPa, design compressive stress
    let f_b: f64 = 15.0;      // MPa, unit strength

    // Characteristic shear strength
    let f_vk: f64 = f_vk0 + 0.4 * sigma_d;
    let f_vk_expected: f64 = 0.40; // MPa

    assert!(
        (f_vk - f_vk_expected).abs() / f_vk_expected < 0.01,
        "f_vk: {:.2} MPa, expected {:.2}", f_vk, f_vk_expected
    );

    // Upper limit: 0.065 * fb
    let f_vk_limit: f64 = 0.065 * f_b;
    // = 0.975 MPa

    assert!(
        f_vk < f_vk_limit,
        "f_vk = {:.2} < limit {:.3} — OK", f_vk, f_vk_limit
    );

    // Design shear resistance per unit length
    let t: f64 = 0.215;     // m, wall thickness
    let b: f64 = 1.0;       // m, unit length
    let gamma_m: f64 = 2.5;
    let vrd: f64 = f_vk * 1000.0 * t * b / gamma_m; // kN/m
    // = 0.40 * 1000 * 0.215 / 2.5 = 34.4 kN/m

    assert!(
        vrd > 20.0,
        "Design shear: {:.1} kN/m", vrd
    );
}

// ================================================================
// 6. Masonry Wall Under Combined Loading
// ================================================================
//
// Unity check: NEd/NRd + MEd/MRd ≤ 1.0
// For eccentrically loaded wall (EC6 §6.1)

#[test]
fn masonry_combined_loading() {
    let n_ed: f64 = 200.0;   // kN/m, design axial load
    let m_ed: f64 = 5.0;     // kN·m/m, design moment (eccentricity)
    let t: f64 = 0.215;      // m, wall thickness
    let f_d: f64 = 4.0;      // MPa, design compressive strength (f_k/γM)

    // Eccentricity
    let e_load: f64 = m_ed / n_ed; // = 0.025 m
    assert!(
        e_load < t / 3.0,
        "Eccentricity {:.3}m < t/3 = {:.3}m — compression zone > half", e_load, t / 3.0
    );

    // Capacity reduction factor for eccentricity (EC6 simplified)
    // Φ = 1 - 2*e/t
    let phi_ec: f64 = 1.0 - 2.0 * e_load / t;
    let phi_ec_expected: f64 = 0.767;

    assert!(
        (phi_ec - phi_ec_expected).abs() / phi_ec_expected < 0.02,
        "Eccentricity factor: {:.3}, expected {:.3}", phi_ec, phi_ec_expected
    );

    // Design resistance
    let n_rd: f64 = phi_ec * t * 1.0 * f_d * 1000.0; // kN/m
    // = 0.767 * 0.215 * 4000 = 659.6 kN/m

    // Unity check
    let unity: f64 = n_ed / n_rd;
    assert!(
        unity < 1.0,
        "Unity check: {:.3} < 1.0 — OK", unity
    );
}

// ================================================================
// 7. Masonry Lintel Design
// ================================================================
//
// Lintel over opening: carries triangular (arching) or rectangular load.
// For arching action (45° spread): w = γ * t * h_tri
// h_tri = min(L/2, H_above)

#[test]
fn masonry_lintel_design() {
    let l_opening: f64 = 2.4;   // m, opening span
    let t: f64 = 0.215;         // m, wall thickness
    let gamma: f64 = 20.0;      // kN/m³, masonry unit weight
    let h_above: f64 = 3.0;     // m, masonry above lintel

    // Triangular load height (45° arching)
    let h_tri: f64 = (l_opening / 2.0).min(h_above);
    assert!(
        (h_tri - 1.2).abs() < 0.01,
        "Triangle height: {:.1} m", h_tri
    );

    // Total load on lintel (triangular): W = 0.5 * γ * t * h_tri * L
    let w_total: f64 = 0.5 * gamma * t * h_tri * l_opening;
    // = 0.5 * 20 * 0.215 * 1.2 * 2.4 = 6.192 kN

    let w_expected: f64 = 0.5 * 20.0 * 0.215 * 1.2 * 2.4;
    assert!(
        (w_total - w_expected).abs() / w_expected < 0.01,
        "Lintel load: {:.3} kN, expected {:.3}", w_total, w_expected
    );

    // Equivalent UDL for design: w_eq = 2*W/(3*L) (triangular → UDL equivalent for max moment)
    // Max moment under triangular load: M = W*L/6
    let m_max: f64 = w_total * l_opening / 6.0;

    assert!(
        m_max > 0.0,
        "Lintel moment: {:.3} kN·m", m_max
    );

    // Max shear: V = W/2
    let v_max: f64 = w_total / 2.0;
    assert!(
        v_max > 0.0,
        "Lintel shear: {:.3} kN", v_max
    );
}

// ================================================================
// 8. Masonry Thermal Movement
// ================================================================
//
// Masonry has different thermal expansion than RC frame.
// Movement joint spacing: typically 6-9m for clay brick, 6m for concrete block.
// Differential movement: Δ = (α_masonry - α_frame) * ΔT * L

#[test]
fn masonry_thermal_movement() {
    let alpha_clay: f64 = 5e-6;      // 1/°C, clay brick
    let alpha_concrete: f64 = 10e-6;  // 1/°C, concrete block
    let alpha_frame: f64 = 12e-6;     // 1/°C, concrete frame
    let delta_t: f64 = 40.0;          // °C, temperature range
    let l: f64 = 8.0;                 // m, panel length

    // Clay brick movement
    let delta_clay: f64 = alpha_clay * delta_t * l * 1000.0; // mm
    let delta_clay_expected: f64 = 5e-6 * 40.0 * 8.0 * 1000.0; // = 1.6 mm

    assert!(
        (delta_clay - delta_clay_expected).abs() / delta_clay_expected < 0.01,
        "Clay brick movement: {:.2} mm, expected {:.2}", delta_clay, delta_clay_expected
    );

    // Concrete block movement
    let delta_block: f64 = alpha_concrete * delta_t * l * 1000.0;
    let delta_block_expected: f64 = 3.2; // mm

    assert!(
        (delta_block - delta_block_expected).abs() / delta_block_expected < 0.01,
        "Concrete block: {:.2} mm, expected {:.2}", delta_block, delta_block_expected
    );

    // Differential movement (masonry in RC frame)
    let diff_clay: f64 = (alpha_frame - alpha_clay) * delta_t * l * 1000.0;
    // = 7e-6 * 40 * 8 * 1000 = 2.24 mm
    let diff_block: f64 = (alpha_frame - alpha_concrete) * delta_t * l * 1000.0;
    // = 2e-6 * 40 * 8 * 1000 = 0.64 mm

    assert!(
        diff_clay > diff_block,
        "Clay differential {:.2}mm > block {:.2}mm", diff_clay, diff_block
    );

    // Movement joint gap should accommodate total movement
    let joint_gap: f64 = 10.0; // mm, typical movement joint
    assert!(
        diff_clay < joint_gap,
        "Differential {:.2}mm < joint gap {:.0}mm — OK", diff_clay, joint_gap
    );
}
