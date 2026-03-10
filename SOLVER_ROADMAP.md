# Dedaliano Solver Roadmap

## Purpose

This document is the `solver roadmap`.

Read next:
- current snapshot: [`CURRENT_STATUS.md`](/Users/unbalancedparen/projects/dedaliano/CURRENT_STATUS.md)
- current proof and capability status: [`BENCHMARKS.md`](/Users/unbalancedparen/projects/dedaliano/BENCHMARKS.md)
- verification method: [`VERIFICATION.md`](/Users/unbalancedparen/projects/dedaliano/VERIFICATION.md)

It is for:
- solver mechanics
- numerical robustness
- validation and benchmark sequencing
- verification strategy sequencing
- performance and scale work

It is not the product, market, or revenue roadmap.
For that, see [`PRODUCT_ROADMAP.md`](/Users/unbalancedparen/projects/dedaliano/PRODUCT_ROADMAP.md).

For current capability and validation status, see [`BENCHMARKS.md`](/Users/unbalancedparen/projects/dedaliano/BENCHMARKS.md).

This document should stay forward-looking.
Historical progress belongs in [`CHANGELOG.md`](/Users/unbalancedparen/projects/dedaliano/CHANGELOG.md).

## Current Frontier

The main remaining work is no longer missing basic solver categories. It is:

- shell family maturity and workflow maturity
- solver-path consistency
- diagnostics and explainability
- verification hardening
- performance and scale
- deeper reference-benchmark coverage on the newest advanced paths
- long-tail nonlinear maturity on hard real models

## What Still Separates Dedaliano From The Strongest Open Solvers

Based on the comparison against projects like OpenSees, Code_Aster, and Kratos, the remaining gaps are not “missing the basics.” They are:

1. `Shell hardening — curved/non-planar frontier`
   MITC4+EAS-7 and MITC9 are both implemented, benchmark-validated (15 benchmarks), and acceptance-covered (4 workflow models). The shell stack is no longer experimental. The remaining gap is the curved/non-planar frontier: twisted beam, Raasch hook, and hemisphere all expose flat-faceted formulation limits in both elements. The next decision is whether to stop at the well-bounded MITC4+MITC9 stack or introduce a curved/solid-shell formulation.

2. `Long-tail nonlinear maturity`
   More years of hardened edge cases are still needed in mixed nonlinear workflows:
   - contact + nonlinear + staging
   - shell + nonlinear interaction
   - difficult convergence cases

3. `Performance / scale maturity`
   Sparse-first 3D is now real, but the solver still needs stronger large-model runtime discipline, ordering quality, and broader sparse-path reuse.

4. `Full solver-path consistency`
   Dense vs sparse, constrained vs unconstrained, shell vs frame-shell mixed, and advanced nonlinear paths must keep converging to the same behavior.

5. `Benchmark moat expansion`
   Dedaliano is already strong here, but broader external-reference proof is also the most realistic path to becoming the best open structural solver.

This changes the strategic target:

- not `be broader than every open-source mechanics framework`
- but `be the strongest open structural solver product with the deepest visible proof of correctness`

## Ranked Priorities

If the goal is `best open structural solver`, the current priority order is:

1. `Shell hardening — curved/non-planar frontier`
   The MITC4+MITC9 stack is implemented and acceptance-covered. Remaining:
   - curved/non-planar frontier (twisted beam, Raasch hook, hemisphere — all flat-faceted limited)
   - decide: stop at bounded MITC4+MITC9 or introduce curved/solid-shell
   - MITC9 corotational extension (deferred)
   - distortion robustness

2. `Performance and scale`
   Turn sparse-first 3D and current performance infrastructure into real large-model runtime wins.

3. `Verification hardening`
   Keep building the proof moat with:
   - benchmark gates
   - acceptance models
   - invariants
   - property-based tests
   - fuzzing

4. `Long-tail nonlinear hardening`
   Harden the hardest mixed workflows:
   - contact + nonlinear + staging
   - shell + nonlinear interaction
   - difficult convergence edge cases

5. `Solver-path consistency`
   Keep dense vs sparse, constrained vs unconstrained, and mixed shell/frame workflows converging to the same behavior.

6. `Constraint-system maturity`
   Finish chained constraints, connector depth, eccentric workflow polish, and remaining parity gaps.

7. `Advanced contact maturity`
   Push harder convergence, richer contact laws, and tougher mixed contact states.

8. `Diagnostics, model health checks, and explainability`
   Make failures clearer, model issues easier to detect, and hard solves easier to understand.

9. `Reference benchmark expansion`
   Keep growing external-reference proof for contact, fiber 3D, SSI, creep/shrinkage, and broader shell workflows.

10. `Reduction, staged/PT coupling, and other second-tier depth`
    Mature the scale-oriented and long-term workflow layers after the core solver-quality gaps above are tighter.

## Current Sequence

The current near-term sequence is:

1. `Shell benchmark and acceptance gates`
   Keep shell benchmark and shell acceptance suites as explicit release gates, now covering both MITC4 and the implemented MITC9 path (6 MITC9 benchmarks passing).

2. `Shell-driven mechanics fixes`
   Use those gates to drive targeted fixes in:
   - load vectors
   - modal/buckling consistency
   - distortion tolerance
   - mixed tri/quad and beam-shell workflows
   - stress-recovery consistency
   - MITC9 hardening on extended curved/non-planar benchmarks

3. `Full-model performance work`
   Use acceptance models and workflow benchmarks to drive sparse, parallel, conditioning, and memory improvements on representative models.

4. `Verification hardening`
   Expand:
   - benchmark gates
   - acceptance models
   - invariants
   - property-based tests
   - fuzzing

5. `Long-tail nonlinear hardening`
   Focus on the hardest mixed cases:
   - contact + nonlinear + staging
   - shell + nonlinear interaction
   - difficult convergence edge cases

6. `Solver-path consistency and remaining maturity work`
   Finish:
   - dense vs sparse parity hardening
   - constrained vs unconstrained parity hardening
   - remaining constraint deepening
   - advanced contact maturity
   - clearer solver-side diagnostics and output semantics

## Active Programs

### 1. Shell Maturity

Focus:
- release-gated shell benchmarks (MITC4 and MITC9)
- shell load vectors
- mixed tri/quad and beam-shell workflows
- shell modal and buckling consistency
- distortion tolerance
- shell stress recovery consistency
- MITC9 hardening on extended curved/non-planar benchmarks
- MITC4-vs-MITC9 comparative benchmark tables

Current status:
- MITC9 is **implemented, benchmark-validated, and acceptance-covered**
- 15 MITC9 benchmarks passing (patch test, Navier, Scordelis-Lo, hemisphere, spherical cap, hypar, twisted beam A+B, Raasch hook, hemisphere R/t sweep)
- 4 acceptance/workflow models passing (cantilever, mixed beam+slab, cylindrical tank, modal plate)
- MITC9 outperforms MITC4 on standard benchmarks at lower mesh density (Navier 2×2: 0.98 vs MITC4 4×4: 0.93; Scordelis-Lo 2×2: 0.96 vs MITC4 6×6: 0.84)

Current remaining shell backlog:
- curved/non-planar frontier: twisted beam, Raasch hook, and hemisphere all still locked in both elements (~0.1% ratio)
- broader curved-shell workflow validation (folded plates, conical, hyperbolic)
- broader shell modal, buckling, and dynamic reference cases with MITC9
- better shell diagnostics and output semantics in solver results
- MITC9 corotational extension (deferred)

Known formulation boundary:
- MITC4+EAS-7: accurate for R/t < ~100 and flat/mildly curved shells
- MITC9: quadratic elements converge faster, extend the envelope for curved shells
- twisted beam (~0.1%), Raasch hook (~0.01%), and hemisphere (~35×) expose the flat-faceted limit in **both** elements — this is a formulation wall, not a bug
- the next decision is whether to:
  - stop at the well-bounded `MITC4 + MITC9` shell stack (likely sufficient for most structural engineering)
  - or introduce solid-shell later for composites/contact/extreme curvature

Recommended shell order:
1. decide whether the curved/non-planar frontier justifies a new element family (solid-shell or MITC with curved geometry)
2. if not, document the bounded capability and shift attention to performance/scale

Why it matters:
Shell quality is one of the clearest separators between a strong structural solver and a top-tier one.

### 2. Constraint-System Reuse and Deepening

Focus:
- consistent reuse of constrained reductions across solver families
- chained constraints
- eccentric workflow polish
- connector depth
- cross-solver parity in forces and outputs

Why it matters:
Real structural models rely heavily on diaphragms, rigid links, MPCs, and eccentric connectivity. Inconsistent constrained behavior destroys trust.

### 3. Performance and Scale

Focus:
- workflow benchmarks
- sparse and parallel wins
- conditioning diagnostics
- memory and runtime discipline on representative full models

Current status:
- sparse-first 3D assembly is live for plates, quads, and frames (models with 64+ free DOFs)
- residual-checked Cholesky fallback to dense LU catches silent ill-conditioning failures
- memory benchmarks show 11-22x reduction on representative 10×10 to 15×15 shell models
- relative pivot threshold in sparse Cholesky catches near-singular matrices earlier

Next steps:
- runtime criterion benchmarks for dense-vs-sparse 3D wall-clock comparison
- better AMD ordering (consider external crate or improved heuristic)
- parallel element loop for sparse 3D assembly (rayon behind `parallel` feature flag)
- sparse extraction for buckling/modal 3D solvers (extend sparse path beyond linear solve)

Why it matters:
A solver is not elite if it only works well on small clean examples.

### 4. Verification Hardening

Focus:
- benchmark gates
- acceptance models
- invariants
- property-based tests
- fuzzing
- differential consistency tests

Why it matters:
This is how the solver becomes visibly trustworthy rather than merely feature-rich.

### 5. Long-Tail Nonlinear Hardening

Focus:
- mixed contact + nonlinear + staged cases
- shell/nonlinear interaction hardening
- difficult convergence edge cases
- stronger fallback and failure behavior on ill-conditioned real models

Why it matters:
This is the main remaining place where mature open solvers still have more years of hardened behavior than Dedaliano.

## Exit Criteria

### Shell hardening — curved/non-planar frontier

Already done:
- MITC9 benchmarked (15 tests) and acceptance-covered (4 workflow models)
- MITC9 accepted as part of the production shell stack
- curved/non-planar benchmarks written and running (twisted beam, Raasch hook, hemisphere R/t sweep) — results document the flat-faceted limit

Remaining to close:
- the `bounded MITC4/MITC9 stack vs curved/solid-shell` decision is explicitly documented
- distortion and warp studies are gated and bounded

### Performance and scale

Done means:
- sparse 3D shows repeatable runtime wins on representative full models
- ordering is no longer an obvious known bottleneck
- large-model memory/runtime baselines are tracked
- sparse-path behavior is reliable on the intended 3D workflows

### Verification hardening

Done means:
- benchmark gates exist for the newest advanced solver families
- acceptance models cover the hardest production-style workflows
- invariants, property tests, and fuzzing exist for sparse/shell/contact/constraint paths
- benchmark discipline is part of release quality, not just local testing

### Long-tail nonlinear hardening

Done means:
- hard mixed nonlinear regressions exist and stay green
- convergence behavior is predictable on difficult reference cases
- failure modes are clearer and less solver-path-specific

### Solver-path consistency

Done means:
- dense vs sparse parity is explicitly covered on representative models
- constrained vs unconstrained parity is stable
- mixed frame/shell workflows do not diverge by solver path
- result outputs remain consistent across linear and advanced solver families

## Must-Have Vs Later

### Must-have to become the best open structural solver

- shell endgame maturity
- performance and scale
- verification hardening
- long-tail nonlinear hardening
- solver-path consistency
- constraint-system maturity

### Important after the core claim is secure

- advanced contact maturity
- broader reference benchmark expansion
- model reduction / substructuring workflow maturity
- deeper prestress / staged time-dependent coupling
- specialized shell breadth
- deterministic-behavior and explainability refinement

### Later specialization

- fire / fatigue / specialized lifecycle domains
- membranes / cable nets / specialized tensile structures
- bridge-specific advanced workflows
- broader domain expansion

## Non-Goals Right Now

- no broad multiphysics expansion
- no new specialty domains before shell, scale, verification, and nonlinear hardening are tighter
- no solver-scope expansion driven by product/UI convenience
- no feature-count work ahead of validation, robustness, and scale
- no roadmap drift into a changelog or benchmark ledger

## Related Docs

- [`README.md`](/Users/unbalancedparen/projects/dedaliano/README.md)
  repo entry point and document map
- [`BENCHMARKS.md`](/Users/unbalancedparen/projects/dedaliano/BENCHMARKS.md)
  capability and benchmark evidence
- [`VERIFICATION.md`](/Users/unbalancedparen/projects/dedaliano/VERIFICATION.md)
  verification philosophy and testing stack
- [`PRODUCT_ROADMAP.md`](/Users/unbalancedparen/projects/dedaliano/PRODUCT_ROADMAP.md)
  app, workflow, market, and product sequencing
