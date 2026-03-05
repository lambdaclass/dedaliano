/// 2D truss local stiffness matrix (4×4).
/// DOFs: [u1, v1, u2, v2]
/// Only axial stiffness.
pub fn truss_local_stiffness_2d(e: f64, a: f64, l: f64) -> Vec<f64> {
    let ea_l = e * a / l;
    vec![
        ea_l, 0.0, -ea_l, 0.0,
        0.0,  0.0,  0.0,  0.0,
       -ea_l, 0.0,  ea_l, 0.0,
        0.0,  0.0,  0.0,  0.0,
    ]
}

/// 3D truss local stiffness matrix (6×6).
/// DOFs: [u1, v1, w1, u2, v2, w2]
/// Only axial stiffness.
pub fn truss_local_stiffness_3d(e: f64, a: f64, l: f64) -> Vec<f64> {
    let ea_l = e * a / l;
    let mut k = vec![0.0; 36];
    k[0 * 6 + 0] = ea_l;
    k[0 * 6 + 3] = -ea_l;
    k[3 * 6 + 0] = -ea_l;
    k[3 * 6 + 3] = ea_l;
    k
}

/// 2D truss global stiffness matrix directly (4×4).
/// More efficient than local + transform for trusses.
pub fn truss_global_stiffness_2d(e: f64, a: f64, l: f64, cos: f64, sin: f64) -> Vec<f64> {
    let ea_l = e * a / l;
    let c2 = cos * cos;
    let s2 = sin * sin;
    let cs = cos * sin;

    vec![
        ea_l * c2,  ea_l * cs, -ea_l * c2, -ea_l * cs,
        ea_l * cs,  ea_l * s2, -ea_l * cs, -ea_l * s2,
       -ea_l * c2, -ea_l * cs,  ea_l * c2,  ea_l * cs,
       -ea_l * cs, -ea_l * s2,  ea_l * cs,  ea_l * s2,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truss_symmetry() {
        let k = truss_global_stiffness_2d(200e6, 0.001, 5.0, 0.6, 0.8);
        for i in 0..4 {
            for j in 0..4 {
                assert!(
                    (k[i * 4 + j] - k[j * 4 + i]).abs() < 1e-6,
                    "Truss K not symmetric at ({},{})",
                    i, j
                );
            }
        }
    }

    #[test]
    fn test_truss_horizontal() {
        // Horizontal truss: cos=1, sin=0
        let k = truss_global_stiffness_2d(200e6, 0.001, 5.0, 1.0, 0.0);
        let ea_l = 200e6 * 0.001 / 5.0;
        assert!((k[0] - ea_l).abs() < 1e-6);
        assert!((k[1]).abs() < 1e-10); // No coupling
    }
}
