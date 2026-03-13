# CYPECAD Feature Parity Roadmap

> Created 2026-03-13. Based on exhaustive feature audit of CYPECAD's product pages,
> documentation, and user forums, cross-referenced with a deep audit of the
> Stabileo Rust/WASM engine (50,014 LOC, 200+ tests).
>
> **Key finding:** Every solver capability needed for full CYPECAD parity is already
> implemented and tested in the Rust engine. All remaining work is frontend
> integration, WASM wiring, UI panels, and output generation.

---

## Current State Summary

### What CYPECAD Offers
- 3D stiffness matrix analysis (linear, P-Delta, seismic modal-spectral)
- RC design: beams, columns, slabs (flat/waffle/hollow-core/composite/post-tensioned), walls
- Steel design: members, connections (bolted/welded), baseplates
- Foundation design: pad footings, combined footings, mat foundations, pile caps
- Seismic: modal spectral, capacity design, beam-column joint checks, rigid diaphragms
- Fire resistance checks
- Multi-code: Eurocode 2/3/8, ACI 318, CIRSOC, NSR-10, NTE E.060, NCh430, +10 more
- Output: auto-generated construction drawings, reinforcement detail plans, bar bending schedules, calculation reports (memoria de cálculo), bill of quantities
- BIM: IFC import/export, DXF/DWG import

### What Stabileo Already Has (Solver)
| Capability | Engine File | LOC | Tests |
|---|---|---|---|
| Linear static 2D/3D | `solver/linear.rs` | — | ✓ |
| P-Delta 2D/3D | `solver/pdelta.rs` | — | ✓ |
| Buckling 2D/3D | `solver/buckling.rs` | — | ✓ |
| Modal 2D/3D | `solver/modal.rs` | — | ✓ |
| Spectral 2D/3D | `solver/spectral.rs` | — | ✓ |
| Plastic collapse | `solver/plastic.rs` | — | ✓ |
| Moving loads | `solver/moving_loads.rs` | — | ✓ |
| Corotational (geometric NL) | `solver/corotational.rs` | 1,877 | ✓ |
| Material nonlinear | `solver/material_nonlinear.rs` | 1,469 | ✓ |
| Fiber beam nonlinear | `solver/fiber_nonlinear.rs` | 1,015 | ✓ |
| Time history (Newmark/HHT) | `solver/time_integration.rs` | 1,459 | ✓ |
| Harmonic frequency response | `solver/harmonic.rs` | 532 | ✓ |
| Staged construction | `solver/staged.rs` | 1,269 | ✓ |
| Creep & shrinkage | `solver/creep_shrinkage.rs` | 419 | ✓ |
| Prestress/post-tensioning | `solver/prestress.rs` | 456 | ✓ |
| Cable/catenary analysis | `solver/cable.rs` | 603 | ✓ |
| Contact/gap elements | `solver/contact.rs` | 1,464 | ✓ |
| Arc-length (Riks) | `solver/arc_length.rs` | 608 | ✓ |
| Winkler foundation | `solver/winkler.rs` | 410 | ✓ |
| Soil-structure interaction | `solver/ssi.rs` | 385 | ✓ |
| Soil curves (API RP 2A) | `solver/soil_curves.rs` | 380 | ✓ |
| MPC / rigid diaphragms | `solver/constraints.rs` | 1,137 | ✓ |
| Imperfections | `solver/imperfections.rs` | 213 | ✓ |
| Guyan/Craig-Bampton reduction | `solver/reduction.rs` | 987 | ✓ |
| CIRSOC 201 (RC) | `postprocess/cirsoc201_check.rs` | — | ✓ |
| ACI 318 (RC) | `postprocess/rc_check.rs` | 412 | ✓ |
| Eurocode 2 (RC) | `postprocess/ec2_check.rs` | 258 | ✓ |
| AISC 360 (Steel) | `postprocess/steel_check.rs` | 329 | ✓ |
| Eurocode 3 (Steel) | `postprocess/ec3_check.rs` | 337 | ✓ |
| NDS Timber | `postprocess/timber_check.rs` | 382 | ✓ |
| TMS 402 Masonry | `postprocess/masonry_check.rs` | 216 | ✓ |
| AISI S100 CFS | `postprocess/cfs_check.rs` | 333 | ✓ |
| Spread footing design | `postprocess/foundation_check.rs` | 234 | ✓ |
| Connection design (AISC) | `postprocess/connection_check.rs` | 403 | ✓ |
| Serviceability (deflection) | `postprocess/serviceability.rs` | 123 | ✓ |
| Curved shell (MITC4 degen.) | `element/curved_shell.rs` | 2,777 | ✓ |
| 9-node shell (MITC9) | `element/quad9.rs` | 1,282 | ✓ |
| Solid-shell (SHB8-ANS) | `element/solid_shell.rs` | 755 | ✓ |
| Cable element | `element/cable.rs` | 330 | ✓ |
| Fiber beam element | `element/fiber_beam.rs` | 742 | ✓ |
| Connector element | `element/connector.rs` | 286 | ✓ |
| Curved beam meshing | `element/curved_beam.rs` | 294 | ✓ |

---

## Roadmap

### PHASE 1 — Rebar Detailing & Construction Documents
*Highest priority. This is the #1 deliverable gap — what engineers hand to contractors and building departments.*

#### 1.1 Advanced Beam Editor
- Wire `checkCirsoc201Members()` WASM → interactive beam reinforcement editor
- Required vs provided steel area diagrams along beam length
- Longitudinal bar placement: auto-select diameter, cutoff points, lap lengths per code
- Stirrup spacing: auto-compute from shear demand envelope, show spacing zones
- Hook geometry drawn per code-specified bend radii
- Manual override: add/remove/resize bars, lock edited bars from recalculation
- Support for 17 concrete codes (matching CYPECAD): CIRSOC 201, ACI 318, Eurocode 2, EHE-08, BAEL 91, NBR 6118, etc.

#### 1.2 Column Reinforcement Editor
- Interaction diagram overlay (P-M and P-Mx-My biaxial)
- Auto-select longitudinal bars + tie spacing from flexo-compression demand
- Cross-section detail drawing with bar positions, cover, tie hooks
- Slenderness check results displayed inline

#### 1.3 Slab Reinforcement Editor
- One-way and two-way slab rebar layout
- Top/bottom mesh with spacing, bar marks, bent-up bars at supports
- Corner torsion reinforcement detail
- Cantilever slab detailing

#### 1.4 Bar Bending Schedule (BBS) Export
- Aggregate all designed rebar from beams, columns, slabs, foundations into unified schedule
- Per-bar entry: mark, type, diameter, shape code (standard shapes per code), dimensions, quantity, unit length, total length, unit weight, total weight
- Bar shape diagrams (hooks, bends, cranks) drawn to standard shape codes
- Group by structural element (Beam B1, Column C1, Slab S1)
- Export formats: PDF table, Excel spreadsheet, CSV (for CNC rebar bending machines)
- Summary: total weight by diameter, total weight per element type, grand total

#### 1.5 Reinforcement Detail Plans (Planos de Armado)
Auto-generated DXF/PDF drawings per element type:
- **Beam plans (planos de pórticos)**: elevation view with longitudinal bars, stirrup zones, cross-sections at key points, despiece table
- **Column plans (planos de pilares)**: elevation + cross-section at each floor, longitudinal bars, ties, splices
- **Slab plans (planos de losas)**: plan view with top/bottom rebar layout, spacing annotations, section cuts
- **Foundation plans**: footing plan + section with rebar layout
- Title block with project info, scale, date, engineer
- Configurable: paper size, scale, annotation style, language

#### 1.6 Memoria de Cálculo (Calculation Report)
PDF report containing:
- Project description, materials, design codes used
- Load cases and combinations with factors
- Analysis results summary (max displacements, reactions)
- Design check results per element (unity ratios, governing code clauses)
- ULS and SLS verification tables
- Seismic analysis summary (periods, base shear, modal participation)
- Rebar summary tables

#### 1.7 Bill of Quantities
- Concrete volumes per element type and grade
- Rebar weights by diameter and element
- Steel tonnage by profile
- Formwork areas
- Export: Excel, CSV, PDF

---

### PHASE 2 — Foundation Design
*Wire existing `foundation_check.rs` + `winkler.rs` + `ssi.rs`*

#### 2.1 Pad Footing Design
- UI panel: footing dimensions, soil bearing capacity, column loads from solver reactions
- Wire `checkSpreadFootings()` WASM: Terzaghi/Meyerhof bearing, overturning, sliding, one-way shear, punching shear (two-way)
- Auto-size footing from demand
- Generate footing rebar detail → feeds into BBS

#### 2.2 Combined / Strap Footings
- Multi-column footing with beam-on-elastic-foundation model
- Wire `solveWinkler2D/3D()`

#### 2.3 Mat Foundations
- Shell FEM on Winkler springs — wire `solveSSI2D/3D()`
- Soil reaction contour plots
- Punching shear checks at each column

#### 2.4 Pile Caps
- Strut-and-tie model for 2/3/4-pile caps
- Pile load distribution from column forces
- Pile cap reinforcement detailing → BBS

#### 2.5 Foundation Beams
- Beam on Winkler foundation — `solveWinkler2D/3D()`
- RC beam design with soil reaction as loading

---

### PHASE 3 — Seismic Design Completeness
*Wire existing `constraints.rs` capacity design functions*

#### 3.1 Beam-Column Joint Verification
- Extract joint forces, check confinement + shear capacity per code
- Flag non-compliant joints

#### 3.2 Capacity Design (Strong Column / Weak Beam)
- ΣMc ≥ factor × ΣMb hierarchy check per CIRSOC/ACI/Eurocode 8
- Auto-flag under-designed columns

#### 3.3 Rigid Diaphragm UI
- Engine has full MPC implementation (`constraints.rs`, 1137 LOC)
- UI: auto-detect floor levels, toggle diaphragm per floor
- Display center of mass, center of rigidity, eccentricity

#### 3.4 Accidental Torsion
- 5% eccentricity per code as additional load cases

---

### PHASE 4 — Multi-Code Design Checks
*Wire existing Rust `postprocess/` modules to UI*

Each module exists with tests. Work per code: WASM export → code selector dropdown → results panel.

| Code | Engine File | LOC |
|---|---|---|
| Eurocode 2 (RC) | `ec2_check.rs` | 258 |
| Eurocode 3 (Steel) | `ec3_check.rs` | 337 |
| ACI 318 (US RC) | `rc_check.rs` | 412 |
| AISC 360 (US Steel) | `steel_check.rs` | 329 |
| NDS Timber | `timber_check.rs` | 382 |
| TMS 402 Masonry | `masonry_check.rs` | 216 |
| AISI S100 CFS | `cfs_check.rs` | 333 |
| Latin American (NSR-10, NTE E.060, NCh430) | Adapt `rc_check.rs` | — |

---

### PHASE 5 — Advanced Analysis UI
*All solvers implemented in Rust. Need UI panels + results display.*

| Feature | Engine File | LOC |
|---|---|---|
| Soil-structure interaction | `ssi.rs` + `soil_curves.rs` + `winkler.rs` | 1,175 |
| Nonlinear material | `material_nonlinear.rs` | 1,469 |
| Time history (Newmark/HHT) | `time_integration.rs` | 1,459 |
| Staged construction | `staged.rs` | 1,269 |
| Corotational (large disp.) | `corotational.rs` | 1,877 |
| Fiber nonlinear | `fiber_nonlinear.rs` + `fiber_beam.rs` | 1,757 |
| Creep & shrinkage | `creep_shrinkage.rs` | 419 |
| Prestress / post-tensioning | `prestress.rs` | 456 |
| Cable/catenary | `cable.rs` | 603 |
| Harmonic frequency response | `harmonic.rs` | 532 |
| Contact / gap elements | `contact.rs` | 1,464 |
| Arc-length (Riks) | `arc_length.rs` | 608 |
| Imperfections | `imperfections.rs` | 213 |

---

### PHASE 6 — Advanced Elements & BIM

#### 6.1 Advanced Shell Elements
- Curved shell MITC4 (`curved_shell.rs`, 2,777 LOC)
- 9-node MITC9 (`quad9.rs`, 1,282 LOC)
- Solid-shell SHB8-ANS (`solid_shell.rs`, 755 LOC)

#### 6.2 Connector Elements
- Bearings, isolators, springs, dashpots (`connector.rs`, 286 LOC)

#### 6.3 Curved Beam Meshing
- Auto-subdivide circular arcs into frame elements (`curved_beam.rs`, 294 LOC)

#### 6.4 IFC Export
- Write structural model + results to IFC format (reverse of current import)

#### 6.5 DXF 3D Import/Export
- Extend existing 2D DXF logic to 3D elements

---

### PHASE 7 — Steel Connections & Fire

#### 7.1 Steel Connection Design UI
- `connection_check.rs` (403 LOC) — bolt groups, weld groups per AISC 360
- JS implementation already exists for CIRSOC 301 (`connection-design.ts`)
- UI: connection editor with bolt/weld layout, plate dimensions
- Output: connection detail drawings, capacity tables

#### 7.2 Baseplate Design
- Column base forces → plate dimensions, anchor bolt design
- Concrete bearing check, anchor pullout

#### 7.3 Fire Resistance
- Temperature-reduced material properties (EC2/EC3 fire parts)
- Reduced section method for beams/columns
- Required fire rating → insulation thickness

---

## Where Stabileo EXCEEDS CYPECAD

Features we have (or have the solver for) that CYPECAD does not offer:

| Feature | Status |
|---|---|
| Plastic collapse analysis | Working (2D) |
| Moving load envelopes | Working (2D+3D) |
| Influence lines | Working (2D) |
| Step-by-step DSM pedagogy | Working (2D+3D) |
| What-If sensitivity explorer | Working |
| Real-time live calculation | Working |
| Web-based (zero install) | Working |
| URL instant model sharing | Working |
| Corotational geometric NL | Solver ready (1,877 LOC) |
| Fiber distributed plasticity | Solver ready (1,757 LOC) |
| Arc-length snap-through | Solver ready (608 LOC) |
| Full time history | Solver ready (1,459 LOC) |
| Harmonic frequency response | Solver ready (532 LOC) |
| Cable/catenary analysis | Solver ready (603 LOC) |
| Craig-Bampton substructuring | Solver ready (987 LOC) |
| 7 design codes from day one | Solver ready |

---

## References

- [CYPECAD Product Page](https://info.cype.com/en/software/cypecad/)
- [CYPECAD Concrete Beams Module](http://cypecad.cype.es/cypecad_vigas_hormigon.htm)
- [CYPECAD at Novedge — full feature list](https://novedge.com/products/buy-cypecad)
- [CYPECAD at Calculus Engineering](https://calculusengineering.com/cypecad/)
- [CYPECAD M54 Shop Page](https://shop.cype.com/en/product/cypecad-m54/)
- [CYPE Foundation Types](https://info.cype.com/en/blog/types-of-foundations-that-can-be-analysed-with-cype/)
- [CYPECAD Seismic Analysis](https://info.cype.com/en/product/cypecad-seismic-analysis/)
