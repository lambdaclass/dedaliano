use crate::types::*;

/// Compute fixed-end forces (FEF) from a prestress load on a 2D frame element.
///
/// Returns a 6-element vector [fx_i, fy_i, mz_i, fx_j, fy_j, mz_j] in local coords.
///
/// Sign convention:
/// - Positive force = compression on concrete (tension in tendon)
/// - Positive eccentricity = below centroid → causes sagging (positive) moment
pub fn prestress_fef_2d(ps: &PrestressLoad, length: f64) -> [f64; 6] {
    let p = ps.force;
    let mut fef = [0.0f64; 6];

    // Axial compression
    fef[0] = -p;
    fef[3] = p;

    match &ps.profile {
        TendonProfile::Straight => {
            fef[2] = p * ps.eccentricity_i;
            fef[5] = -p * ps.eccentricity_j;
        }
        TendonProfile::Parabolic { e_mid } => {
            let e_chord_mid = (ps.eccentricity_i + ps.eccentricity_j) / 2.0;
            let net_sag = e_mid - e_chord_mid;
            let l2 = length * length;
            let w_eq = 8.0 * p * net_sag / l2;

            fef[1] = w_eq * length / 2.0;
            fef[2] = w_eq * l2 / 12.0;
            fef[4] = w_eq * length / 2.0;
            fef[5] = -w_eq * l2 / 12.0;

            fef[2] += p * ps.eccentricity_i;
            fef[5] -= p * ps.eccentricity_j;
        }
        TendonProfile::Harped { e_harp } => {
            // Harped tendon: linear eccentricity from e_i to e_harp at midspan,
            // then from e_harp to e_j. Equivalent to two point loads at the harp point.
            //
            // The angular change at midspan produces a concentrated transverse force:
            //   P_y = P * (slope_left + slope_right) at L/2
            //
            // slope_left  = (e_harp - e_i) / (L/2)
            // slope_right = (e_harp - e_j) / (L/2)
            let half_l = length / 2.0;
            let slope_left = (e_harp - ps.eccentricity_i) / half_l;
            let slope_right = (e_harp - ps.eccentricity_j) / half_l;
            let p_y = p * (slope_left + slope_right);

            // Point load at midspan on fixed-fixed beam:
            // V_i = V_j = P_y/2, M_i = P_y*L/8, M_j = -P_y*L/8
            fef[1] = p_y / 2.0;
            fef[2] = p_y * length / 8.0;
            fef[4] = p_y / 2.0;
            fef[5] = -p_y * length / 8.0;

            // Add end moments from eccentricity at supports
            fef[2] += p * ps.eccentricity_i;
            fef[5] -= p * ps.eccentricity_j;
        }
    }

    // Apply friction losses if specified
    if let (Some(mu), Some(kappa)) = (ps.mu, ps.kappa) {
        let alpha = match &ps.profile {
            TendonProfile::Straight => {
                let de = (ps.eccentricity_j - ps.eccentricity_i).abs();
                (de / length).atan()
            }
            TendonProfile::Parabolic { e_mid } => {
                let e_chord_mid = (ps.eccentricity_i + ps.eccentricity_j) / 2.0;
                let net_sag = (*e_mid - e_chord_mid).abs();
                8.0 * net_sag / length
            }
            TendonProfile::Harped { e_harp } => {
                let half_l = length / 2.0;
                let slope_l = ((e_harp - ps.eccentricity_i) / half_l).atan();
                let slope_r = ((e_harp - ps.eccentricity_j) / half_l).atan();
                slope_l.abs() + slope_r.abs()
            }
        };

        let loss_factor = (-mu * alpha - kappa * length).exp();
        let avg_loss = (1.0 + loss_factor) / 2.0;
        for v in fef.iter_mut() {
            *v *= avg_loss;
        }
    }

    fef
}

/// Compute fixed-end forces from a prestress load on a 3D frame element.
///
/// Returns a 12-element vector in local coords:
/// [fx_i, fy_i, fz_i, mx_i, my_i, mz_i, fx_j, fy_j, fz_j, mx_j, my_j, mz_j]
///
/// Extends 2D approach by adding vertical (Z) eccentricity.
/// `ecc_y_i/j`: eccentricity in local Y direction at nodes I, J
/// `ecc_z_i/j`: eccentricity in local Z direction at nodes I, J
pub fn prestress_fef_3d(
    p: f64,
    ecc_y_i: f64,
    ecc_y_j: f64,
    ecc_z_i: f64,
    ecc_z_j: f64,
    length: f64,
    profile: &TendonProfile,
) -> [f64; 12] {
    let mut fef = [0.0f64; 12];

    // Axial compression
    fef[0] = -p;
    fef[6] = p;

    match profile {
        TendonProfile::Straight => {
            // Y-eccentricity → moment about Z axis
            fef[5] = p * ecc_y_i;
            fef[11] = -p * ecc_y_j;
            // Z-eccentricity → moment about Y axis (negative sign: θy = -dw/dx)
            fef[4] = -p * ecc_z_i;
            fef[10] = p * ecc_z_j;
        }
        TendonProfile::Parabolic { e_mid } => {
            // For 3D parabolic, e_mid applies to Y-direction
            let e_chord_mid = (ecc_y_i + ecc_y_j) / 2.0;
            let net_sag = e_mid - e_chord_mid;
            let l2 = length * length;
            let w_eq_y = 8.0 * p * net_sag / l2;

            // Y-direction FEF
            fef[1] = w_eq_y * length / 2.0;
            fef[5] = w_eq_y * l2 / 12.0;
            fef[7] = w_eq_y * length / 2.0;
            fef[11] = -w_eq_y * l2 / 12.0;

            // End moments from Y eccentricity
            fef[5] += p * ecc_y_i;
            fef[11] -= p * ecc_y_j;

            // Z-direction: straight tendon behavior
            fef[4] = -p * ecc_z_i;
            fef[10] = p * ecc_z_j;
        }
        TendonProfile::Harped { e_harp } => {
            // Harped in Y-direction
            let half_l = length / 2.0;
            let slope_l = (e_harp - ecc_y_i) / half_l;
            let slope_r = (e_harp - ecc_y_j) / half_l;
            let p_y = p * (slope_l + slope_r);

            fef[1] = p_y / 2.0;
            fef[5] = p_y * length / 8.0;
            fef[7] = p_y / 2.0;
            fef[11] = -p_y * length / 8.0;

            fef[5] += p * ecc_y_i;
            fef[11] -= p * ecc_y_j;

            // Z-direction: straight
            fef[4] = -p * ecc_z_i;
            fef[10] = p * ecc_z_j;
        }
    }

    fef
}

/// Anchorage set (seating) loss calculation.
///
/// When the tendon is locked off, the wedge slip at the anchor causes
/// a loss that propagates back along the tendon until it meets the
/// friction profile.
///
/// Returns the force loss at the jacking end (kN).
///
/// `p_jack`: jacking force (kN)
/// `mu`: friction coefficient
/// `kappa`: wobble coefficient (1/m)
/// `l_set`: anchorage set slip (mm, typically 6-8mm for post-tensioning)
/// `e_ps`: tendon elastic modulus (MPa)
/// `a_ps`: tendon area (m²)
/// `length`: total tendon length (m)
/// `alpha_total`: total angular change over tendon length (rad)
pub fn anchorage_set_loss(
    p_jack: f64,
    mu: f64,
    kappa: f64,
    l_set: f64,
    e_ps: f64,
    a_ps: f64,
    length: f64,
    alpha_total: f64,
) -> f64 {
    // Slope of friction loss per unit length
    let p_slope = p_jack * (mu * alpha_total / length + kappa);

    if p_slope < 1e-15 {
        return 0.0;
    }

    // Set distance: distance affected by anchorage set
    // l_set_dist = sqrt(Δs * E_ps * A_ps / p_slope)
    let delta_s = l_set / 1000.0; // mm → m
    let l_set_dist = (delta_s * e_ps * 1000.0 * a_ps / p_slope).sqrt();

    // Loss at the anchor
    if l_set_dist >= length {
        // Set loss extends beyond tendon — entire tendon affected
        2.0 * p_slope * length
    } else {
        2.0 * p_slope * l_set_dist
    }
}

/// Time-dependent prestress losses per EC2/ACI approach.
///
/// Returns total long-term loss (MPa) from creep, shrinkage, and relaxation.
///
/// `t_days`: time since loading (days)
/// `rh`: relative humidity (%, e.g. 70)
/// `fc`: concrete compressive strength at 28 days (MPa)
/// `fci`: concrete compressive strength at transfer (MPa)
/// `f_pi`: initial tendon stress after transfer (MPa)
/// `f_py`: tendon yield stress (MPa)
/// `e_ps`: tendon elastic modulus (MPa)
/// `e_ci`: concrete modulus at transfer (MPa)
/// `is_low_relax`: low-relaxation strand?
/// `h_0`: notional size = 2*Ac/u (mm), where Ac=area, u=perimeter
pub fn time_dependent_losses(
    t_days: f64,
    rh: f64,
    fc: f64,
    fci: f64,
    f_pi: f64,
    f_py: f64,
    e_ps: f64,
    e_ci: f64,
    is_low_relax: bool,
    h_0: f64,
) -> f64 {
    // --- Creep loss (EC2 approach) ---
    // Creep coefficient φ(t,t0)
    let beta_h = {
        let alpha = if fc <= 35.0 { 1.0 } else { (35.0 / fc).powf(0.7) };
        let h_val = h_0.min(1500.0 * alpha);
        1.5 * (1.0 + (0.012 * rh).powi(18)) * h_val + 250.0 * alpha
    };
    let t0 = 7.0_f64; // assume loading at 7 days
    let beta_c = ((t_days - t0) / (beta_h + t_days - t0)).powf(0.3);

    let phi_rh = if rh < 99.0 {
        let alpha_1 = (35.0 / fc).powf(0.7);
        let alpha_2 = (35.0 / fc).powf(0.2);
        (1.0 + (1.0 - rh / 100.0) / (0.1 * h_0.powf(1.0 / 3.0)) * alpha_1) * alpha_2
    } else {
        1.0
    };
    let beta_fcm = 16.8 / fc.sqrt();
    let beta_t0 = 1.0 / (0.1 + t0.powf(0.20));
    let phi_0 = phi_rh * beta_fcm * beta_t0;
    let phi_t = phi_0 * beta_c;

    // Creep loss: Δσ_cr = E_ps/E_ci * φ(t,t0) * σ_cgp
    // σ_cgp = concrete stress at tendon CG ≈ f_pi * A_ps / A_c
    // For a typical prestress ratio, σ_cgp ≈ 0.3 * fci to 0.5 * fci
    // Simplified: σ_cgp ≈ fci * 0.45 (typical for prestressed members)
    let n_ratio = e_ps / e_ci;
    let sigma_cgp = fci * 0.45;
    let loss_creep = n_ratio * phi_t * sigma_cgp;

    // --- Shrinkage loss ---
    // EC2 drying shrinkage
    let eps_cd0 = {
        let alpha_ds1 = if fc <= 40.0 { 4.0 } else { 3.0 };
        let alpha_ds2 = 0.13;
        let f_cm0 = 10.0; // MPa reference
        let beta_rh = 1.55 * (1.0 - (rh / 100.0).powi(3));
        alpha_ds1 * alpha_ds2 * (-0.1 * fc / f_cm0).exp() * beta_rh * 1e-3
    };
    let beta_ds = {
        let t_s = 3.0; // curing period (days)
        let num = t_days - t_s;
        if num > 0.0 {
            num / (num + 0.04 * h_0.powf(1.5))
        } else {
            0.0
        }
    };
    let eps_sh = eps_cd0 * beta_ds;
    let loss_shrinkage = eps_sh * e_ps;

    // --- Relaxation loss ---
    let ratio = f_pi / f_py;
    let rho_1000 = if is_low_relax { 0.025 } else { 0.08 }; // Class 2 vs Class 1
    let t_hours = t_days * 24.0;
    let loss_relaxation = if ratio > 0.5 {
        f_pi * rho_1000 * (t_hours / 1000.0).powf(0.75 * (1.0 - ratio)) * ((ratio - 0.55).max(0.0) / 0.45)
    } else {
        0.0
    };

    loss_creep + loss_shrinkage + loss_relaxation
}

/// Compute friction loss along a tendon at distance x from jacking end.
///
/// P(x) = P_jack * exp(-μ*α(x) - κ*x)
pub fn tendon_force_at(
    p_jack: f64,
    mu: f64,
    kappa: f64,
    alpha_cumulative: f64,
    x: f64,
) -> f64 {
    p_jack * (-mu * alpha_cumulative - kappa * x).exp()
}

/// Compute elastic shortening loss for pretensioned members.
///
/// Δf_pES = n * f_cgp
pub fn elastic_shortening_loss(
    e_ps: f64,
    e_ci: f64,
    f_cgp: f64,
) -> f64 {
    let n = e_ps / e_ci;
    n * f_cgp
}

/// ACI 318 lump-sum prestress losses (approximate).
///
/// Returns total long-term losses in MPa.
pub fn aci_lump_sum_losses(
    _f_pi: f64,
    is_low_relax: bool,
) -> f64 {
    if is_low_relax { 240.0 } else { 310.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_tendon_fef() {
        let ps = PrestressLoad {
            element_id: 1,
            force: 1000.0,
            eccentricity_i: 0.1,
            eccentricity_j: 0.1,
            profile: TendonProfile::Straight,
            mu: None,
            kappa: None,
        };
        let fef = prestress_fef_2d(&ps, 10.0);

        assert!((fef[0] - (-1000.0)).abs() < 1e-10);
        assert!((fef[3] - 1000.0).abs() < 1e-10);
        assert!((fef[2] - 100.0).abs() < 1e-10);
        assert!((fef[5] - (-100.0)).abs() < 1e-10);
    }

    #[test]
    fn parabolic_tendon_fef() {
        let ps = PrestressLoad {
            element_id: 1,
            force: 1000.0,
            eccentricity_i: 0.0,
            eccentricity_j: 0.0,
            profile: TendonProfile::Parabolic { e_mid: 0.2 },
            mu: None,
            kappa: None,
        };
        let l = 10.0;
        let fef = prestress_fef_2d(&ps, l);

        assert!((fef[1] - 80.0).abs() < 1e-10);
        assert!((fef[4] - 80.0).abs() < 1e-10);
        assert!((fef[2] - 133.333333333).abs() < 0.01);
        assert!((fef[5] - (-133.333333333)).abs() < 0.01);
    }

    #[test]
    fn harped_tendon_fef() {
        // Harped tendon: e_i = e_j = 0, e_harp = 0.2m at midspan on 10m beam
        let ps = PrestressLoad {
            element_id: 1,
            force: 1000.0,
            eccentricity_i: 0.0,
            eccentricity_j: 0.0,
            profile: TendonProfile::Harped { e_harp: 0.2 },
            mu: None,
            kappa: None,
        };
        let l = 10.0;
        let fef = prestress_fef_2d(&ps, l);

        // slope_l = slope_r = 0.2 / 5 = 0.04
        // P_y = 1000 * (0.04 + 0.04) = 80 kN (point load at midspan)
        assert!((fef[1] - 40.0).abs() < 1e-10); // V_i = P_y/2
        assert!((fef[4] - 40.0).abs() < 1e-10); // V_j = P_y/2
        assert!((fef[2] - 100.0).abs() < 1e-10); // M_i = P_y*L/8 = 80*10/8 = 100
    }

    #[test]
    fn prestress_fef_3d_straight() {
        let fef = prestress_fef_3d(
            1000.0, 0.1, 0.1, 0.05, 0.05, 10.0,
            &TendonProfile::Straight,
        );
        assert!((fef[0] - (-1000.0)).abs() < 1e-10);
        assert!((fef[6] - 1000.0).abs() < 1e-10);
        // Y eccentricity → Mz
        assert!((fef[5] - 100.0).abs() < 1e-10);
        // Z eccentricity → My (negative sign)
        assert!((fef[4] - (-50.0)).abs() < 1e-10);
    }

    #[test]
    fn anchorage_set_loss_basic() {
        let loss = anchorage_set_loss(
            1500.0, 0.2, 0.002, 6.0, 195_000.0, 0.001, 30.0, 0.3,
        );
        assert!(loss > 0.0);
        assert!(loss < 1500.0); // loss < jacking force
    }

    #[test]
    fn time_dependent_losses_basic() {
        let loss = time_dependent_losses(
            10000.0,  // 10000 days (~27 years)
            70.0,     // 70% RH
            40.0,     // fc = 40 MPa
            30.0,     // fci = 30 MPa
            1200.0,   // f_pi = 1200 MPa
            1860.0,   // f_py = 1860 MPa
            195_000.0, // E_ps
            30_000.0,  // E_ci
            true,      // low-relaxation
            200.0,     // h_0 = 200mm
        );
        // Typical long-term loss should be 200-400 MPa
        assert!(loss > 50.0, "loss={}", loss);
        assert!(loss < 800.0, "loss={}", loss);
    }

    #[test]
    fn friction_loss() {
        let p = tendon_force_at(1000.0, 0.2, 0.002, 0.1, 20.0);
        let expected = 1000.0 * (-0.2_f64 * 0.1 - 0.002 * 20.0).exp();
        assert!((p - expected).abs() < 0.01);
    }
}
