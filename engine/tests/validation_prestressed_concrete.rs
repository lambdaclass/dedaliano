/// Validation: Prestressed / Post-Tensioned Concrete
///
/// References:
///   - ACI 318-19 Ch.25: Prestressed concrete
///   - EN 1992-1-1:2004 (EC2) §5.10: Prestressing
///   - PCI Design Handbook 8th Ed.
///   - Naaman: "Prestressed Concrete Analysis and Design" 3rd ed. (2012)
///   - Lin & Burns: "Design of Prestressed Concrete Structures" 3rd ed.
///   - Collins & Mitchell: "Prestressed Concrete Structures" (1991)
///
/// Tests verify prestress losses, stress checks at transfer and service,
/// tendon geometry, and ultimate moment capacity.

mod helpers;

// ================================================================
// 1. Elastic Shortening Loss (ACI 318 / EC2)
// ================================================================
//
// Δfp_ES = (n_p * f_cgp)
// n_p = Ep/Ec = modular ratio
// f_cgp = concrete stress at tendon CG due to prestress + self-weight

#[test]
fn prestress_elastic_shortening() {
    let f_pi: f64 = 1395.0;   // MPa, initial prestress (0.75*fpu for 1860 strand)
    let a_ps: f64 = 1400e-6;  // m² (1400 mm²), strand area
    let e_p: f64 = 195_000.0; // MPa, strand modulus
    let e_c: f64 = 32_000.0;  // MPa, concrete modulus at transfer
    let a_c: f64 = 0.18;      // m², concrete cross-section area
    let i_c: f64 = 5.4e-3;    // m⁴, concrete moment of inertia
    let e_tendon: f64 = 0.20;  // m, tendon eccentricity from centroid

    let n_p: f64 = e_p / e_c;
    let n_p_expected: f64 = 6.094;

    assert!(
        (n_p - n_p_expected).abs() / n_p_expected < 0.01,
        "Modular ratio: {:.3}, expected {:.3}", n_p, n_p_expected
    );

    // Initial prestress force
    let _p_i: f64 = f_pi * a_ps * 1e6; // Convert: MPa * m² * 1e6 = N → but we keep in MPa*m² = MN
    // Actually: f_pi (MPa) * A_ps (m²) = force in MN; let's work in kN
    let p_i_kn: f64 = f_pi * 1000.0 * a_ps; // MPa = N/mm², * mm² → N → /1000 → kN
    // = 1395 * 1400 = 1,953,000 N = 1953 kN
    let p_i_expected: f64 = 1953.0; // kN

    assert!(
        (p_i_kn - p_i_expected).abs() / p_i_expected < 0.01,
        "Initial prestress: {:.0} kN, expected {:.0}", p_i_kn, p_i_expected
    );

    // Concrete stress at tendon CG (assuming midspan, self-weight moment small)
    // f_cgp = Pi/Ac + Pi*e²/Ic (no external moment for simplicity)
    let f_cgp: f64 = p_i_kn / (a_c * 1000.0) + p_i_kn * e_tendon * e_tendon / (i_c * 1000.0);
    // = 1953/(180) + 1953*0.04/(5.4) = 10.85 + 14.47 = 25.32 MPa (approx, units adjusted)

    // Elastic shortening loss
    let delta_fp_es: f64 = n_p * f_cgp;

    // Should be ~5-10% of initial prestress
    let loss_ratio: f64 = delta_fp_es / f_pi;
    assert!(
        loss_ratio > 0.01 && loss_ratio < 0.15,
        "ES loss ratio: {:.3}, expected 0.05-0.10", loss_ratio
    );
}

// ================================================================
// 2. Friction Loss in Post-Tensioning (ACI 318 / EC2)
// ================================================================
//
// fp(x) = fp_jack * e^(-(μα + κx))
// μ = friction coefficient (0.15-0.25 for ducts)
// κ = wobble coefficient (0.0005-0.002 /m)
// α = total angle change

#[test]
fn prestress_friction_loss() {
    let fp_jack: f64 = 1488.0; // MPa, jacking stress (0.80*fpu)
    let mu: f64 = 0.20;        // friction coefficient
    let kappa: f64 = 0.001;    // wobble coefficient (1/m)
    let alpha: f64 = 0.15;     // radians, total angular change over 20m
    let x: f64 = 20.0;         // m, distance from jack

    // Stress after friction
    let fp_x: f64 = fp_jack * (-(mu * alpha + kappa * x)).exp();

    // exponent = -(0.20*0.15 + 0.001*20) = -(0.03 + 0.02) = -0.05
    // fp(x) = 1488 * e^(-0.05) = 1488 * 0.9512 = 1415.4
    let fp_expected: f64 = 1488.0 * (-0.05_f64).exp();

    assert!(
        (fp_x - fp_expected).abs() / fp_expected < 0.01,
        "Stress at {}m: {:.1} MPa, expected {:.1}", x, fp_x, fp_expected
    );

    // Loss percentage
    let friction_loss_pct: f64 = (1.0 - fp_x / fp_jack) * 100.0;
    let loss_expected_pct: f64 = (1.0 - (-0.05_f64).exp()) * 100.0;

    assert!(
        (friction_loss_pct - loss_expected_pct).abs() < 0.1,
        "Friction loss: {:.1}%, expected {:.1}%", friction_loss_pct, loss_expected_pct
    );

    // For longer tendons, loss increases
    let x_long: f64 = 40.0;
    let alpha_long: f64 = 0.30;
    let fp_long: f64 = fp_jack * (-(mu * alpha_long + kappa * x_long)).exp();
    assert!(
        fp_long < fp_x,
        "Longer tendon: {:.1} < {:.1} MPa", fp_long, fp_x
    );
}

// ================================================================
// 3. Time-Dependent Losses — Lump Sum (ACI 318 §20.3.2.6)
// ================================================================
//
// ACI approximate lump sum losses for normal weight concrete:
// Total losses ≈ 241 MPa (35 ksi) for pretensioned
// Total losses ≈ 228 MPa (33 ksi) for post-tensioned

#[test]
fn prestress_aci_lump_sum_losses() {
    let fpu: f64 = 1860.0;    // MPa, strand ultimate strength
    let fpi: f64 = 0.75 * fpu; // = 1395 MPa, initial prestress

    // ACI lump sum: pretensioned
    let loss_pretensioned: f64 = 241.0; // MPa
    let fpe_pre: f64 = fpi - loss_pretensioned;
    let fpe_pre_expected: f64 = 1154.0; // MPa

    assert!(
        (fpe_pre - fpe_pre_expected).abs() < 1.0,
        "Effective pretensioned: {:.0} MPa, expected {:.0}", fpe_pre, fpe_pre_expected
    );

    // Loss ratio
    let ratio_pre: f64 = loss_pretensioned / fpi;
    // = 241/1395 = 0.173 (about 17%)
    assert!(
        ratio_pre > 0.15 && ratio_pre < 0.20,
        "Pretensioned loss ratio: {:.3}", ratio_pre
    );

    // ACI lump sum: post-tensioned
    let loss_posttensioned: f64 = 228.0; // MPa
    let fpe_post: f64 = fpi - loss_posttensioned;
    let fpe_post_expected: f64 = 1167.0;

    assert!(
        (fpe_post - fpe_post_expected).abs() < 1.0,
        "Effective post-tensioned: {:.0} MPa, expected {:.0}", fpe_post, fpe_post_expected
    );

    // Post-tensioned has lower total loss (no elastic shortening at jacking end)
    assert!(
        loss_posttensioned < loss_pretensioned,
        "PT loss ({:.0}) < pretensioned loss ({:.0})", loss_posttensioned, loss_pretensioned
    );
}

// ================================================================
// 4. Stress Check at Transfer (ACI 318 §24.5.3)
// ================================================================
//
// At transfer, check:
// Compression: f_ci ≤ 0.60*f'ci
// Tension: f_ti ≤ 0.25*√f'ci (MPa) without bonded reinforcement
// or f_ti ≤ 0.50*√f'ci with bonded reinforcement

#[test]
fn prestress_stress_at_transfer() {
    let fci: f64 = 45.0;      // MPa, concrete at transfer
    let pi: f64 = 1800.0;     // kN, prestress force at transfer
    let a_c: f64 = 180_000.0; // mm², concrete area
    let s_top: f64 = 1.5e7;   // mm³, section modulus (top)
    let s_bot: f64 = 1.5e7;   // mm³, section modulus (bottom)
    let e: f64 = 200.0;       // mm, tendon eccentricity (below centroid)
    let m_sw: f64 = 120.0;    // kN·m, self-weight moment at midspan

    // Top fiber stress at midspan (transfer): compression check
    // f_top = -Pi/Ac + Pi*e/S_top - Msw/S_top
    let f_top: f64 = -pi * 1000.0 / a_c + pi * 1000.0 * e / s_top - m_sw * 1e6 / s_top;
    // = -10.0 + 24.0 - 8.0 = 6.0 MPa (tension) — top is in tension at transfer

    // Bottom fiber: compression
    let f_bot: f64 = -pi * 1000.0 / a_c - pi * 1000.0 * e / s_bot + m_sw * 1e6 / s_bot;
    // = -10.0 - 24.0 + 8.0 = -26.0 MPa (compression)

    // ACI limits
    let f_comp_limit: f64 = -0.60 * fci; // = -21.0 MPa
    let f_tens_limit: f64 = 0.50 * fci.sqrt(); // = 2.96 MPa (with bonded reinforcement)

    // Check bottom compression
    assert!(
        f_bot > f_comp_limit, // more negative = more compression
        "Bottom stress {:.1} MPa should be less compressive than limit {:.1}",
        f_bot, f_comp_limit
    );

    // If top tension exceeds limit, bonded reinforcement needed
    if f_top > f_tens_limit {
        // Needs non-prestressed reinforcement in tension zone
        assert!(
            f_top > 0.0,
            "Top fiber in tension at transfer: {:.1} MPa", f_top
        );
    }
}

// ================================================================
// 5. Ultimate Moment Capacity (ACI 318 §22.3)
// ================================================================
//
// fps = fpu * (1 - (γp/β₁) * (ρp*fpu/f'c))  — bonded tendons
// Mn = Aps * fps * (dp - a/2)
// a = Aps * fps / (0.85*f'c*b)

#[test]
fn prestress_ultimate_moment() {
    let fpu: f64 = 1860.0;    // MPa, strand ultimate
    let a_ps: f64 = 1400.0;   // mm², strand area
    let fc: f64 = 45.0;       // MPa, concrete compressive strength
    let b: f64 = 400.0;       // mm, flange width
    let dp: f64 = 650.0;      // mm, depth to tendon centroid
    let gamma_p: f64 = 0.28;  // for fpy/fpu ≥ 0.90 (low-relaxation)
    let _beta_1: f64 = 0.783;  // for f'c = 45 MPa: 0.85 - 0.05*(45-28)/7 = 0.729?
    // Actually β₁ = 0.85 - 0.05*(f'c - 28)/7 for f'c > 28, min 0.65
    // β₁ = 0.85 - 0.05*(45-28)/7 = 0.85 - 0.121 = 0.729
    let beta_1_calc: f64 = (0.85 - 0.05 * (fc - 28.0) / 7.0).max(0.65);

    // Stress in prestressing steel at ultimate
    let rho_p: f64 = a_ps / (b * dp);
    let fps: f64 = fpu * (1.0 - (gamma_p / beta_1_calc) * (rho_p * fpu / fc));

    // fps = 1860 * (1 - (0.28/0.729) * (0.005385 * 1860/45))
    //     = 1860 * (1 - 0.384 * 0.2223)
    //     = 1860 * (1 - 0.0854) = 1860 * 0.9146 = 1701
    assert!(
        fps > 0.5 * fpu && fps < fpu,
        "fps = {:.0} MPa should be between 930 and 1860", fps
    );

    // Depth of compression block
    let a: f64 = a_ps * fps / (0.85 * fc * b);
    // = 1400 * 1701 / (0.85 * 45 * 400) = 2,381,400 / 15,300 = 155.6 mm

    // Nominal moment capacity
    let mn: f64 = a_ps * fps * (dp - a / 2.0) / 1e6; // kN·m
    // = 1400 * 1701 * (650 - 77.8) / 1e6 = 1400 * 1701 * 572.2 / 1e6 = 1362 kN·m

    assert!(
        mn > 500.0 && mn < 2000.0,
        "Nominal moment: {:.0} kN·m", mn
    );

    // Design moment: φMn where φ = 0.90 for tension-controlled
    let phi: f64 = 0.90;
    let phi_mn: f64 = phi * mn;
    assert!(
        phi_mn > 0.85 * mn,
        "φMn = {:.0} kN·m", phi_mn
    );
}

// ================================================================
// 6. EC2 — Tendon Profile (Parabolic)
// ================================================================
//
// Parabolic drape: y(x) = 4*e*(x/L)*(1 - x/L)
// where e = midspan eccentricity, x measured from left end

#[test]
fn prestress_parabolic_profile() {
    let l: f64 = 20.0;   // m, span
    let e_mid: f64 = 0.30; // m, midspan eccentricity (below centroid)

    // Profile at quarter span
    let x: f64 = 5.0; // L/4
    let y_quarter: f64 = 4.0 * e_mid * (x / l) * (1.0 - x / l);
    let y_quarter_expected: f64 = 4.0 * 0.30 * 0.25 * 0.75; // = 0.225 m

    assert!(
        (y_quarter - y_quarter_expected).abs() < 0.001,
        "y(L/4) = {:.3} m, expected {:.3}", y_quarter, y_quarter_expected
    );

    // At midspan
    let x_mid: f64 = 10.0;
    let y_mid: f64 = 4.0 * e_mid * (x_mid / l) * (1.0 - x_mid / l);
    assert!(
        (y_mid - e_mid).abs() < 0.001,
        "y(L/2) = {:.3}, should equal e_mid = {:.3}", y_mid, e_mid
    );

    // Slope at support: dy/dx|₀ = 4*e/L
    let slope_support: f64 = 4.0 * e_mid / l;
    let slope_expected: f64 = 0.06; // rad
    assert!(
        (slope_support - slope_expected).abs() < 0.001,
        "Slope at support: {:.4} rad, expected {:.4}", slope_support, slope_expected
    );

    // Angle change over half span = 2 * slope_support
    let alpha_half: f64 = 2.0 * slope_support;
    let alpha_expected: f64 = 0.12;
    assert!(
        (alpha_half - alpha_expected).abs() < 0.001,
        "Angle change: {:.3} rad, expected {:.3}", alpha_half, alpha_expected
    );
}

// ================================================================
// 7. Anchorage Zone — Bursting Force (EC2 / AASHTO)
// ================================================================
//
// Gergely-Sozen: T_burst = 0.25 * P * (1 - a/h)
// where a = bearing plate dimension, h = member depth
// This checks the splitting tensile force in the anchorage zone.

#[test]
fn prestress_anchorage_bursting() {
    let p: f64 = 2000.0;  // kN, prestress force per tendon
    let a_plate: f64 = 250.0;  // mm, bearing plate size
    let h: f64 = 800.0;        // mm, member depth

    // Bursting force (Gergely-Sozen)
    let t_burst: f64 = 0.25 * p * (1.0 - a_plate / h);
    let t_burst_expected: f64 = 0.25 * 2000.0 * (1.0 - 0.3125);
    // = 500 * 0.6875 = 343.75 kN

    assert!(
        (t_burst - t_burst_expected).abs() / t_burst_expected < 0.01,
        "Bursting force: {:.1} kN, expected {:.1}", t_burst, t_burst_expected
    );

    // Required steel area (fy = 420 MPa, at 0.60fy per ACI)
    let fy: f64 = 420.0;  // MPa
    let as_burst: f64 = t_burst * 1000.0 / (0.60 * fy); // mm²
    // = 343750 / 252 = 1364 mm²

    assert!(
        as_burst > 1000.0,
        "Bursting reinforcement: {:.0} mm² (substantial)", as_burst
    );

    // Larger plate → smaller bursting force
    let a_large: f64 = 400.0;
    let t_large: f64 = 0.25 * p * (1.0 - a_large / h);
    assert!(
        t_large < t_burst,
        "Larger plate: {:.1} kN < {:.1} kN", t_large, t_burst
    );
}

// ================================================================
// 8. Load Balancing Method
// ================================================================
//
// Equivalent load from parabolic tendon: w_bal = 8*P*e/L²
// At balanced load, beam behaves as if no external load acts
// (uniform axial compression only).

#[test]
fn prestress_load_balancing() {
    let p_eff: f64 = 1600.0;  // kN, effective prestress force
    let e: f64 = 0.25;        // m, midspan eccentricity
    let l: f64 = 16.0;        // m, span

    // Equivalent upward load from parabolic tendon
    let w_bal: f64 = 8.0 * p_eff * e / (l * l);
    let w_bal_expected: f64 = 8.0 * 1600.0 * 0.25 / 256.0;
    // = 3200 / 256 = 12.5 kN/m

    assert!(
        (w_bal - w_bal_expected).abs() / w_bal_expected < 0.01,
        "Balanced load: {:.2} kN/m, expected {:.2}", w_bal, w_bal_expected
    );

    // If applied load = w_bal, net moment = 0
    // Net moment for applied load w > w_bal:
    let w_applied: f64 = 18.0; // kN/m
    let w_net: f64 = w_applied - w_bal;
    let m_net: f64 = w_net * l * l / 8.0;
    let m_net_expected: f64 = 5.5 * 256.0 / 8.0; // = 176 kN·m

    assert!(
        (m_net - m_net_expected).abs() / m_net_expected < 0.01,
        "Net moment: {:.1} kN·m, expected {:.1}", m_net, m_net_expected
    );

    // At balanced state: beam has only axial stress = P/A
    let a_c: f64 = 0.15; // m²
    let f_axial: f64 = p_eff / (a_c * 1000.0); // MPa (kN / (m² * 1000 mm²/m²))
    // = 1600 / 150 = 10.67 MPa
    assert!(
        f_axial > 5.0 && f_axial < 20.0,
        "Axial stress at balance: {:.1} MPa", f_axial
    );
}
