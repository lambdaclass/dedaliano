# Solver Improvements

## Current state

The TypeScript solver is a well-engineered, thoroughly-tested DSM implementation. 1,050+ tests across 31 suites validate it against analytical solutions, Chopra textbook benchmarks, and real structures. Equilibrium (ΣF = 0, ΣM = 0) is checked on every test.

### What works well

- **2D solver** is production-grade: linear static, P-Delta, buckling, modal, plastic, spectral, moving loads, influence lines — all complete and validated
- **3D solver** handles linear static with load combinations and envelopes
- **Stress analysis** covers Navier, Jourawski, Bredt/Saint-Venant torsion, Mohr's circle, Von Mises/Tresca in both 2D and 3D (biaxial bending + torsion)
- **Self-contained**: no external linear algebra dependencies. All algorithms readable and testable
- **Real-time**: live updates as the user edits (< 100ms for typical structures)

### Where it falls short

| Limitation | Impact |
|---|---|
| Dense matrices everywhere | Memory is O(n²), solve is O(n³). Practical ceiling ~500-1,000 elements |
| Jacobi eigenvalue solver is O(n⁴) | Modal/buckling hard-capped at 500 DOFs |
| 3D missing advanced analysis | No P-Delta, buckling, modal, plastic, or spectral in 3D |
| 2D/3D code duplication | ~4,000 LOC of near-identical logic across solver-js.ts, solver-3d.ts, and their detailed variants |
| Euler-Bernoulli beams only | No shear deformation — inaccurate for stocky members (depth/span > 1/10) |
| St. Venant torsion only | No warping torsion — wrong for open thin-walled sections (I-beams, channels) |
| Rust/WASM solver incomplete | 3,400 LOC skeleton with basic LU only. No eigenvalue, no P-Delta, no sparse solve |
| No matrix reordering | Fill-in during factorization is uncontrolled — matters once plates/shells land |
| No condition number warning | Users get silently bad results from ill-conditioned models |
| Approximate internal force diagrams | Hermite interpolation with 21 sample points instead of exact analytical superposition |

---

## Improvement plan

### Phase A — Easy wins (2-3 weeks)

Textbook algorithms, low risk, high confidence. Each item is self-contained.

#### A.1 Reverse Cuthill-McKee / AMD reordering

~150 LOC. Reorder DOFs before factorization to minimize bandwidth and fill-in. The algorithms are pseudocode in every sparse matrix textbook. Plug in before factorization. Becomes critical once plates/shells introduce irregular mesh connectivity.

**Effort:** 2-3 days.

#### A.2 Timoshenko beam elements

Modify 4 bending-related entries in the local stiffness matrix to include shear deformation via a shear correction factor κA. Add shear area as a section property. Formulas are in Przemieniecki. Improves accuracy for deep beams, short columns, and composite sections (depth/span > 1/10).

**Effort:** 2-3 days.

#### A.3 Condition number estimation

A 1-norm condition estimator after Cholesky is ~30 LOC. `lu.rs` already has a `condition_estimate` function that's never called. Warn users when their model has condition number > 10^10: "results may be unreliable — check for nearly-coplanar elements or very stiff springs."

**Effort:** 1 day.

#### A.4 Exact internal force diagrams

Currently: Hermite interpolation with 21 sample points per element.
Better: superimpose the analytical load contribution (known closed-form for uniform, triangular, point loads) on the homogeneous cubic solution. Gives exact M(x), V(x), N(x) with fewer evaluation points.

**Effort:** 2-3 days.

#### A.5 3D geometric stiffness matrix K_G

Direct extension of the 2D Przemieniecki formulation to 12x12. The 2D version in `pdelta.ts` is the template. This unblocks 3D P-Delta and 3D buckling.

**Effort:** 3-4 days.

#### A.6 3D consistent mass matrix

Already partially implemented. Extend the 6x6 consistent mass (ρAL/420 coefficients) to the full 12x12 including rotational inertia and torsional mass. Textbook formula. Unblocks 3D modal analysis.

**Effort:** 2-3 days.

---

### Phase B — Medium difficulty (2-3 months)

Substantial but predictable work. Known algorithms, effort is in careful implementation and testing.

#### B.1 Sparse Cholesky factorization

CSR/CSC storage + symbolic factorization (elimination tree to compute fill-in pattern) + left-looking column Cholesky on the sparse pattern. ~500-700 LOC.

The tricky part is the symbolic phase — computing which entries fill in during factorization without doing the actual arithmetic. Requires building an elimination tree from the sparsity pattern. Well-documented in Davis, "Direct Methods for Sparse Linear Systems."

Once working:
- Memory drops from O(n²) to O(nnz)
- Solve time drops from O(n³) to O(n × bandwidth²)
- 5,000-10,000 DOF models become feasible in the browser

**Effort:** 2-3 weeks.

#### B.2 2D/3D solver unification

Refactor ~4,000 LOC of duplicated logic into a unified solver parameterized by DOFs-per-node (3 or 6). Extract DOF numbering, assembly, and post-processing into shared modules. The detailed solver variants add another ~1,700 LOC of duplication.

Not algorithmically hard, but every one of the 1,050+ tests must still pass after. Best done incrementally:
1. Extract DOF numbering → shared module
2. Extract assembly loop → shared module
3. Extract post-processing → shared module
4. Unify solver entry points

**Effort:** 2-3 weeks.

#### B.3 3D advanced analysis types

Four analysis types, each 3-5 days. The 2D implementations are the template.

| Analysis | What changes from 2D | Depends on |
|---|---|---|
| **3D P-Delta** | Use 12x12 K_G (from A.5). Same iterative (K + K_G)U = F loop | A.5 |
| **3D buckling** | Generalized eigenvalue with 3D K_G. Same solver | A.5, eigenvalue solver |
| **3D modal** | 12x12 consistent mass (from A.6). Participation factors in X, Y, Z directions | A.6, eigenvalue solver |
| **3D spectral** | 3D modal superposition. Combine responses in three directions. CQC/SRSS unchanged | 3D modal |

The test writing takes as long as the implementation — each needs validation against published 3D benchmarks.

**Effort:** 3-4 weeks total.

#### B.4 Warping torsion (Vlasov)

For open thin-walled sections (I-beams, channels), warping torsion (EI_w) dominates over St. Venant for short members. Two approaches:

- **7-DOF element**: adds warping DOF, changes DOF numbering scheme, ripples through assembly/post-processing/diagrams. More general, more invasive.
- **Simplified bimoment approach**: adds warping stiffness without a new DOF. Less general, less invasive.

Important for lateral-torsional buckling calculations in the upcoming steel design code checks (AISC 360 Chapter F, Eurocode 3).

**Effort:** 1-2 weeks.

---

### Phase C — Hard (3-4 months)

Real engineering challenges. Wide confidence intervals. These are the items where you discover problems mid-implementation that weren't obvious at the start.

#### C.1 Lanczos eigensolver

The basic Lanczos algorithm is simple (~200 LOC). Making it robust is not.

**Required for production quality:**
- **Implicit restarts** — control memory by restarting with a refined starting vector after k steps. Without this, memory grows linearly with iteration count
- **Re-orthogonalization** — loss of orthogonality is the #1 failure mode of Lanczos. Need either full re-orthogonalization (expensive) or partial/selective (complex bookkeeping)
- **Shift-invert mode** — to find eigenvalues near a target (e.g., "eigenvalues closest to zero" for buckling). Requires solving (K - σM)⁻¹Mx = θx at each iteration, which means a sparse factorization of the shifted matrix
- **Convergence criteria** — when are the Ritz values accurate enough? Residual-based criteria with careful tolerances

A production-quality implementation is ~1,000-1,500 LOC with extensive edge-case handling.

**Alternative:** port ARPACK-NG (BSD-licensed Fortran). Translating to Rust or TypeScript is tedious but eliminates the "getting convergence right" risk.

**Combined with sparse Cholesky, this pushes the modal/buckling limit from 500 to 50,000+ DOFs.**

**Effort:** 3-4 weeks. Could be 2 weeks if lucky with convergence, 6 weeks if fighting re-orthogonalization bugs on edge-case matrices.

#### C.2 Rust/WASM solver completion

The Rust solver is a skeleton — basic LU only, no eigenvalue, no advanced analysis. The work:

1. Port sparse Cholesky to Rust (~500 LOC)
2. Port Lanczos eigensolver to Rust (~1,000 LOC)
3. Port all advanced analysis types (P-Delta, buckling, modal, plastic, spectral)
4. WASM compilation via wasm-pack
5. JS↔WASM data boundary — pass sparse matrices without serialization overhead (SharedArrayBuffer or careful memory management)
6. Match every TypeScript test result

The WASM boundary is particularly annoying. You need to efficiently transfer large Float64Arrays between JS and WASM without copying. SharedArrayBuffer works but has cross-origin isolation requirements that affect deployment.

Then you maintain two implementations (TS for fallback, Rust for performance) until confident enough to drop one.

**Expected speedup:** 10-50x for large eigenvalue problems. WASM is ~1-2x native speed, vs JavaScript being 5-20x slower for tight numerical loops.

**Effort:** 6-8 weeks. Could be 4 weeks if the WASM boundary cooperates, 10 weeks if SharedArrayBuffer causes cross-browser issues.

#### C.3 3D plastic analysis

The 2D version checks a scalar: M vs M_p. The 3D version needs a biaxial interaction surface (M_y, M_z, N) — detecting when a section reaches its yield surface in 3D stress space.

**The hard part:** the interaction surface depends on section shape:
- I-shapes: AISC provides parametric equations, but they're approximate
- Hollow sections: different interaction shape entirely
- Circular pipes: simple (Von Mises), but different from I-shapes
- General sections: must discretize the cross-section into fibers and numerically compute the yield surface

The event-to-event algorithm then tracks which combination of internal forces triggered yielding and at which element. This is a combinatorial problem that grows with model size.

**Effort:** 2-3 weeks.

---

## Summary

| Phase | What | Calendar time | Confidence |
|---|---|---|---|
| **A** Easy wins | Reordering, Timoshenko, condition #, exact diagrams, 3D K_G, 3D mass | 2-3 weeks | 95% |
| **B** Medium | Sparse Cholesky, solver unification, 3D advanced analysis, warping torsion | 2-3 months | 80% |
| **C** Hard | Lanczos eigensolver, Rust/WASM, 3D plastic | 3-4 months | 60% |
| **Total** | | **~6-8 months** (one developer) | |

The Hard items have wide confidence intervals. Lanczos might take 2 weeks or 6 weeks depending on convergence behavior on edge-case matrices. Rust/WASM might take 4 weeks or 10 weeks depending on the JS↔WASM boundary. These estimates could easily be 50% low.

---

## Recommended sequencing

```
Phase A (Easy wins)
  ├── A.1 RCM reordering
  ├── A.2 Timoshenko beams
  ├── A.3 Condition number estimation
  ├── A.4 Exact internal force diagrams
  ├── A.5 3D geometric stiffness ──────────┐
  └── A.6 3D consistent mass ─────────┐    │
                                       │    │
Phase B (Medium)                       │    │
  ├── B.1 Sparse Cholesky             │    │
  ├── B.2 Solver unification (2D/3D)  │    │
  ├── B.3 3D advanced analysis ←──────┘────┘
  │     ├── 3D P-Delta (needs A.5)
  │     ├── 3D buckling (needs A.5 + eigensolver)
  │     ├── 3D modal (needs A.6 + eigensolver)
  │     └── 3D spectral (needs 3D modal)
  └── B.4 Warping torsion
                │
Phase C (Hard)  │
  ├── C.1 Lanczos eigensolver (unblocks B.3 buckling/modal at scale)
  ├── C.2 Rust/WASM (optimization — TS solver works, just slower)
  └── C.3 3D plastic analysis
```

Do the Easy items first — immediate value, almost no risk. Then sparse Cholesky + Lanczos (biggest performance impact). Then 3D advanced analysis (unblocks Roadmap Phase 1.1). Leave Rust/WASM for last — it's an optimization, not a feature.

---

## What these improvements unlock

| Before | After |
|---|---|
| ~500 DOF ceiling for modal/buckling | 50,000+ DOFs with sparse Lanczos |
| ~1,000 element practical limit | 5,000-10,000 elements with sparse Cholesky |
| 3D linear static only | Full 3D: P-Delta, buckling, modal, spectral, plastic |
| Euler-Bernoulli only | Timoshenko for stocky members |
| St. Venant torsion only | Warping torsion for open sections |
| Silent numerical errors | Condition number warnings |
| Approximate force diagrams | Exact analytical diagrams |
| JS-only computation | Rust/WASM for 10-50x speedup on large models |

After all phases, the solver handles real multi-story 3D buildings with thousands of elements and dozens of load combinations — competitive with ETABS/SAP2000's analysis capabilities, running in the browser.
