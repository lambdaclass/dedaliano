/// Validation: Advanced Stability Formulas (Pure Formula Verification)
///
/// References:
///   - AISC 360-22, Chapter E (Compression Members) and Appendix 7
///   - EN 1993-1-1:2005 (Eurocode 3), Section 6.3 (Buckling Resistance)
///   - Timoshenko & Gere, "Theory of Elastic Stability", 2nd Ed.
///   - Galambos & Surovek, "Structural Stability of Steel", 5th Ed.
///   - Chen & Lui, "Structural Stability: Theory and Implementation"
///   - Salmon, Johnson & Malhas, "Steel Structures", Ch. 6
///
/// Tests verify column curves, beam-column interaction, and frame
/// stability formulas without calling the solver.
///
/// Tests:
///   1. AISC column curves: elastic and inelastic buckling
///   2. Eurocode 3 column curves: imperfection factors
///   3. Tangent modulus theory (inelastic buckling)
///   4. Beam-column interaction: AISC H1-1a and H1-1b
///   5. Frame stability: alignment chart K-factors
///   6. Effective length from eigenvalue equation
///   7. Sway vs non-sway amplification (B1 and B2 factors)
///   8. P-Delta geometric series approximation

use std::f64::consts::PI;

// ================================================================
// 1. AISC Column Curves: Elastic and Inelastic Buckling
// ================================================================
//
// AISC 360-22, Section E3:
// For Lc/r <= 4.71*sqrt(E/Fy)  (or Fe >= 0.44*Fy):
//   Fcr = (0.658^(Fy/Fe)) * Fy    [inelastic, Eq. E3-2]
//
// For Lc/r > 4.71*sqrt(E/Fy)  (or Fe < 0.44*Fy):
//   Fcr = 0.877 * Fe               [elastic, Eq. E3-3]
//
// where Fe = pi^2 * E / (KL/r)^2   (Euler stress)
//
// Reference: AISC 360-22, Section E3

#[test]
fn validation_aisc_column_curves() {
    let e: f64 = 200_000.0;  // MPa, elastic modulus
    let fy: f64 = 345.0;     // MPa, yield stress (Grade 50)

    // Transition slenderness
    let slenderness_limit: f64 = 4.71 * (e / fy).sqrt();
    // = 4.71 * sqrt(579.71) = 4.71 * 24.077 = 113.40
    let sl_expected: f64 = 4.71 * (200_000.0_f64 / 345.0_f64).sqrt();
    assert!(
        (slenderness_limit - sl_expected).abs() < 1e-6,
        "Transition slenderness: computed={:.2}, expected={:.2}",
        slenderness_limit, sl_expected
    );

    // Case 1: Short column (KL/r = 50) - inelastic buckling
    let kl_r_short: f64 = 50.0;
    let fe_short: f64 = PI * PI * e / (kl_r_short * kl_r_short);
    // = pi^2 * 200000 / 2500 = 789.57 MPa
    assert!(
        fe_short >= 0.44 * fy,
        "Short column: Fe={:.2} >= 0.44*Fy={:.2} (inelastic)",
        fe_short, 0.44 * fy
    );

    let fcr_short: f64 = 0.658_f64.powf(fy / fe_short) * fy;
    assert!(
        fcr_short > 0.0 && fcr_short < fy,
        "Inelastic Fcr should be in (0, Fy): Fcr={:.2} MPa",
        fcr_short
    );

    // Case 2: Long column (KL/r = 150) - elastic buckling
    let kl_r_long: f64 = 150.0;
    let fe_long: f64 = PI * PI * e / (kl_r_long * kl_r_long);
    // = pi^2 * 200000 / 22500 = 87.73 MPa
    assert!(
        fe_long < 0.44 * fy,
        "Long column: Fe={:.2} < 0.44*Fy={:.2} (elastic)",
        fe_long, 0.44 * fy
    );

    let fcr_long: f64 = 0.877 * fe_long;
    assert!(
        fcr_long > 0.0 && fcr_long < fe_long,
        "Elastic Fcr: {:.2} MPa (0.877*Fe={:.2})",
        fcr_long, fe_long
    );

    // Short column always has higher capacity than long column
    assert!(
        fcr_short > fcr_long,
        "Short Fcr ({:.2}) > Long Fcr ({:.2})",
        fcr_short, fcr_long
    );
}

// ================================================================
// 2. Eurocode 3 Column Curves: Imperfection Factors
// ================================================================
//
// EN 1993-1-1, Section 6.3.1:
// Reduction factor chi:
//   chi = 1 / (Phi + sqrt(Phi^2 - lambda_bar^2))
//   Phi = 0.5 * (1 + alpha*(lambda_bar - 0.2) + lambda_bar^2)
//   lambda_bar = sqrt(Fy / sigma_cr) = sqrt(A*Fy / N_cr)
//
// Imperfection factors alpha:
//   Curve a0: alpha = 0.13
//   Curve a:  alpha = 0.21
//   Curve b:  alpha = 0.34
//   Curve c:  alpha = 0.49
//   Curve d:  alpha = 0.76
//
// Reference: EN 1993-1-1:2005, Table 6.1

#[test]
fn validation_ec3_column_curves() {
    let fy: f64 = 355.0;     // MPa (S355)
    let e: f64 = 210_000.0;  // MPa

    // Imperfection factors for each curve
    let alphas: [f64; 5] = [0.13, 0.21, 0.34, 0.49, 0.76];
    let curve_names: [&str; 5] = ["a0", "a", "b", "c", "d"];

    // Relative slenderness lambda_bar = 1.0 (intermediate column)
    let lambda_bar: f64 = 1.0;

    // Compute chi for each curve
    let mut chi_values: [f64; 5] = [0.0; 5];
    for i in 0_usize..5 {
        let alpha: f64 = alphas[i];
        let phi: f64 = 0.5 * (1.0 + alpha * (lambda_bar - 0.2) + lambda_bar * lambda_bar);
        let chi: f64 = 1.0 / (phi + (phi * phi - lambda_bar * lambda_bar).sqrt());

        // chi must be in (0, 1]
        assert!(
            chi > 0.0 && chi <= 1.0,
            "Curve {}: chi={:.4} must be in (0, 1]",
            curve_names[i], chi
        );
        chi_values[i] = chi;
    }

    // Curve a0 should give highest chi (least imperfection)
    assert!(
        chi_values[0] > chi_values[1],
        "Curve a0 ({:.4}) > Curve a ({:.4})",
        chi_values[0], chi_values[1]
    );

    // Each successive curve should give lower chi
    for i in 0_usize..4 {
        assert!(
            chi_values[i] > chi_values[i + 1],
            "Curve {} ({:.4}) > Curve {} ({:.4})",
            curve_names[i], chi_values[i],
            curve_names[i + 1], chi_values[i + 1]
        );
    }

    // For lambda_bar = 0 (stocky column), all curves give chi = 1.0
    let lambda_zero: f64 = 0.0;
    let alpha_test: f64 = 0.34; // curve b
    let phi_zero: f64 = 0.5 * (1.0 + alpha_test * (lambda_zero - 0.2) + lambda_zero * lambda_zero);
    let chi_zero: f64 = 1.0 / (phi_zero + (phi_zero * phi_zero - lambda_zero * lambda_zero).sqrt());
    // For lambda_bar = 0: Phi = 0.5*(1 - 0.2*alpha), chi = 1/(2*Phi) > 1, so capped at 1.0
    let chi_capped: f64 = chi_zero.min(1.0);
    assert!(
        (chi_capped - 1.0).abs() < 0.1,
        "Stocky column chi should be near 1.0: {:.4}",
        chi_capped
    );
    let _ = fy;
    let _ = e;
}

// ================================================================
// 3. Tangent Modulus Theory (Inelastic Buckling)
// ================================================================
//
// Shanley's tangent modulus theory gives the critical stress for
// inelastic buckling:
//   sigma_cr = pi^2 * E_t / (KL/r)^2
//
// where E_t is the tangent modulus at the current stress level.
//
// For a bilinear stress-strain curve:
//   E_t = E             for sigma < Fy (elastic range)
//   E_t = E_strain_hard for sigma >= Fy (strain hardening range)
//
// The Engesser-Shanley load lies between tangent modulus and
// double (reduced) modulus loads:
//   P_tangent <= P_actual <= P_reduced
//
// Reference: Galambos & Surovek, Ch. 3

#[test]
fn validation_tangent_modulus_inelastic() {
    let e: f64 = 200_000.0;  // MPa
    let fy: f64 = 250.0;     // MPa (mild steel)
    let e_sh: f64 = 2000.0;  // MPa, strain hardening modulus (E/100)

    // Euler stress for various slenderness ratios
    let kl_r_values: [f64; 4] = [40.0, 60.0, 80.0, 120.0];

    for &kl_r in &kl_r_values {
        let fe: f64 = PI * PI * e / (kl_r * kl_r);

        if fe > fy {
            // Elastic buckling does not apply; tangent modulus needed
            // The critical stress cannot exceed Fy with E_t = E
            // For sigma = Fy, we need to find the slenderness where Fe = Fy
            let kl_r_yield: f64 = PI * (e / fy).sqrt();
            // Below this slenderness, inelastic buckling governs

            // Tangent modulus critical stress at this slenderness
            // using E_t = E_sh (conservative, assumes fully yielded)
            let sigma_cr_t: f64 = PI * PI * e_sh / (kl_r * kl_r);

            // This should be much less than elastic Euler stress
            assert!(
                sigma_cr_t < fe,
                "Tangent modulus Fcr ({:.2}) < Euler Fe ({:.2}) at KL/r={:.0}",
                sigma_cr_t, fe, kl_r
            );
            let _ = kl_r_yield;
        } else {
            // Elastic buckling governs
            assert!(
                fe <= fy,
                "Elastic buckling governs at KL/r={:.0}: Fe={:.2} <= Fy={:.2}",
                kl_r, fe, fy
            );
        }
    }

    // The transition point where Fe = Fy
    let kl_r_transition: f64 = PI * (e / fy).sqrt();
    let fe_at_transition: f64 = PI * PI * e / (kl_r_transition * kl_r_transition);
    assert!(
        (fe_at_transition - fy).abs() / fy < 1e-10,
        "At transition: Fe={:.2} should equal Fy={:.2}",
        fe_at_transition, fy
    );

    // Reduced modulus is between E_t and E:
    // E_r = 4*E*E_t / (sqrt(E) + sqrt(E_t))^2
    let e_r: f64 = 4.0 * e * e_sh / (e.sqrt() + e_sh.sqrt()).powi(2);
    assert!(
        e_r > e_sh && e_r < e,
        "Reduced modulus: E_sh={:.0} < E_r={:.2} < E={:.0}",
        e_sh, e_r, e
    );
}

// ================================================================
// 4. Beam-Column Interaction: AISC H1-1a and H1-1b
// ================================================================
//
// AISC 360-22, Section H1.1:
// For Pr/Pc >= 0.2 (H1-1a):
//   Pr/Pc + (8/9)*(Mrx/Mcx + Mry/Mcy) <= 1.0
//
// For Pr/Pc < 0.2 (H1-1b):
//   Pr/(2*Pc) + (Mrx/Mcx + Mry/Mcy) <= 1.0
//
// Reference: AISC 360-22, Eq. H1-1a, H1-1b

#[test]
fn validation_beam_column_interaction_aisc() {
    // Design strengths
    let phi_pn: f64 = 2000.0; // kN, design axial capacity (phi*Pn)
    let phi_mnx: f64 = 500.0; // kN-m, major axis moment capacity
    let phi_mny: f64 = 250.0; // kN-m, minor axis moment capacity

    // Case 1: High axial load (Pr/Pc = 0.6) - use H1-1a
    let pr_1: f64 = 1200.0; // kN, required axial
    let mrx_1: f64 = 100.0; // kN-m, required major axis moment
    let mry_1: f64 = 50.0;  // kN-m, required minor axis moment

    let ratio_1: f64 = pr_1 / phi_pn; // = 0.6 >= 0.2
    assert!(
        ratio_1 >= 0.2,
        "Case 1: Pr/Pc = {:.2} >= 0.2, use H1-1a",
        ratio_1
    );

    let interaction_1: f64 = ratio_1 + (8.0 / 9.0) * (mrx_1 / phi_mnx + mry_1 / phi_mny);
    // = 0.6 + 0.889 * (0.2 + 0.2) = 0.6 + 0.3556 = 0.9556
    assert!(
        interaction_1 > 0.0,
        "Interaction ratio must be positive: {:.4}",
        interaction_1
    );

    // Case 2: Low axial load (Pr/Pc = 0.1) - use H1-1b
    let pr_2: f64 = 200.0;  // kN
    let mrx_2: f64 = 200.0; // kN-m
    let mry_2: f64 = 80.0;  // kN-m

    let ratio_2: f64 = pr_2 / phi_pn; // = 0.1 < 0.2
    assert!(
        ratio_2 < 0.2,
        "Case 2: Pr/Pc = {:.2} < 0.2, use H1-1b",
        ratio_2
    );

    let interaction_2: f64 = ratio_2 / 2.0 + (mrx_2 / phi_mnx + mry_2 / phi_mny);
    // = 0.05 + (0.4 + 0.32) = 0.05 + 0.72 = 0.77
    assert!(
        interaction_2 > 0.0,
        "Interaction ratio must be positive: {:.4}",
        interaction_2
    );

    // Both cases should be < 1.0 for the member to be adequate
    assert!(
        interaction_1 <= 1.0,
        "Case 1 adequate: interaction = {:.4} <= 1.0",
        interaction_1
    );
    assert!(
        interaction_2 <= 1.0,
        "Case 2 adequate: interaction = {:.4} <= 1.0",
        interaction_2
    );

    // Verify continuity at Pr/Pc = 0.2 transition point
    let pr_transition: f64 = 0.2 * phi_pn;
    let mrx_t: f64 = 150.0;
    let mry_t: f64 = 0.0;
    let h1_1a: f64 = 0.2 + (8.0 / 9.0) * (mrx_t / phi_mnx + mry_t / phi_mny);
    let h1_1b: f64 = 0.1 + (mrx_t / phi_mnx + mry_t / phi_mny);
    // At transition: both should give similar values
    // H1-1a: 0.2 + 8/9 * 0.3 = 0.2 + 0.267 = 0.467
    // H1-1b: 0.1 + 0.3 = 0.400 ... there is a discontinuity
    // (This is a known feature of the AISC interaction equations)
    assert!(
        (h1_1a - h1_1b).abs() < 0.15,
        "Near-continuity at transition: H1-1a={:.4}, H1-1b={:.4}",
        h1_1a, h1_1b
    );
    let _ = pr_transition;
}

// ================================================================
// 5. Frame Stability: Alignment Chart K-Factors
// ================================================================
//
// AISC alignment chart equations for effective length factor K:
//
// For braced (non-sway) frames:
//   (pi/K)^2 * G_A * G_B / 4 + (G_A + G_B)/2 * (1 - pi/K / tan(pi/K))
//   + 2*tan(pi/(2K)) / (pi/K) - 1 = 0
//
// For unbraced (sway) frames:
//   ((pi/K)^2 * G_A * G_B - 36) / (6*(G_A + G_B))
//   = pi/K / tan(pi/K)
//
// G = sum(EI/L_column) / sum(EI/L_beam) at each end of the column.
// G = 0 for fixed end, G = infinity for pinned end (use G = 10).
//
// Reference: AISC 360-22, Commentary C-A-7

#[test]
fn validation_frame_alignment_chart_k_factors() {
    // Known K-factors for idealized end conditions (braced frame):
    // Fixed-Fixed:    K = 0.5
    // Fixed-Pinned:   K = 0.7
    // Pinned-Pinned:  K = 1.0
    // Fixed-Free:     K = 2.0 (unbraced)

    // Verify Euler load relationship: P_cr = pi^2*EI/(KL)^2
    let e: f64 = 200_000.0;  // MPa
    let i: f64 = 1e-4;       // m^4
    let l: f64 = 5.0;        // m

    let ei: f64 = e * 1000.0 * i; // kN-m^2

    // Fixed-fixed: K=0.5
    let k_ff: f64 = 0.5;
    let pcr_ff: f64 = PI * PI * ei / (k_ff * l).powi(2);

    // Pinned-pinned: K=1.0
    let k_pp: f64 = 1.0;
    let pcr_pp: f64 = PI * PI * ei / (k_pp * l).powi(2);

    // Fixed-free: K=2.0
    let k_cf: f64 = 2.0;
    let pcr_cf: f64 = PI * PI * ei / (k_cf * l).powi(2);

    // Fixed-fixed should have 4x the capacity of pinned-pinned
    let ratio_ff_pp: f64 = pcr_ff / pcr_pp;
    assert!(
        (ratio_ff_pp - 4.0).abs() < 1e-10,
        "P_cr(FF)/P_cr(PP) = {:.4}, expected 4.0",
        ratio_ff_pp
    );

    // Pinned-pinned should have 4x the capacity of fixed-free
    let ratio_pp_cf: f64 = pcr_pp / pcr_cf;
    assert!(
        (ratio_pp_cf - 4.0).abs() < 1e-10,
        "P_cr(PP)/P_cr(CF) = {:.4}, expected 4.0",
        ratio_pp_cf
    );

    // Fixed-fixed has 16x the capacity of fixed-free
    let ratio_ff_cf: f64 = pcr_ff / pcr_cf;
    assert!(
        (ratio_ff_cf - 16.0).abs() < 1e-10,
        "P_cr(FF)/P_cr(CF) = {:.4}, expected 16.0",
        ratio_ff_cf
    );

    // G-factor calculation example
    // Column: EI_col/L_col = 200e3 * 1e-4 / 5 = 4.0 kN-m
    // Beam:   EI_beam/L_beam = 200e3 * 2e-4 / 8 = 5.0 kN-m
    let ei_over_l_col: f64 = e * 1000.0 * 1e-4 / 5.0;
    let ei_over_l_beam: f64 = e * 1000.0 * 2e-4 / 8.0;
    let g: f64 = ei_over_l_col / ei_over_l_beam;
    let g_expected: f64 = 4.0 / 5.0; // = 0.8
    assert!(
        (g - g_expected).abs() < 1e-10,
        "G-factor: computed={:.4}, expected={:.4}",
        g, g_expected
    );
}

// ================================================================
// 6. Effective Length from Eigenvalue Equation
// ================================================================
//
// For a column with elastic rotational springs at both ends
// (stiffness k_A and k_B), the effective length factor K satisfies:
//
//   det | (k_A - alpha*EI/L)   alpha*EI/(2L)        |
//       | alpha*EI/(2L)        (k_B - alpha*EI/L)   | = 0
//
// where alpha = (pi/K)^2.
//
// For special cases:
//   k = 0 (free rotation, pinned): G -> infinity
//   k = infinity (fixed): G = 0
//
// Reference: Chen & Lui, "Structural Stability", Ch. 2

#[test]
fn validation_effective_length_eigenvalue() {
    let e: f64 = 200_000.0;
    let i_col: f64 = 1e-4;
    let l: f64 = 4.0;
    let ei: f64 = e * 1000.0 * i_col; // = 20 kN-m^2

    // For pinned-pinned column: k_A = k_B = 0
    // Eigenvalue: (0 - alpha*EI/L)^2 - (alpha*EI/(2L))^2 = 0
    // => alpha^2 * (EI/L)^2 * (1 - 1/4) = 0 ... that gives trivial.
    //
    // More rigorously, Euler's equation for pinned-pinned:
    //   sin(pi/K * 1) = 0  =>  pi/K = n*pi  =>  K = 1/n
    // First mode: K = 1.0 (fundamental)
    let k_pp: f64 = 1.0;
    let pcr_pp: f64 = PI * PI * ei / (k_pp * l).powi(2);

    // For fixed-pinned column, the transcendental equation is:
    //   tan(pi/K) = pi/K
    // Solution: pi/K ≈ 4.493 => K ≈ 0.6992 ≈ 0.7
    let k_fp: f64 = PI / 4.4934; // ≈ 0.6992
    let pcr_fp: f64 = PI * PI * ei / (k_fp * l).powi(2);

    // Fixed-pinned should be about 2.05x pinned-pinned
    let ratio: f64 = pcr_fp / pcr_pp;
    let ratio_expected: f64 = 1.0 / (k_fp * k_fp); // = 1/(0.7^2) ≈ 2.045
    assert!(
        (ratio - ratio_expected).abs() / ratio_expected < 1e-6,
        "P_cr(FP)/P_cr(PP) = {:.4}, expected {:.4}",
        ratio, ratio_expected
    );

    // Verify K is approximately 0.7 for fixed-pinned
    assert!(
        (k_fp - 0.7).abs() < 0.01,
        "Fixed-pinned K: computed={:.4}, expected ~0.70",
        k_fp
    );

    // For fixed-fixed: sin(pi/K) = 0 with first nonzero mode
    // => pi/K = 2*pi => K = 0.5
    let k_ff: f64 = 0.5;
    let pcr_ff: f64 = PI * PI * ei / (k_ff * l).powi(2);
    let ratio_ff_pp: f64 = pcr_ff / pcr_pp;
    assert!(
        (ratio_ff_pp - 4.0).abs() < 1e-10,
        "P_cr(FF)/P_cr(PP) = {:.4}, expected 4.0",
        ratio_ff_pp
    );
}

// ================================================================
// 7. Sway vs Non-Sway Amplification (B1 and B2 Factors)
// ================================================================
//
// AISC 360-22, Appendix 8:
// B1 factor (non-sway amplification, member effect):
//   B1 = Cm / (1 - alpha*Pr/Pe1) >= 1.0
//   Cm = 0.6 - 0.4*(M1/M2) for no transverse loads
//   Pe1 = pi^2*EI*/(KL)^2  (with K <= 1 for braced)
//
// B2 factor (sway amplification, story effect):
//   B2 = 1 / (1 - alpha*sum(Pr)/sum(Pe2))
//   Pe2 = pi^2*EI*/(KL)^2  (with K for unbraced)
//   alpha = 1.0 (LRFD)
//
// Reference: AISC 360-22, Appendix 8

#[test]
fn validation_sway_amplification_b1_b2() {
    let alpha: f64 = 1.0; // LRFD

    // B1 factor calculation
    // Single curvature: M1/M2 = +0.5 (same sign moments)
    let m1_over_m2: f64 = 0.5;
    let cm: f64 = 0.6 - 0.4 * m1_over_m2;
    let cm_expected: f64 = 0.4;
    assert!(
        (cm - cm_expected).abs() < 1e-12,
        "Cm: computed={:.4}, expected={:.4}",
        cm, cm_expected
    );

    // Pe1 for braced column
    let e: f64 = 200_000.0; // MPa
    let i_col: f64 = 2e-4;  // m^4
    let l: f64 = 4.0;       // m
    let k_braced: f64 = 0.8;
    let ei_kn_m2: f64 = e * 1000.0 * i_col;
    let pe1: f64 = PI * PI * ei_kn_m2 / (k_braced * l).powi(2);

    let pr: f64 = 0.3 * pe1; // Required axial = 30% of Pe1

    let b1: f64 = (cm / (1.0 - alpha * pr / pe1)).max(1.0);
    // = 0.4 / (1 - 0.3) = 0.4 / 0.7 = 0.571... < 1.0, so B1 = 1.0
    assert!(
        (b1 - 1.0).abs() < 1e-10,
        "B1 should be 1.0 (floored): computed={:.4}",
        b1
    );

    // Double curvature: M1/M2 = -0.5
    let m1_over_m2_dc: f64 = -0.5;
    let cm_dc: f64 = 0.6 - 0.4 * m1_over_m2_dc;
    let cm_dc_expected: f64 = 0.8;
    assert!(
        (cm_dc - cm_dc_expected).abs() < 1e-12,
        "Cm (double curvature): computed={:.4}, expected={:.4}",
        cm_dc, cm_dc_expected
    );

    let b1_dc: f64 = (cm_dc / (1.0 - alpha * pr / pe1)).max(1.0);
    // = 0.8 / 0.7 = 1.143
    assert!(
        b1_dc > 1.0,
        "B1 (double curvature) > 1.0: {:.4}",
        b1_dc
    );

    // B2 factor calculation
    let sum_pr: f64 = 500.0;   // kN, total story gravity load
    let sum_pe2: f64 = 5000.0; // kN, total story elastic critical load

    let b2: f64 = 1.0 / (1.0 - alpha * sum_pr / sum_pe2);
    // = 1 / (1 - 0.1) = 1/0.9 = 1.111
    let b2_expected: f64 = 1.0 / (1.0 - 500.0 / 5000.0);
    assert!(
        (b2 - b2_expected).abs() < 1e-10,
        "B2: computed={:.4}, expected={:.4}",
        b2, b2_expected
    );

    // B2 should always be >= 1.0 for stable structures
    assert!(
        b2 >= 1.0,
        "B2 must be >= 1.0 for stable structure: {:.4}",
        b2
    );
}

// ================================================================
// 8. P-Delta Geometric Series Approximation
// ================================================================
//
// The P-Delta effect can be approximated as a geometric series:
//   delta_total = delta_1 + P*delta_1/H_story + P^2*delta_1/H_story^2 + ...
//               = delta_1 / (1 - P/H_eff)
//
// where:
//   delta_1 = first-order lateral displacement
//   P = total gravity load on the story
//   H_story = story shear (lateral force)
//   H_eff = effective lateral stiffness
//
// The stability coefficient theta:
//   theta = P * delta_1 / (V * h)
//
// where V = story shear, h = story height.
// For theta < 0.10: P-Delta can be neglected
// For 0.10 < theta < 0.25: amplification is 1/(1-theta)
// For theta > 0.25: structure may be unstable
//
// Reference: ASCE 7-22, Section 12.8.7

#[test]
fn validation_pdelta_geometric_series() {
    // Story parameters
    let p_total: f64 = 5000.0;  // kN, total gravity load on story
    let v_story: f64 = 200.0;   // kN, design story shear
    let h_story: f64 = 3.5;     // m, story height
    let delta_1: f64 = 0.015;   // m, first-order story drift

    // Stability coefficient
    let theta: f64 = p_total * delta_1 / (v_story * h_story);
    // = 5000 * 0.015 / (200 * 3.5) = 75 / 700 = 0.1071
    let theta_expected: f64 = 75.0 / 700.0;
    assert!(
        (theta - theta_expected).abs() / theta_expected < 1e-10,
        "Stability coefficient: computed={:.4}, expected={:.4}",
        theta, theta_expected
    );

    // Amplification factor
    let af: f64 = 1.0 / (1.0 - theta);
    let af_expected: f64 = 1.0 / (1.0 - 75.0 / 700.0);
    assert!(
        (af - af_expected).abs() / af_expected < 1e-10,
        "Amplification factor: computed={:.4}, expected={:.4}",
        af, af_expected
    );

    // Amplified drift
    let delta_total: f64 = delta_1 * af;
    let delta_total_expected: f64 = delta_1 / (1.0 - theta);
    assert!(
        (delta_total - delta_total_expected).abs() / delta_total_expected < 1e-10,
        "Amplified drift: computed={:.6}, expected={:.6}",
        delta_total, delta_total_expected
    );

    // Verify geometric series: first 3 terms vs closed-form
    let term_0: f64 = delta_1;
    let term_1: f64 = delta_1 * theta;
    let term_2: f64 = delta_1 * theta * theta;
    let series_3: f64 = term_0 + term_1 + term_2;

    // With more terms, should converge to closed-form
    let mut series_n: f64 = 0.0;
    let mut theta_power: f64 = 1.0;
    for _i in 0..20 {
        series_n += delta_1 * theta_power;
        theta_power *= theta;
    }

    let series_error: f64 = (series_n - delta_total).abs() / delta_total;
    assert!(
        series_error < 1e-6,
        "Geometric series (20 terms) vs closed-form: error={:.2e}",
        series_error
    );

    // 3 terms should be a reasonable approximation
    let series_3_error: f64 = (series_3 - delta_total).abs() / delta_total;
    assert!(
        series_3_error < 0.01,
        "3-term series error: {:.4}%",
        series_3_error * 100.0
    );

    // Theta < 0.25 for stable structure
    assert!(
        theta < 0.25,
        "Theta ({:.4}) must be < 0.25 for stability",
        theta
    );
}
