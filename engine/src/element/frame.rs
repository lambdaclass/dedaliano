/// Unified frame element stiffness matrix.
/// Works for both 2D (6×6, dofs_per_node=3) and 3D (12×12, dofs_per_node=6).

/// 2D frame local stiffness matrix (6×6).
/// DOFs: [u1, v1, θ1, u2, v2, θ2]
/// E in kN/m², A in m², Iz in m⁴, L in m.
pub fn frame_local_stiffness_2d(
    e: f64,
    a: f64,
    iz: f64,
    l: f64,
    hinge_start: bool,
    hinge_end: bool,
) -> Vec<f64> {
    let mut k = vec![0.0; 36]; // 6×6

    let ea_l = e * a / l;
    let ei = e * iz;
    let l2 = l * l;
    let l3 = l2 * l;

    if hinge_start && hinge_end {
        // Both hinges: only axial stiffness
        k[0 * 6 + 0] = ea_l;
        k[0 * 6 + 3] = -ea_l;
        k[3 * 6 + 0] = -ea_l;
        k[3 * 6 + 3] = ea_l;
        return k;
    }

    // Axial terms
    k[0 * 6 + 0] = ea_l;
    k[0 * 6 + 3] = -ea_l;
    k[3 * 6 + 0] = -ea_l;
    k[3 * 6 + 3] = ea_l;

    if !hinge_start && !hinge_end {
        // No hinges: standard beam
        let c1 = 12.0 * ei / l3;
        let c2 = 6.0 * ei / l2;
        let c3 = 4.0 * ei / l;
        let c4 = 2.0 * ei / l;

        k[1 * 6 + 1] = c1;
        k[1 * 6 + 2] = c2;
        k[1 * 6 + 4] = -c1;
        k[1 * 6 + 5] = c2;

        k[2 * 6 + 1] = c2;
        k[2 * 6 + 2] = c3;
        k[2 * 6 + 4] = -c2;
        k[2 * 6 + 5] = c4;

        k[4 * 6 + 1] = -c1;
        k[4 * 6 + 2] = -c2;
        k[4 * 6 + 4] = c1;
        k[4 * 6 + 5] = -c2;

        k[5 * 6 + 1] = c2;
        k[5 * 6 + 2] = c4;
        k[5 * 6 + 4] = -c2;
        k[5 * 6 + 5] = c3;
    } else if hinge_start {
        // Hinge at start (M1 = 0): condensed stiffness
        let c1 = 3.0 * ei / l3;
        let c2 = 3.0 * ei / l2;
        let c3 = 3.0 * ei / l;

        k[1 * 6 + 1] = c1;
        k[1 * 6 + 4] = -c1;
        k[1 * 6 + 5] = c2;

        k[4 * 6 + 1] = -c1;
        k[4 * 6 + 4] = c1;
        k[4 * 6 + 5] = -c2;

        k[5 * 6 + 1] = c2;
        k[5 * 6 + 4] = -c2;
        k[5 * 6 + 5] = c3;
    } else {
        // Hinge at end (M2 = 0): condensed stiffness
        let c1 = 3.0 * ei / l3;
        let c2 = 3.0 * ei / l2;
        let c3 = 3.0 * ei / l;

        k[1 * 6 + 1] = c1;
        k[1 * 6 + 2] = c2;
        k[1 * 6 + 4] = -c1;

        k[2 * 6 + 1] = c2;
        k[2 * 6 + 2] = c3;
        k[2 * 6 + 4] = -c2;

        k[4 * 6 + 1] = -c1;
        k[4 * 6 + 2] = -c2;
        k[4 * 6 + 4] = c1;
    }

    k
}

/// 3D frame local stiffness matrix (12×12).
/// DOFs: [u1, v1, w1, θx1, θy1, θz1, u2, v2, w2, θx2, θy2, θz2]
/// E in kN/m², A in m², Iy in m⁴, Iz in m⁴, J in m⁴, L in m.
/// G = E / (2*(1+nu)), typically nu=0.3 → G = E/2.6
pub fn frame_local_stiffness_3d(
    e: f64,
    a: f64,
    iy: f64,
    iz: f64,
    j: f64,
    l: f64,
    g: f64,
    hinge_start: bool,
    hinge_end: bool,
) -> Vec<f64> {
    let mut k = vec![0.0; 144]; // 12×12
    let n = 12;

    let ea_l = e * a / l;
    let gj_l = g * j / l;
    let l2 = l * l;
    let l3 = l2 * l;

    if hinge_start && hinge_end {
        // Both hinges: axial + torsion only
        k[0 * n + 0] = ea_l;
        k[0 * n + 6] = -ea_l;
        k[6 * n + 0] = -ea_l;
        k[6 * n + 6] = ea_l;
        k[3 * n + 3] = gj_l;
        k[3 * n + 9] = -gj_l;
        k[9 * n + 3] = -gj_l;
        k[9 * n + 9] = gj_l;
        return k;
    }

    // Axial
    k[0 * n + 0] = ea_l;
    k[0 * n + 6] = -ea_l;
    k[6 * n + 0] = -ea_l;
    k[6 * n + 6] = ea_l;

    // Torsion
    k[3 * n + 3] = gj_l;
    k[3 * n + 9] = -gj_l;
    k[9 * n + 3] = -gj_l;
    k[9 * n + 9] = gj_l;

    if !hinge_start && !hinge_end {
        // Bending in Y-Z plane (strong axis, Iz)
        // v-displacement, θz rotation
        let c1z = 12.0 * e * iz / l3;
        let c2z = 6.0 * e * iz / l2;
        let c3z = 4.0 * e * iz / l;
        let c4z = 2.0 * e * iz / l;

        k[1 * n + 1] = c1z;
        k[1 * n + 5] = c2z;
        k[1 * n + 7] = -c1z;
        k[1 * n + 11] = c2z;

        k[5 * n + 1] = c2z;
        k[5 * n + 5] = c3z;
        k[5 * n + 7] = -c2z;
        k[5 * n + 11] = c4z;

        k[7 * n + 1] = -c1z;
        k[7 * n + 5] = -c2z;
        k[7 * n + 7] = c1z;
        k[7 * n + 11] = -c2z;

        k[11 * n + 1] = c2z;
        k[11 * n + 5] = c4z;
        k[11 * n + 7] = -c2z;
        k[11 * n + 11] = c3z;

        // Bending in X-Z plane (weak axis, Iy)
        // w-displacement, θy rotation (note: sign convention θy = -dw/dx)
        let c1y = 12.0 * e * iy / l3;
        let c2y = 6.0 * e * iy / l2;
        let c3y = 4.0 * e * iy / l;
        let c4y = 2.0 * e * iy / l;

        k[2 * n + 2] = c1y;
        k[2 * n + 4] = -c2y;
        k[2 * n + 8] = -c1y;
        k[2 * n + 10] = -c2y;

        k[4 * n + 2] = -c2y;
        k[4 * n + 4] = c3y;
        k[4 * n + 8] = c2y;
        k[4 * n + 10] = c4y;

        k[8 * n + 2] = -c1y;
        k[8 * n + 4] = c2y;
        k[8 * n + 8] = c1y;
        k[8 * n + 10] = c2y;

        k[10 * n + 2] = -c2y;
        k[10 * n + 4] = c4y;
        k[10 * n + 8] = c2y;
        k[10 * n + 10] = c3y;
    } else if hinge_start {
        // Hinge at start: 3EI/L³ formulation
        let c1z = 3.0 * e * iz / l3;
        let c2z = 3.0 * e * iz / l2;
        let c3z = 3.0 * e * iz / l;

        k[1 * n + 1] = c1z;
        k[1 * n + 7] = -c1z;
        k[1 * n + 11] = c2z;
        k[7 * n + 1] = -c1z;
        k[7 * n + 7] = c1z;
        k[7 * n + 11] = -c2z;
        k[11 * n + 1] = c2z;
        k[11 * n + 7] = -c2z;
        k[11 * n + 11] = c3z;

        let c1y = 3.0 * e * iy / l3;
        let c2y = 3.0 * e * iy / l2;
        let c3y = 3.0 * e * iy / l;

        k[2 * n + 2] = c1y;
        k[2 * n + 8] = -c1y;
        k[2 * n + 10] = -c2y;
        k[8 * n + 2] = -c1y;
        k[8 * n + 8] = c1y;
        k[8 * n + 10] = c2y;
        k[10 * n + 2] = -c2y;
        k[10 * n + 8] = c2y;
        k[10 * n + 10] = c3y;
    } else {
        // Hinge at end: 3EI/L³ formulation
        let c1z = 3.0 * e * iz / l3;
        let c2z = 3.0 * e * iz / l2;
        let c3z = 3.0 * e * iz / l;

        k[1 * n + 1] = c1z;
        k[1 * n + 5] = c2z;
        k[1 * n + 7] = -c1z;
        k[5 * n + 1] = c2z;
        k[5 * n + 5] = c3z;
        k[5 * n + 7] = -c2z;
        k[7 * n + 1] = -c1z;
        k[7 * n + 5] = -c2z;
        k[7 * n + 7] = c1z;

        let c1y = 3.0 * e * iy / l3;
        let c2y = 3.0 * e * iy / l2;
        let c3y = 3.0 * e * iy / l;

        k[2 * n + 2] = c1y;
        k[2 * n + 4] = -c2y;
        k[2 * n + 8] = -c1y;
        k[4 * n + 2] = -c2y;
        k[4 * n + 4] = c3y;
        k[4 * n + 8] = c2y;
        k[8 * n + 2] = -c1y;
        k[8 * n + 4] = c2y;
        k[8 * n + 8] = c1y;
    }

    k
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_2d_symmetry() {
        let k = frame_local_stiffness_2d(200e6, 0.01, 1e-4, 3.0, false, false);
        for i in 0..6 {
            for j in 0..6 {
                assert!(
                    (k[i * 6 + j] - k[j * 6 + i]).abs() < 1e-6,
                    "K not symmetric at ({},{}): {} vs {}",
                    i, j, k[i * 6 + j], k[j * 6 + i]
                );
            }
        }
    }

    #[test]
    fn test_frame_2d_both_hinges() {
        let k = frame_local_stiffness_2d(200e6, 0.01, 1e-4, 3.0, true, true);
        // Only axial terms should be nonzero
        let ea_l = 200e6 * 0.01 / 3.0;
        assert!((k[0 * 6 + 0] - ea_l).abs() < 1e-6);
        assert!((k[3 * 6 + 3] - ea_l).abs() < 1e-6);
        // All bending terms should be zero
        assert!(k[1 * 6 + 1].abs() < 1e-10);
        assert!(k[2 * 6 + 2].abs() < 1e-10);
    }

    #[test]
    fn test_frame_3d_symmetry() {
        let k = frame_local_stiffness_3d(
            200e6, 0.01, 1e-4, 2e-4, 5e-5, 3.0,
            200e6 / 2.6, false, false,
        );
        for i in 0..12 {
            for j in 0..12 {
                assert!(
                    (k[i * 12 + j] - k[j * 12 + i]).abs() < 1e-3,
                    "K3D not symmetric at ({},{}): {} vs {}",
                    i, j, k[i * 12 + j], k[j * 12 + i]
                );
            }
        }
    }
}
