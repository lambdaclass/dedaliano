/// Validation: Wind Engineering Fundamentals
///
/// References:
///   - Simiu & Scanlan: "Wind Effects on Structures" 3rd ed. (1996)
///   - EN 1991-1-4 (EC1): Wind Actions on Structures
///   - ASCE 7-22 Chapter 26-31: Wind Loads
///   - Davenport (1961): "Application of Statistical Concepts to Wind Loading"
///   - Holmes: "Wind Loading of Structures" 3rd ed. (2015)
///   - ESDU Data Items: Wind Engineering Series
///
/// Tests verify wind profile, gust factor, vortex shedding,
/// across-wind response, and cladding pressure.

mod helpers;

// ================================================================
// 1. Mean Wind Profile -- Log Law & Power Law
// ================================================================
//
// Log law: V(z) = (u*/κ) × ln(z/z0)
// Power law: V(z) = V_ref × (z/z_ref)^α
// α depends on terrain: 0.11 (open) to 0.30 (urban)

#[test]
fn wind_velocity_profile() {
    let v_ref: f64 = 25.0;      // m/s at z_ref = 10m
    let z_ref: f64 = 10.0;      // m

    // Power law: open terrain
    let alpha_open: f64 = 0.14;
    // Power law: suburban terrain
    let alpha_suburban: f64 = 0.22;

    let z: f64 = 100.0;         // m, height of interest

    let v_open: f64 = v_ref * (z / z_ref).powf(alpha_open);
    let v_suburban: f64 = v_ref * (z / z_ref).powf(alpha_suburban);

    // Wind speed increases with height
    assert!(
        v_open > v_ref,
        "V(100m, open) = {:.1} > V_ref = {:.1} m/s", v_open, v_ref
    );

    // Higher α → steeper profile → more increase above reference height
    // (In reality, open terrain has higher V_ref at 10m for same gradient wind)
    assert!(
        v_suburban > v_open,
        "Steeper profile (α={:.2}): {:.1} > {:.1} m/s at 100m",
        alpha_suburban, v_suburban, v_open
    );

    // Log law check
    let z0: f64 = 0.03;         // m, roughness length (open terrain)
    let kappa: f64 = 0.41;      // von Karman constant
    let u_star: f64 = v_ref * kappa / (z_ref / z0).ln();
    let v_log: f64 = u_star / kappa * (z / z0).ln();

    assert!(
        (v_log - v_open).abs() / v_open < 0.15,
        "Log law {:.1} ≈ power law {:.1} m/s", v_log, v_open
    );
}

// ================================================================
// 2. Gust Factor -- Along-Wind Response
// ================================================================
//
// Peak factor: g ≈ √(2*ln(ν*T)) + 0.5772/√(2*ln(ν*T))
// Gust effect factor: G = 1 + 2*g*Iv*√(B² + R²)
// B = background response, R = resonant response

#[test]
fn wind_gust_factor() {
    // Turbulence intensity at building height
    let iv: f64 = 0.18;         // at z = 60m, suburban terrain

    // Background response factor
    let b_sq: f64 = 0.63;       // (depends on building dimensions)

    // Resonant response factor
    let r_sq: f64 = 0.15;       // (depends on frequency, damping)

    // Peak factor
    let nu: f64 = 0.5;          // Hz, effective crossing rate
    let t: f64 = 3600.0;        // s, averaging period (1 hour)
    let arg: f64 = 2.0 * (nu * t).ln();
    let g: f64 = arg.sqrt() + 0.5772 / arg.sqrt();

    assert!(
        g > 3.0 && g < 4.5,
        "Peak factor: {:.2}", g
    );

    // Gust effect factor (ASCE 7 simplified)
    let gf: f64 = 1.0 + 2.0 * g * iv * (b_sq + r_sq).sqrt();

    assert!(
        gf > 1.0 && gf < 3.0,
        "Gust factor: {:.2}", gf
    );

    // For rigid structures (fn > 1 Hz): resonant component negligible
    let gf_rigid: f64 = 1.0 + 2.0 * g * iv * b_sq.sqrt();
    assert!(
        gf_rigid < gf,
        "Rigid GF {:.2} < flexible GF {:.2}", gf_rigid, gf
    );
}

// ================================================================
// 3. Vortex Shedding -- Lock-In
// ================================================================
//
// Strouhal number: St = f_s × D / V
// Lock-in when f_s ≈ f_n (natural frequency)
// Critical wind speed: V_cr = f_n × D / St
// Circular cylinder: St ≈ 0.20

#[test]
fn wind_vortex_shedding() {
    let d: f64 = 0.50;          // m, cylinder diameter (chimney)
    let fn_struct: f64 = 2.0;   // Hz, natural frequency
    let st: f64 = 0.20;         // Strouhal number (circular)

    // Critical wind speed
    let v_cr: f64 = fn_struct * d / st;
    // = 2.0 * 0.5 / 0.2 = 5.0 m/s

    assert!(
        v_cr > 3.0 && v_cr < 20.0,
        "Critical speed: {:.1} m/s", v_cr
    );

    // Shedding frequency at given wind speed
    let v_wind: f64 = 15.0;     // m/s
    let f_shed: f64 = st * v_wind / d;

    assert!(
        f_shed > fn_struct,
        "f_shed = {:.1} Hz (above natural {:.1} Hz at this speed)", f_shed, fn_struct
    );

    // Reynolds number check (affects St)
    let nu_air: f64 = 1.5e-5;   // m²/s, kinematic viscosity
    let re: f64 = v_cr * d / nu_air;

    assert!(
        re > 1000.0,
        "Re = {:.0} (subcritical regime)", re
    );

    // Lock-in range: typically 0.8*V_cr to 1.2*V_cr
    let v_lock_low: f64 = 0.8 * v_cr;
    let v_lock_high: f64 = 1.2 * v_cr;
    let lock_range: f64 = v_lock_high - v_lock_low;

    assert!(
        lock_range > 0.0,
        "Lock-in range: {:.1} to {:.1} m/s", v_lock_low, v_lock_high
    );
}

// ================================================================
// 4. Pressure Coefficients -- Building
// ================================================================
//
// External pressure: pe = qp × Cpe
// Cpe depends on zone (windward, leeward, side, roof).
// EN 1991-1-4: Cpe,10 for loaded areas > 10 m²

#[test]
fn wind_pressure_coefficients() {
    // Design wind pressure
    let qp: f64 = 1.2;          // kN/m², peak velocity pressure

    // External pressure coefficients (flat roof building)
    let cpe_windward: f64 = 0.8;
    let cpe_leeward: f64 = -0.5;
    let cpe_side: f64 = -0.7;
    let cpe_roof: f64 = -1.0;   // near edge (zone F)

    // Net pressure on windward wall
    let cpi: f64 = 0.2;         // internal pressure (dominant openings)
    let p_net_windward: f64 = qp * (cpe_windward - cpi);
    let p_net_leeward: f64 = qp * (cpe_leeward - cpi);

    // Windward wall: positive net pressure (push)
    assert!(
        p_net_windward > 0.0,
        "Windward: {:.2} kN/m² (push)", p_net_windward
    );

    // Leeward wall: negative net pressure (suction)
    assert!(
        p_net_leeward < 0.0,
        "Leeward: {:.2} kN/m² (suction)", p_net_leeward
    );

    // Total horizontal force coefficient
    let cf_total: f64 = cpe_windward - cpe_leeward;
    assert!(
        cf_total > 1.0,
        "Total Cf = {:.1} (Cpe,w - Cpe,l)", cf_total
    );

    // Roof suction (critical for cladding)
    let p_roof: f64 = qp * (cpe_roof - cpi);
    assert!(
        p_roof < -1.0,
        "Roof suction: {:.2} kN/m²", p_roof
    );

    let _cpe_side = cpe_side;
}

// ================================================================
// 5. Dynamic Wind -- Across-Wind Response
// ================================================================
//
// Tall slender structures: across-wind (crosswind) response
// may exceed along-wind response.
// Crosswind force from vortex shedding and turbulence buffeting.

#[test]
fn wind_crosswind_response() {
    let h: f64 = 200.0;         // m, building height
    let b: f64 = 40.0;          // m, building width
    let d_build: f64 = 40.0;    // m, building depth

    // Aspect ratio (slenderness)
    let aspect: f64 = h / b;

    // For aspect ratio > 3: crosswind may govern
    assert!(
        aspect > 3.0,
        "H/B = {:.1} > 3 -- crosswind response important", aspect
    );

    // Generalized crosswind force spectrum peak
    // Typically at St ≈ 0.10-0.12 for rectangular buildings
    let st: f64 = 0.10;
    let v_design: f64 = 40.0;   // m/s at top
    let f_shed: f64 = st * v_design / b;

    // If shedding frequency close to natural frequency: large response
    let fn_struct: f64 = 0.20;  // Hz (approximate for 200m building)
    let freq_ratio: f64 = f_shed / fn_struct;

    assert!(
        freq_ratio > 0.0,
        "f_shed/f_n = {:.2}", freq_ratio
    );

    // Crosswind acceleration (simplified)
    // a_rms ≈ (ρ × V² × B × CL') / (2 × m × ξ)
    let rho: f64 = 1.225;       // kg/m³
    let cl_prime: f64 = 0.10;   // RMS lift coefficient
    let m: f64 = 200_000.0;     // kg/m, building mass per unit height
    let xi: f64 = 0.015;        // damping ratio

    let a_rms: f64 = rho * v_design * v_design * b * cl_prime / (2.0 * m * xi);

    assert!(
        a_rms > 0.0,
        "Crosswind acceleration: {:.4} m/s²", a_rms
    );

    let _d_build = d_build;
}

// ================================================================
// 6. Cladding Design -- Local Pressures
// ================================================================
//
// Peak local pressures much higher than area-averaged.
// EN 1991-1-4: Cpe,1 for loaded areas < 1m² (local coefficients).
// Corners and edges: Cp up to -2.0 to -3.0

#[test]
fn wind_cladding_pressures() {
    let qp: f64 = 1.5;          // kN/m², design velocity pressure

    // Local peak coefficients (EN 1991-1-4 Table 7.1)
    let cpe_1_zone_a: f64 = -1.4;  // wall corner (small area)
    let cpe_1_zone_f: f64 = -2.5;  // roof corner
    let cpe_10_zone_d: f64 = 0.8;  // windward (large area)

    // Internal pressure (worst case for cladding)
    let cpi_pos: f64 = 0.2;
    let cpi_neg: f64 = -0.3;

    // Worst suction on corner (external suction + internal pressure)
    let p_corner: f64 = qp * (cpe_1_zone_a - cpi_pos);
    // Worst positive pressure on windward (positive external - negative internal)
    let p_windward: f64 = qp * (cpe_10_zone_d - cpi_neg);

    assert!(
        p_corner < -2.0,
        "Corner suction: {:.2} kN/m²", p_corner
    );

    assert!(
        p_windward > 1.0,
        "Windward pressure: {:.2} kN/m²", p_windward
    );

    // Roof edge is most severe
    let p_roof_edge: f64 = qp * (cpe_1_zone_f - cpi_pos);
    assert!(
        p_roof_edge.abs() > p_corner.abs(),
        "Roof edge {:.2} > wall corner {:.2} kN/m² (suction)",
        p_roof_edge, p_corner
    );
}

// ================================================================
// 7. Terrain Roughness -- EC1 Categories
// ================================================================
//
// EN 1991-1-4: 5 terrain categories (0 to IV)
// z0 and zmin vary: z0 from 0.003m (sea) to 1.0m (city)
// kr = 0.19 × (z0/z0,II)^0.07

#[test]
fn wind_terrain_categories() {
    let v_b: f64 = 30.0;        // m/s, basic wind speed

    // Terrain parameters (EC1 Table 4.1)
    let terrains: [(f64, f64, &str); 4] = [
        (0.003, 1.0, "sea/coast"),      // Cat 0
        (0.05, 2.0, "open"),            // Cat II
        (0.30, 5.0, "suburban"),         // Cat III
        (1.00, 10.0, "urban"),           // Cat IV
    ];

    let z0_ii: f64 = 0.05;      // reference roughness (Category II)
    let z: f64 = 50.0;          // m, height

    let mut prev_vm: f64 = f64::MAX;

    for (z0, z_min, name) in &terrains {
        let kr: f64 = 0.19 * (z0 / z0_ii).powf(0.07);
        let z_eff: f64 = z.max(*z_min);
        let cr: f64 = kr * (z_eff / z0).ln();

        let vm: f64 = cr * v_b; // mean wind speed at z

        if *name != "sea/coast" {
            assert!(
                vm < prev_vm,
                "{}: Vm = {:.1} m/s (rougher = slower)", name, vm
            );
        }
        prev_vm = vm;
    }
}

// ================================================================
// 8. Wind Tunnel -- Pressure Tap Data Analysis
// ================================================================
//
// Wind tunnel: time series of pressure coefficients.
// Statistical analysis: mean, RMS, peak (positive/negative).
// Peak factor: gp ≈ 3.5-4.0 for Gaussian processes.

#[test]
fn wind_tunnel_analysis() {
    // Simulated pressure coefficient statistics (from wind tunnel)
    let cp_mean: f64 = -0.80;   // mean
    let cp_rms: f64 = 0.25;     // RMS fluctuation
    let gp: f64 = 3.5;          // peak factor

    // Peak negative Cp
    let cp_peak_neg: f64 = cp_mean - gp * cp_rms;
    // = -0.80 - 3.5*0.25 = -1.675

    // Peak positive Cp
    let _cp_peak_pos: f64 = cp_mean + gp * cp_rms;
    // = -0.80 + 0.875 = 0.075

    assert!(
        cp_peak_neg < cp_mean,
        "Peak suction {:.3} < mean {:.3}", cp_peak_neg, cp_mean
    );

    // Pressure from wind tunnel Cp
    let q: f64 = 1.0;           // kPa, reference dynamic pressure
    let p_mean: f64 = q * cp_mean;
    let p_peak: f64 = q * cp_peak_neg;

    assert!(
        p_peak < p_mean,
        "Peak pressure {:.3} < mean {:.3} kPa", p_peak, p_mean
    );

    // Area-averaging effect: larger areas have lower peak Cp
    let tvl: f64 = 0.75;        // TVL correction for 10m² area
    let cp_peak_10: f64 = cp_mean + tvl * gp * cp_rms;

    // Less extreme than point peak
    // (Note: both are suction, so peak_10 is less negative = less extreme)
    assert!(
        cp_peak_10.abs() < cp_peak_neg.abs(),
        "Area-averaged |{:.3}| < point |{:.3}|", cp_peak_10, cp_peak_neg
    );
}
