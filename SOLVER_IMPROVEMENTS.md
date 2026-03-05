# Solver Improvements

## Decision: Rust/WASM replaces the TypeScript solver

The TypeScript solver gets deleted. Rust owns the entire numerical pipeline — from model validation through assembly, factorization, all analysis types, internal force computation, stress analysis, and diagram evaluation. TypeScript becomes a thin presentation layer: UI, rendering, import/export, serialization.

No fallback. No maintaining two implementations. One solver, written in Rust, compiled to WASM for the browser and native for the server.

### Architecture

```
TypeScript (presentation)            Rust/WASM (all computation)
─────────────────────────            ──────────────────────────
Svelte UI + reactivity               Model validation
Model editing state                  DOF numbering + AMD/RCM reordering
3D rendering (Three.js)              Element stiffness (K, K_G, M)
Import/export (DXF, IFC, Excel)      Sparse assembly (CSC)
Serialize model → binary/JSON        Sparse Cholesky factorization
Deserialize results                  LU with partial pivoting (fallback)
Diagram rendering (SVG/Canvas)       Lanczos eigensolver
Stress visualization (color maps)    All analysis types:
User interaction                       - Linear static
                                       - P-Delta (2D + 3D)
                                       - Buckling (2D + 3D)
                                       - Modal (2D + 3D)
                                       - Plastic (2D + 3D)
                                       - Spectral (2D + 3D)
                                       - Moving loads
                                       - Influence lines
                                     Internal forces (M, V, N, T at sample points)
                                     Stress (Navier, Jourawski, Bredt, Von Mises)
                                     Kinematic analysis
                                     Condition number estimation
                                     Eventually: design code checks
```

**The WASM boundary:**

```
TS → Rust:  { nodes, elements, materials, sections, supports, loads, combinations, settings }
Rust → TS:  { displacements, reactions, diagrams[], stresses[], buckling_factors,
              mode_shapes[], frequencies[], participation_factors[],
              plastic_hinges[], kinematic_report, condition_number, warnings[] }
```

One call in, one call out. TS maps result arrays onto visualization. Rust never touches the DOM.

**Why this is right:**

1. One numerical codebase. Bugs are in Rust or in the UI — never split across a JS/WASM boundary mid-computation.
2. Same Rust binary compiles to WASM (browser) and native (server). The server compute tier is just the Rust binary behind an HTTP endpoint. No Node.js.
3. Design code checks (AISC 360, ACI 318, Eurocode) eventually go in Rust too. Server runs full analysis + design in one call.
4. No backward compatibility. The TS solver is a reference for expected results during porting, then it gets deleted.

---

## Current state

### TypeScript solver (reference implementation — will be deleted)

1,050+ tests across 31 suites. Validated against analytical solutions, Chopra textbook, and published benchmarks.

**2D (complete):** linear static, P-Delta, buckling, modal, plastic, spectral, moving loads, influence lines.
**3D (partial):** linear static with load combinations and envelopes. No advanced analysis.
**Stress:** Navier, Jourawski, Bredt/Saint-Venant torsion, Mohr's circle, Von Mises/Tresca in 2D and 3D.
**Linear algebra:** dense Cholesky + LU, Jacobi eigenvalue. All O(n³) or worse.

### Rust solver (skeleton — starting point)

3,400 LOC. Basic model types, DOF numbering, sparse CSR format (unused), dense LU only. No eigenvalue, no advanced analysis, no stress, no diagrams. ~10 tests.

### Limitations to fix

| Limitation | Impact |
|---|---|
| Dense matrices everywhere | O(n²) memory, O(n³) solve. Ceiling ~500-1,000 elements |
| Jacobi eigenvalue is O(n⁴) | Modal/buckling capped at 500 DOFs |
| 3D missing advanced analysis | No P-Delta, buckling, modal, plastic, spectral in 3D |
| Euler-Bernoulli only | Inaccurate for stocky members (depth/span > 1/10) |
| St. Venant torsion only | Wrong for open thin-walled sections |
| No matrix reordering | Uncontrolled fill-in during factorization |
| No condition number warning | Silent numerical errors on ill-conditioned models |
| Approximate diagrams | Hermite interpolation instead of exact analytical superposition |

---

## Implementation plan

Everything is built in Rust from the start. The TS solver is only used as a reference for test validation during porting.

### Phase 1 — Core linear algebra (3-4 weeks)

Build the foundation that every analysis type depends on.

#### 1.1 Sparse matrix storage (CSC)

Compressed Sparse Column format. Triple-based assembly (row, col, value triplets accumulated during element loop, then compressed). CSC is preferred over CSR for column-oriented Cholesky.

~300 LOC. Includes: triplet-to-CSC conversion, symmetric storage (lower triangle only), sparse matrix-vector multiply, sparse transpose.

**Effort:** 3-4 days.

#### 1.2 AMD/RCM reordering

Approximate Minimum Degree or Reverse Cuthill-McKee permutation to minimize fill-in before factorization. ~150-200 LOC. Textbook pseudocode.

**Effort:** 2-3 days.

#### 1.3 Sparse Cholesky factorization

Left-looking supernodal Cholesky on the CSC pattern. Two phases:

1. **Symbolic:** build elimination tree from sparsity pattern, compute fill-in, allocate factor storage. ~200 LOC.
2. **Numeric:** factorize using the precomputed pattern. ~300 LOC.

Reference: Davis, "Direct Methods for Sparse Linear Systems."

Once working: memory O(nnz), solve O(n × bandwidth²). 5,000-10,000 DOF models feasible.

**Effort:** 2-3 weeks.

#### 1.4 Sparse LU with partial pivoting

Fallback for non-SPD systems. ~400 LOC. Same symbolic/numeric split as Cholesky.

**Effort:** 1 week.

#### 1.5 Condition number estimation

1-norm condition estimator after factorization. ~50 LOC. Warns when κ(K) > 10^10.

**Effort:** 1 day.

---

### Phase 2 — Element formulations (2-3 weeks)

#### 2.1 Unified frame element (2D + 3D)

Single implementation parameterized by DOFs-per-node (3 or 6). No code duplication.

- **Euler-Bernoulli + Timoshenko:** shear correction factor κ adds 4 modified stiffness terms. Przemieniecki formulas.
- **Axial + torsion (3D):** EA/L axial, GJ/L St. Venant torsion.
- **Geometric stiffness K_G:** Przemieniecki formulation, 6x6 (2D) and 12x12 (3D). Used by P-Delta and buckling.
- **Consistent mass M:** ρAL/420 coefficients, 6x6 (2D) and 12x12 (3D) including rotational inertia. Used by modal and spectral.
- **Hinge condensation:** static condensation of moment-released DOFs. All-or-nothing per end.

~800-1,000 LOC total for the unified element.

**Effort:** 1-1.5 weeks.

#### 2.2 Truss element (2D + 3D)

Pure axial. 4x4 (2D) or 6x6 (3D). Trivial — subset of the frame element.

**Effort:** 1 day.

#### 2.3 Fixed-end forces (all load types)

Work-equivalent nodal forces for:
- Uniform distributed load
- Trapezoidal/triangular distributed load
- Point load at arbitrary position (Hermite shape functions)
- Partial distributed load (Simpson's rule or closed-form)
- Thermal load (axial + bending gradient)
- Hinge adjustment (redistribute FEF for moment releases)

~400 LOC.

**Effort:** 3-4 days.

#### 2.4 Coordinate transformation

Direction cosines from node positions + orientation vector + roll angle. Gram-Schmidt orthogonalization for 3D. Transformation T for stiffness, mass, geometric stiffness, and load vectors.

~200 LOC.

**Effort:** 2 days.

#### 2.5 Warping torsion (Vlasov)

7-DOF beam element for open thin-walled sections. Adds warping DOF (dθ/dx) per node. Warping stiffness EI_w/L terms in the local stiffness matrix. Important for lateral-torsional buckling of I-beams and channels.

~300 LOC. Affects DOF numbering and assembly (variable DOFs per node).

**Effort:** 1-1.5 weeks.

---

### Phase 3 — Assembly and linear solve (1-2 weeks)

#### 3.1 DOF numbering

Map (node_id, dof_index) → global equation number. Handle supports (fixed DOFs partitioned out), prescribed displacements, and variable DOFs per node (6 for frames, 7 for warping elements, 3 for 2D).

~200 LOC.

**Effort:** 2-3 days.

#### 3.2 Global assembly

Element loop: compute local K (and optionally K_G, M), transform to global, scatter into sparse triplet list. Compress to CSC. Apply boundary conditions by partitioning into free/fixed DOF sets.

~300 LOC.

**Effort:** 2-3 days.

#### 3.3 Linear static solve

Assemble K, apply loads (nodal + FEF), reorder (AMD), factorize (Cholesky), back-substitute for each load case. Compute reactions from full system. Support prescribed displacements via penalty method or partition approach.

~200 LOC (orchestration — the heavy lifting is in Phase 1).

**Effort:** 2-3 days.

#### 3.4 Load combinations

Apply LRFD/ASD factors to individual load case results. Compute envelopes (max/min of each result quantity across all combinations). Per-combination and per-envelope results.

~200 LOC.

**Effort:** 2 days.

---

### Phase 4 — Advanced analysis types (4-5 weeks)

All analysis types work for both 2D and 3D from the start (unified element formulation from Phase 2).

#### 4.1 P-Delta (second-order)

Iterative: solve (K + K_G)U = F where K_G is assembled from axial forces of the previous iteration. Converge when ‖ΔU‖/‖U‖ < 1e-4. Typically 3-6 iterations. Return B₂ amplification factor.

~200 LOC.

**Effort:** 3-4 days.

#### 4.2 Lanczos eigensolver

The hardest single item. Required for buckling and modal at scale.

**Implementation:**
- Basic Lanczos tridiagonalization: ~200 LOC
- Implicit restarts (IRLM): ~300 LOC
- Full re-orthogonalization (safer than partial for a first implementation): ~100 LOC
- Shift-invert mode (solve (K - σM)⁻¹Mx = θx per iteration, reuses sparse Cholesky): ~200 LOC
- Convergence check (residual-based): ~50 LOC
- Fallback: dense Jacobi for small problems (< 200 DOFs) where Lanczos overhead isn't worth it

Total: ~800-1,000 LOC.

**Alternative:** port ARPACK-NG (BSD Fortran) to Rust. Eliminates convergence risk but is tedious translation work.

**Effort:** 3-4 weeks. Wide confidence interval — could be 2 weeks or 6 weeks.

#### 4.3 Linear buckling

Generalized eigenvalue: Kφ = λ(-K_G)φ. Assemble K and K_G from a reference load case. Solve with Lanczos (shift-invert near zero). Critical load factor = smallest positive eigenvalue. Per-element effective length from K_eff·L = π√(EI/P_cr).

~150 LOC (plus Lanczos from 4.2).

**Effort:** 2-3 days.

#### 4.4 Modal analysis

Generalized eigenvalue: Kφ = ω²Mφ. Solve with Lanczos for lowest k modes. Compute:
- Natural frequencies f_n = ω_n / 2π
- Mode shapes (mass-normalized)
- Participation factors Γ_n = φ_n^T M r / (φ_n^T M φ_n) in each direction
- Effective mass ratios per mode and cumulative
- Rayleigh damping coefficients a₀, a₁ for target damping ratio

~250 LOC.

**Effort:** 3-4 days.

#### 4.5 Spectral analysis

Modal superposition with design response spectrum. For each mode: spectral acceleration S_a(T_n, ξ), modal response = Γ_n · S_a · φ_n. Combine modes via SRSS or CQC. CQC correlation coefficient ρ_ij from damping ratio and frequency ratio.

Predefined spectra: CIRSOC 103 (zones 1-4, soil types I-III). Extensible to ASCE 7 and Eurocode 8 spectra.

~300 LOC.

**Effort:** 3-4 days.

#### 4.6 Plastic collapse (event-to-event)

Incremental load. At each step: find the element/section closest to its yield surface. For 2D: M vs M_p (scalar). For 3D: biaxial interaction surface (M_y, M_z, N) — depends on section shape.

Interaction surfaces:
- I-shapes: AISC parametric equations
- Hollow sections: Von Mises-based interaction
- Circular pipes: simple Von Mises
- General sections: fiber discretization as fallback

Insert hinge at yielded section, redistribute, repeat until mechanism forms. Return collapse load multiplier and hinge sequence.

~500 LOC (interaction surfaces are the bulk).

**Effort:** 2-3 weeks.

#### 4.7 Moving loads and influence lines

Unit load traverses a path (element chain). For each position: solve, record response at target point. Envelope of all positions gives influence line. Moving load train: superposition of multiple unit load influence lines with spacing offsets.

~400 LOC.

**Effort:** 3-4 days.

---

### Phase 5 — Post-processing (2-3 weeks)

All computed in Rust. Results returned as arrays that TS renders directly.

#### 5.1 Internal force diagrams

For each element, evaluate M(x), V(x), N(x), T(x) at sample points. Use exact analytical superposition: homogeneous solution (from end displacements) + particular solution (from element loads). Not Hermite interpolation — exact for all supported load types.

~400 LOC.

**Effort:** 3-4 days.

#### 5.2 Cross-section stress

At each evaluation point along each element:
- Normal stress: σ(y,z) = N/A + M_z·y/I_z - M_y·z/I_y (Navier)
- Bending shear: τ = V·Q/(I·b) (Jourawski) per shear plane
- Torsional shear: Bredt formula (closed sections), Saint-Venant (open sections)
- Combined: Von Mises σ_vm = √(σ² + 3τ²), Tresca τ_max = (σ₁ - σ₂)/2
- Mohr's circle: principal stresses σ₁, σ₂, orientation angle

Per-section-type shear flow paths (I/H: 5 paths, U: 3, RHS: 6 with q₀ correction, circular: continuous).

~600-800 LOC.

**Effort:** 1-1.5 weeks.

#### 5.3 Kinematic analysis

Mechanism detection via rank analysis of the reduced stiffness matrix. Degree-of-freedom formula check. Diagnostic report: which DOFs are unconstrained, which elements form the mechanism.

~300 LOC.

**Effort:** 2-3 days.

#### 5.4 Deformed shape

Hermite cubic interpolation of displaced shape from nodal displacements. Sample at configurable density for smooth rendering.

~150 LOC.

**Effort:** 1-2 days.

---

### Phase 6 — WASM integration and TS deletion (2-3 weeks)

#### 6.1 WASM compilation

wasm-pack build. Expose a single entry point: `pub fn solve(model_json: &str) -> String`. JSON in, JSON out. Simple, debuggable, no shared memory complexity.

JSON is the right choice: `JSON.stringify`/`JSON.parse` are native V8 C++ and faster than any JS-based binary format (MessagePack, etc.) on the serialization side. `serde_json` on the Rust side is one derive macro. Total boundary cost is ~2ms for a 1,000-element model vs 10-10,000ms for the actual solve — serialization is < 1% of runtime.

~100 LOC for the WASM boundary.

**Effort:** 2-3 days.

#### 6.2 TypeScript integration

Replace all TS solver calls with a single `await wasmSolve(model)` call. The WASM module loads once at startup. Solve runs synchronously in a Web Worker to avoid blocking the UI thread.

~200 LOC (worker setup, message passing, result mapping).

**Effort:** 3-4 days.

#### 6.3 Test migration

Port all 1,050+ tests to Rust (`#[test]`). Each test sends the same input model and asserts the same expected outputs. The TS solver results are the reference — any discrepancy must be investigated and resolved.

Tests remain in both Rust (unit tests, run in CI) and TS/Vitest (integration tests, verify the WASM boundary).

**Effort:** 1-2 weeks.

#### 6.4 Delete the TypeScript solver

Remove: `solver-js.ts`, `solver-3d.ts`, `solver-detailed.ts`, `solver-detailed-3d.ts`, `matrix-utils.ts`, `mass-matrix.ts`, `pdelta.ts`, `buckling.ts`, `modal.ts`, `plastic.ts`, `spectral.ts`, `moving-loads.ts`, `kinematic-2d.ts`, `kinematic-3d.ts`.

Keep: `diagrams.ts` and `section-stress.ts` only if any rendering-specific interpolation stays in TS. Otherwise delete those too.

~14,000 LOC removed.

**Effort:** 1 day (plus a week of fixing anything that breaks).

---

## Summary

| Phase | What | Time | Confidence |
|---|---|---|---|
| 1 | Core linear algebra (sparse Cholesky, LU, AMD, condition #) | 3-4 weeks | 80% |
| 2 | Element formulations (frame, truss, Timoshenko, warping, K_G, M) | 2-3 weeks | 90% |
| 3 | Assembly and linear solve (DOF numbering, assembly, load combos) | 1-2 weeks | 90% |
| 4 | Advanced analysis (Lanczos, P-Delta, buckling, modal, spectral, plastic, moving loads) | 4-5 weeks | 60% |
| 5 | Post-processing (diagrams, stress, kinematic, deformed shape) | 2-3 weeks | 85% |
| 6 | WASM integration + TS deletion | 2-3 weeks | 75% |
| **Total** | | **~4-5 months** (one developer) | |

Phase 4 has the widest confidence interval because of Lanczos and 3D plastic analysis. Everything else is predictable.

---

## Sequencing and dependencies

```
Phase 1: Core linear algebra
  ├── 1.1 CSC sparse storage
  ├── 1.2 AMD/RCM reordering
  ├── 1.3 Sparse Cholesky ← (1.1, 1.2)
  ├── 1.4 Sparse LU ← (1.1)
  └── 1.5 Condition number estimation ← (1.3 or 1.4)

Phase 2: Element formulations (parallel with late Phase 1)
  ├── 2.1 Unified frame element (Euler-Bernoulli + Timoshenko)
  ├── 2.2 Truss element
  ├── 2.3 Fixed-end forces (all load types)
  ├── 2.4 Coordinate transformation
  └── 2.5 Warping torsion (Vlasov)

Phase 3: Assembly + linear solve ← (Phase 1, Phase 2)
  ├── 3.1 DOF numbering
  ├── 3.2 Global assembly
  ├── 3.3 Linear static solve
  └── 3.4 Load combinations

Phase 4: Advanced analysis ← (Phase 3)
  ├── 4.1 P-Delta
  ├── 4.2 Lanczos eigensolver ← (1.3)
  ├── 4.3 Buckling ← (4.2)
  ├── 4.4 Modal ← (4.2)
  ├── 4.5 Spectral ← (4.4)
  ├── 4.6 Plastic collapse
  └── 4.7 Moving loads + influence lines

Phase 5: Post-processing ← (Phase 3 for linear, Phase 4 for advanced)
  ├── 5.1 Internal force diagrams
  ├── 5.2 Cross-section stress
  ├── 5.3 Kinematic analysis
  └── 5.4 Deformed shape

Phase 6: WASM + deletion ← (all above)
  ├── 6.1 WASM compilation
  ├── 6.2 TS integration
  ├── 6.3 Test migration
  └── 6.4 Delete TS solver
```

Phases 1 and 2 can overlap — element formulations don't depend on sparse Cholesky. Phase 5 can start as soon as Phase 3 is done (post-processing for linear static) and grow as Phase 4 delivers each analysis type.

---

## What the solver looks like after

| Metric | Before (TS) | After (Rust/WASM) |
|---|---|---|
| Max practical DOFs | ~500 (modal), ~3,000 (static) | 50,000+ (modal), 100,000+ (static) |
| Modal/buckling eigensolver | Jacobi O(n⁴) | Lanczos O(k·nnz) |
| Linear solve | Dense Cholesky O(n³) | Sparse Cholesky O(n·bw²) |
| 3D analysis types | Linear static only | All: P-Delta, buckling, modal, spectral, plastic |
| Beam formulation | Euler-Bernoulli | Euler-Bernoulli + Timoshenko |
| Torsion | St. Venant only | St. Venant + Vlasov warping |
| Code duplication | ~4,000 LOC (2D/3D split) | Zero (unified element) |
| Diagrams | Approximate (21-point Hermite) | Exact analytical superposition |
| Condition monitoring | None | 1-norm estimate with warning |
| Server deployment | Requires Node.js | Native Rust binary, no runtime |
| Total solver LOC | ~14,600 (TS) | ~5,000-6,000 (Rust, estimated) |

The Rust solver is smaller because there's no 2D/3D duplication, no detailed/pedagogical variants (those can be rebuilt in TS as a UI feature over the WASM results), and Rust is more concise for numerical code.
