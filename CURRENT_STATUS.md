# Current Status

This file is the short project snapshot.

This is the `canonical status snapshot` for the repo-level docs.
If an exact top-level test count or current-status sentence needs to live anywhere, it should live here first.

For the proof and detailed capability matrix, see [`BENCHMARKS.md`](/Users/unbalancedparen/projects/dedaliano/BENCHMARKS.md).
For sequencing, see [`SOLVER_ROADMAP.md`](/Users/unbalancedparen/projects/dedaliano/SOLVER_ROADMAP.md).

Read next:
- proof and evidence: [`BENCHMARKS.md`](/Users/unbalancedparen/projects/dedaliano/BENCHMARKS.md)
- next solver work: [`SOLVER_ROADMAP.md`](/Users/unbalancedparen/projects/dedaliano/SOLVER_ROADMAP.md)
- product execution: [`PRODUCT_ROADMAP.md`](/Users/unbalancedparen/projects/dedaliano/PRODUCT_ROADMAP.md)

## Solver Snapshot

Latest reported status:

- `5897` passing tests, `0` failures
- explicit CI gate stages for shell benchmarks, shell acceptance models, and constraint benchmarks
- broad 2D and 3D structural analysis coverage
- nonlinear, staged, contact, SSI, fiber, imperfections, and creep/shrinkage support
- strong benchmark, acceptance-model, integration, and differential/parity coverage

At a high level, Dedaliano already has:

- 2D and 3D linear, second-order, buckling, modal, spectrum, time history, and harmonic analysis
- nonlinear frame, fiber, contact, SSI, staged, prestress, imperfections, and creep/shrinkage workflows
- triangular plates and a multi-family shell stack: MITC4, MITC9, and SHB8-ANS
- constraint systems, reduction/substructuring, and broad postprocessing/design modules
- a browser-native product surface on top of the solver

That same solver surface can support multiple user layers:
- engineering firms
- students and professors
- design-build / temporary works workflows
- BIM / computational design users
- later, a guardrailed conceptual mode for architects

## Strongest Areas

- broad structural analysis coverage
- unusually visible benchmark and validation discipline
- strong product surface for an open solver project
- multi-family shell stack: MITC4 (ANS + EAS-7), MITC9 (9-node, ANS shear tying), and SHB8-ANS solid-shell, benchmark-validated and acceptance-covered
- sparse-first 3D path with dense-vs-sparse parity coverage and significant memory reduction on shell models

## Main Remaining Gaps

The biggest remaining gaps are no longer basic solver categories. They are:

- performance and scale
  broader sparse-path runtime wins and large-model discipline
- shell-family hardening
  MITC4, MITC9, and SHB8-ANS are all implemented; remaining work is shell-family guidance, frontier benchmarking, and workflow maturity rather than missing breadth
- verification depth
  more invariants, property tests, fuzzing, and acceptance-model coverage
- long-tail nonlinear hardening
  hard mixed nonlinear workflows and more mature failure behavior
- solver-path consistency
  dense vs sparse, constrained vs unconstrained, shell-family selection, and mixed shell/frame workflows

## Canonical Snapshot Rules

- keep the canonical repo-level test count here
- keep the shortest “where the solver stands now” summary here
- let [`README.md`](/Users/unbalancedparen/projects/dedaliano/README.md) stay qualitative and short
- let [`BENCHMARKS.md`](/Users/unbalancedparen/projects/dedaliano/BENCHMARKS.md) carry the detailed proof and capability matrix

## Next Priorities

1. performance and scale
2. verification hardening
3. solver-path consistency
4. long-tail nonlinear hardening
5. shell-family hardening and selection guidance

## Working Description

`Dedaliano is becoming one of the strongest open structural solvers, with a broader product surface than most solver-first projects.`
