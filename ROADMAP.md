# Dedaliano Roadmap

## Mission

Own the structural engineering software market vertically — from load determination to calculation report — by building a browser-native platform that is 5-10x cheaper than incumbents, AI-native from day one, and validated against every published benchmark in the industry.

---

## Current state

Dedaliano is a browser-native 2D + 3D structural analysis application implementing the Direct Stiffness Method from scratch. The solver is written in pure TypeScript with no external linear algebra dependencies. Over 1,050 tests across 31 suites validate the engine against analytical solutions.

### Implemented

- **2D solver** (3 DOF/node): linear static, P-Delta, buckling, modal, plastic, spectral, moving loads, influence lines
- **3D solver** (6 DOF/node): linear static, load combinations, envelopes
- **Load combinations**: LRFD factors, per-case and per-combination results, envelopes
- **Cross-section stress**: Navier, Jourawski, Bredt/Saint-Venant torsion, Mohr's circle, Von Mises/Tresca failure criteria
- **Section catalog**: 100+ European steel profiles (IPE, HEB, HEA, IPN, UPN, L, RHS, CHS), concrete sections, custom parametric builder
- **DSM wizard**: 9-step interactive walkthrough of the Direct Stiffness Method with KaTeX-rendered matrices
- **Kinematic analysis**: mechanism detection, rank check, diagnostic reports
- **Import/export**: JSON, DXF (R12), IFC (via web-ifc WASM), Excel, PNG, URL sharing (LZ-compressed)
- **3D rendering**: Three.js with Line2 screen-space elements, extruded section profiles, deformed shape visualization, stress utilization color maps
- **Undo/redo**, **autosave** (localStorage), **feedback system** (GitHub issues)
- **Rust solver**: experimental implementation in `engine/`, not yet connected via WASM

---

## Market strategy

### The opportunity

Structural engineering software is a $7-12B global market. Incumbents charge $2,000-15,000+/yr per seat:

| Software | Pricing | Owner |
|---|---|---|
| ETABS | $5,000-15,600 perpetual + $875-2,730/yr maintenance | CSI (Computers & Structures) |
| SAP2000 | $4,000-9,000 perpetual + maintenance | CSI |
| RFEM 6 | EUR 4,750 base + EUR 1,150-2,950 per add-on | Dlubal |
| STAAD.Pro | $3,210-4,411/yr | Bentley Systems |
| Robot Structural Analysis | $3,060/yr (bundled with AEC Collection) | Autodesk |
| IDEA StatiCa | $1,990-5,250/yr | IDEA StatiCa (Hilti) |
| ADAPT | $5,000+/yr | Trimble |
| RISA | $2,000-6,000/yr | RISA Technologies |

Browser-based competitors are small and growing: SkyCiv ($69-179/month, ~$1.7M ARR), ClearCalcs ($79-149/month, ~$793K ARR). Neither covers the full pipeline.

**83-94% of structural engineering firms have fewer than 20 employees.** These firms cannot afford $50,000-150,000/year in software licenses. A 10-person firm switching from incumbents to Dedaliano All-in-One ($99/month = $1,188/year) saves $30,000-80,000/year.

### Vertical ownership thesis

A structural engineer's workflow has 12 phases. No single tool covers them all today — engineers use 3-5 different programs per project. Each handoff loses data, wastes time, and introduces errors.

| Phase | What happens | Status | Incumbent |
|---|---|---|---|
| 1. Conceptual design | Select structural system, preliminary sizing | Not covered | Rules of thumb, spreadsheets |
| 2. Load determination | Dead, live, wind, snow, seismic per code | Partial | Spreadsheets, ASCE 7 Hazard Tool |
| 3. Modeling | Nodes, elements, supports, materials, loads | **Done** | ETABS, SAP2000, RFEM, Robot |
| 4. Analysis | Linear, P-Delta, buckling, modal, spectral | **Done (2D)**, 3D partial | ETABS, SAP2000, RFEM, STAAD |
| 5. Member design | Code-check every member per design code | Not covered | ETABS, RFEM, RISA, spreadsheets |
| 6. Connection design | Beam-column, brace, splice, base plate | Not covered | IDEA StatiCa, SteelSmart |
| 7. Foundation design | Footings, piles, retaining walls | Not covered | Spreadsheets, specialized tools |
| 8. Detailing | Rebar schedules, connection detail sheets | Not covered | Revit, Tekla |
| 9. Construction drawings | Plans, sections, schedules | Out of scope | Revit, Tekla (BIM/CAD) |
| 10. Calculation report | Permit-ready documentation | Not covered | Word, Mathcad, Tedds |
| 11. Quantity takeoff | Tons of steel, m3 of concrete, cost | Not covered | Spreadsheets |
| 12. Construction support | RFIs, change orders, field verification | Not covered | Manual process |

**Target: phases 2 through 8 + 10 + 11.** Phase 9 (construction drawings) requires a CAD engine — that's Revit/Tekla territory. We integrate with them via IFC round-trip and live link plugins rather than replacing them.

```
Loads → Model → Analysis → Code checks → Connections → Foundations → Report + Quantities
```

Owning this pipeline means the engineer never leaves Dedaliano except for construction drawings. Every phase generates revenue. Every phase increases switching costs.

### Competitive moat: AI-native architecture

Desktop incumbents (ETABS, SAP2000, RFEM, STAAD) are built on C/C++/Fortran codebases from the 1990s-2000s. They cannot easily add AI features. This is the structural advantage:

**AI features that incumbents cannot match:**

1. **Auto-sizing optimizer**: describe constraints ("steel moment frame, 4 stories, 30ft bays, AISC 360") and AI proposes initial member sizes, iterates analysis + code checks until all pass with minimal weight. Traditional approach: engineer guesses sizes, runs analysis, adjusts manually over hours
2. **Natural language model editing**: "add a 20kN point load at midspan of beam B3" or "change all W14 columns to W16" — parsed into model mutations. No menu navigation
3. **Intelligent load generation**: input building geometry and location, AI determines all applicable load cases (wind, seismic, snow, rain), applies them correctly per code, generates all required combinations. Currently takes engineers hours of manual work per project
4. **AI design review**: after analysis completes, AI reviews results and flags concerns ("column C7 has 0.98 utilization ratio under seismic, consider upsizing", "drift at story 4 exceeds H/400 limit", "foundation reactions exceed typical bearing capacity for assumed soil")
5. **Report narration**: AI writes the engineering narrative sections of the calculation report — describing the structural system, justifying design decisions, explaining why certain load combinations govern. Engineers currently spend 25-35% of time on documentation
6. **Code provision lookup**: engineer asks "what's the minimum reinforcement ratio for this beam per ACI 318-19?" and gets the exact clause, formula, and computed value in context

These features are trivial to add in a browser-based, AI-native architecture. They are nearly impossible to retrofit into a 30-year-old desktop C++ application.

### Pricing

| Tier | Price | Includes |
|---|---|---|
| **Free** | $0 | Full analysis (browser-only, small models), section properties, basic load calculator |
| **Pro** | $49/month | Server compute for large models, reports, full load calculator, AI assistant, unit toggle |
| **Steel** | +$39/month | Steel connection design |
| **Concrete** | +$29/month | Concrete design suite |
| **All-in-One** | $99/month | Everything: analysis + all design modules + AI features + server compute |
| **Education** | $50-100/student/yr | University site license with auto-grading and LMS integration |
| **API** | Pay-per-solve | HTTP solver endpoint for developers and integrations |

**Revenue model**: free tier drives adoption; $99/month All-in-One is the target conversion. At $1,188/year vs $5,000-15,000/year for incumbents, the price objection disappears. Revenue scales with users, not with per-seat licenses.

### Trust and credibility strategy

Engineers are conservative. They will not switch from SAP2000/ETABS unless they trust the results. Trust is earned, not claimed.

**Published verification documents:**
- For every design code module, publish a verification document comparing Dedaliano's results against hand calculations and incumbent software (SAP2000, ETABS, RFEM) for the same problems
- Format: problem description, hand calculation, Dedaliano result, SAP2000/ETABS result, percent difference
- Make these freely downloadable — they serve as both quality proof and marketing content

**Benchmark suite:**
- Run every CSI verification problem (SAP2000/ETABS publish hundreds of verification examples with expected answers)
- Run every AISC design example from the Steel Construction Manual companion
- Run every ACI 318 worked example from PCA and other publishers
- Run NAFEMS standard benchmark problems for FEA validation
- Publish results publicly with pass/fail status and numerical comparison

**Third-party validation:**
- Submit the software for review by independent structural engineering professors
- Seek listing in building department approved software lists where applicable
- Present benchmark results at industry conferences (NASCC, ACI Convention, SEAOC)

**Marquee clients:**
- Partner with 5-10 well-known structural engineering firms for beta testing
- Their public endorsement ("we verified Dedaliano against our SAP2000 models and results match within 0.1%") is worth more than any marketing campaign
- Offer free All-in-One for 1 year in exchange for a published case study

**Open source advantage:**
- The solver code is AGPL-3.0 and publicly readable. Any engineer can inspect the stiffness matrix assembly, the eigenvalue solver, the design code check formulas
- This transparency is impossible for closed-source incumbents. "Don't trust us — read the code" is a powerful message to a skeptical profession

### Distribution and go-to-market

**Content marketing (primary channel):**
- Technical blog posts solving real structural engineering problems with Dedaliano (SEO for "how to design a steel moment frame", "AISC 360 beam design example", etc.)
- Video tutorials comparing Dedaliano workflow vs ETABS/SAP2000 for the same problem
- Every benchmark verification document doubles as a long-form content piece
- Engineers spend 25-35% of time on documentation — content showing how Dedaliano automates this resonates immediately

**University partnerships:**
- Target structural analysis and steel/concrete design courses
- Professors get free accounts. Students learn on Dedaliano. After graduation, they bring it to their firms
- Dedaliano's DSM wizard already exists and is ideal for teaching — expand to design code teaching modules
- This is the long-term flywheel: every graduating class creates new potential customers

**Industry conferences:**
- Present at NASCC (steel), ACI Convention (concrete), SEAOC (seismic), ASCE Structures Congress
- Live demos comparing speed and accuracy against incumbent software
- Distribute benchmark verification documents
- Sponsor student competitions and capstone projects

**Revit/Tekla live link plugins:**
- Most firms use Revit or Tekla for BIM/drawings. They need analysis software that talks to their BIM tool
- Build bidirectional sync plugins: send model from Revit → Dedaliano for analysis → send results back to Revit with design data
- Not just IFC export — a live link that keeps the analytical model and BIM model synchronized
- This is how ETABS and Robot win deals: they integrate with Revit. We must match this

**Benchmark publications:**
- Publish benchmark comparison papers in ASCE Journal of Structural Engineering, Engineering Structures, etc.
- Academic credibility translates directly to practitioner trust
- "Peer-reviewed verification" is the gold standard in engineering

---

## Development methodology

Most code is generated by AI. Every pull request and every commit is reviewed by humans — expert software engineers for architecture, performance, and correctness, and expert structural engineers for every design code formula, coefficient, table lookup, and edge case.

AI handles what it does well: translating well-documented specifications (AISC 360 clauses, ACI 318 provisions, Eurocode formulas) into code, generating UI and rendering boilerplate, implementing textbook algorithms (DSM, eigenvalue solvers, sparse matrix operations), and producing test cases from published examples. LLMs already know these standards and can generate correct implementations for the vast majority of cases.

Humans handle what AI cannot guarantee: verifying that the generated code matches the intent of the design standard in every edge case, reviewing architectural decisions, catching subtle interactions between code provisions that AI might miss, and making final judgment calls on engineering correctness.

Additionally, every feature is validated against published benchmarks — CSI verification manuals, AISC design examples, Eurocode worked examples, NAFEMS benchmark problems. If the code passes hundreds of benchmark tests from authoritative sources, that is objective evidence of correctness that complements human review. The combination of AI generation, human review on every PR, and extensive benchmark validation is both faster and more rigorous than traditional development where a single engineer writes and self-reviews code.

Every feature ships with comprehensive automated tests validated against published benchmarks:

- **CSI verification manuals**: hundreds of worked examples from SAP2000/ETABS documentation, covering frame analysis, P-Delta, buckling, modal, spectral, and design code checks
- **AISC design examples**: the Steel Construction Manual companion examples for every chapter (tension, compression, flexure, shear, combined forces, connections)
- **ACI 318 worked examples**: published design examples for beams, columns, slabs, footings, shear walls
- **Eurocode worked examples**: official and third-party guides with full numerical solutions
- **NAFEMS benchmarks**: standard FEA validation problems for plates, shells, and nonlinear analysis
- **Textbook solutions**: analytical solutions for known problems (cantilever δ = PL³/3EI, simply supported beams, continuous beams, trusses)
- **Equilibrium checks**: every test case verifies ΣF = 0 and ΣM = 0 automatically

The existing solver already passes 1,050+ tests. Every new module (member design, connections, loads, concrete, timber) adds hundreds more. If the software passes every published benchmark for a given design code, that is objective evidence of correctness — stronger than any individual reading the code, because it tests outputs against known right answers.

This approach reduces development timelines by roughly 3x compared to traditional engineering software development. The bottleneck shifts from writing code to human review — which is where it should be for software where a wrong coefficient can affect structural safety.

---

## Phase 1 — Complete design tool for one market (months 1-4, 2-3 devs)

**Business goal:** ship a sellable product. Pick US (AISC 360 + ACI 318) or EU (Eurocode 2 + 3). Build the full pipeline for steel and concrete buildings in that code system. This is the minimum product that replaces incumbent software for the most common use cases.

**Total: 6-8 dev-months.**

### 1.1 3D solver parity

The 3D solver currently handles linear static analysis only. Every advanced analysis type is restricted to 2D. This is the most critical gap. The 2D implementations are complete and well-structured. The 3D versions follow the same algorithms with 12x12 element matrices (6 DOF per node) instead of 6x6.

| Analysis | 2D status | 3D work |
|---|---|---|
| P-Delta | Complete. Iterative (K + K_G)U = F, tolerance 1e-4. | Assemble 12x12 geometric stiffness K_G from 3D axial forces. Same iteration loop. |
| Buckling | Complete. Generalized eigenvalue via Cholesky + Jacobi. | 3D K_G in the eigenvalue problem. Same solver. |
| Modal | Complete. Consistent mass matrix, participation factors, effective mass. | 3D consistent mass matrix (12x12 element M). Participation factors in X, Y, Z. |
| Plastic | Complete. Event-to-event hinge formation. | Biaxial moment interaction surface (M_y, M_z) instead of uniaxial M_p check. |
| Spectral | Complete. SRSS/CQC, CIRSOC 103 spectra. | 3D modal superposition. Combine responses in three directions. |
| Influence lines | Complete. Unit load on 2D element chain. | 3D load paths. |
| Moving loads | Complete. Train of loads, envelope. | Same as influence lines: 3D paths. |
| DXF import/export | Complete. R12 format. | Handle 3D entity types (3DFACE, 3DPOLYLINE). |

Estimated effort: 1-1.5 dev-months.

### 1.2 Steel member design — code checking

Check every steel member against the design code. This is what turns Dedaliano from an analysis tool into a design tool. Without it, the engineer runs analysis here and opens another program to check members. **This is the single most important feature for revenue.**

**AISC 360 scope:**
- Tension members: yielding on gross section, rupture on net section (Chapter D)
- Compression members: flexural buckling, torsional buckling, flexural-torsional buckling. Column curves with effective length KL (Chapter E)
- Flexure: lateral-torsional buckling (LTB), flange local buckling (FLB), web local buckling (WLB). Compact, noncompact, slender classification (Chapter F)
- Shear: web shear with and without tension field action (Chapter G)
- Combined forces: H1-1a and H1-1b interaction equations (Chapter H)
- Serviceability: deflection limits L/360 (live), L/240 (total) (Chapter L)
- Slenderness limits: L/r recommendations

**Eurocode 3 scope:**
- Cross-section classification (Class 1-4)
- Tension, compression (buckling curves a, b, c, d), bending, shear, combined
- Lateral-torsional buckling (general method and simplified)
- Serviceability deflection limits

Output per member: utilization ratio (demand/capacity), governing limit state, governing load combination, pass/fail. Color-coded visualization on the model: green (<0.7), yellow (0.7-0.9), red (>0.9), failed (>1.0).

Estimated effort: 1.5-2 dev-months per code.

### 1.3 Concrete member design — code checking

**ACI 318 scope:**
- Beam flexure: required As from moment envelope using rectangular stress block (Whitney), minimum and maximum reinforcement ratios, bar selection and spacing
- Beam shear: required Av/s, stirrup spacing, Vs + Vc checks
- Beam deflection: immediate (Ie effective moment of inertia) and long-term (lambda multiplier for creep)
- Beam crack width: Gergely-Lutz or direct tension stress check
- Column design: P-M interaction diagram (uniaxial), P-Mx-My contour (biaxial), slenderness effects (moment magnification δns, δs)
- One-way slab design: strip method, minimum thickness tables
- Two-way slab design: direct design method, equivalent frame method, punching shear (Vc at critical perimeter d/2 from column)

**Eurocode 2 scope:**
- Same categories, different formulas: parabolic-rectangular stress block, variable strut inclination for shear (θ method), crack width per EN 1992-1-1 §7.3, effective creep for long-term deflection

Output: required reinforcement schedule per member (As_top, As_bot at each section, stirrup spacing), interaction diagrams for columns, summary table with governing sections.

Estimated effort: 1.5-2 dev-months per code.

### 1.4 Load determination

Full code-based load generation for building structures.

**Wind loads (ASCE 7 Ch. 26-31 or Eurocode 1-4):**
- Input: building geometry, location (zip code or coordinates), exposure category, terrain, topography, risk category, enclosure classification
- Output: design wind speed, velocity pressure at each height (Kz profile), external pressure coefficients (Cp) for windward, leeward, side walls, roof zones, internal pressure (GCpi), wind pressures on each surface
- Automatically creates load cases for each wind direction

**Seismic loads (ASCE 7 Ch. 11-23 or Eurocode 8):**
- Input: location, site class, risk category, structural system (R, Cd, Ω0)
- Output: Ss, S1 from USGS API (or national maps), SDS, SD1, seismic design category, base shear (ELF), story forces Fx, accidental torsion
- Optionally: modal response spectrum analysis (already implemented for CIRSOC 103, adapt for ASCE 7/Eurocode 8 spectra)

**Other loads:**
- Snow loads (ASCE 7 Ch. 7 or Eurocode 1-3): ground snow load pg, flat roof snow pf, drift loads, sliding loads
- Rain loads (ASCE 7 Ch. 8): ponding on flat roofs
- Live loads: lookup tables by occupancy (ASCE 7 Table 4.3-1 or Eurocode 1-1 Table 6.2)
- Dead load takedown: self-weight from model + superimposed dead (user input per floor)

**Load combinations:**
- Generate all combinations per ASCE 7 §2.3 (LRFD) or Eurocode 0 (limit states)
- Include companion factors, notional loads, orthogonal seismic effects

Estimated effort: 1-1.5 dev-months per code.

### 1.5 Calculation reports

Auto-generate permit-ready calculation packages from analysis and design results.

Contents:
- Model description: geometry (node coordinates, element connectivity), materials (E, ν, fy), sections (A, I, J with profile designation), supports
- Loading: load cases with all applied loads, combination table with LRFD factors
- Analysis summary: solver method, convergence, key assumptions
- Results per combination: displacement tables, reaction tables, internal force diagrams (M, V, N, T), envelope curves
- Member design checks: utilization ratios at critical sections, governing limit state, governing combination per element
- Stress checks: Von Mises/Tresca verification where applicable
- Stability checks: buckling load factors, effective lengths, slenderness ratios

Output formats:
- **LaTeX source**: paste into Overleaf or compile locally
- **PDF**: one-click export for submission or review
- Copy any individual matrix (K, F, U, element k, transformation T) from the DSM wizard as LaTeX source

Template system for different firms, jurisdictions, and detail levels (summary vs full).

Estimated effort: 0.5-1 dev-months.

### 1.6 Quantity takeoff and cost estimation

The model already contains member lengths, section properties, and materials. Compute:
- Tons of structural steel (by section type and grade)
- Volume of concrete (m3 by element type)
- Weight of reinforcement (kg, from member design output)
- Number of bolts, length of welds (from connection design, Phase 2)
- Bill of materials table
- Cost estimate from user-defined unit costs

Estimated effort: 0.5 dev-months.

### 1.7 Unit system toggle

All internal calculations remain in SI (m, kN, kN·m, MPa). Display layer converts to imperial (ft, kip, kip·ft, ksi) when the user selects imperial. Conversion factors at the UI boundary only. Persisted in localStorage. Essential for US market adoption.

Estimated effort: 0.5 dev-months.

### 1.8 AI assistant (Phase 1 features)

Ship the first AI-native features alongside Phase 1 to differentiate from day one:

- **Intelligent load generation**: user inputs building geometry and location, AI determines all applicable load cases and combinations per code. Saves hours of manual work per project
- **AI design review**: after analysis + code checks, AI scans results and flags concerns (high utilization, drift limits, bearing capacity issues). First-pass review before human engineer reviews
- **Report narration**: AI writes the engineering narrative sections of the calculation report — structural system description, design assumptions, governing conditions

These are high-impact, low-risk AI applications. The AI suggests; the engineer reviews and approves. No autonomous decisions on structural safety.

Estimated effort: 1 dev-month.

---

## Phase 2 — Connections, server, performance (months 4-8, 3-4 devs)

**Business goal:** close the design loop with connections. Enable server-side computation and collaboration. Open the API and education markets.

**Total: 7-10 dev-months.**

### 2.1 Steel connection design

Design the 20 most common steel connections. For each: select bolt pattern, weld sizes, stiffener plates. Check all failure modes per AISC 360/Eurocode 3.

**Connection types (initial 10, then expand to 20):**
1. Shear tab (fin plate / single plate)
2. Double angle shear connection
3. Unstiffened seated connection
4. Extended end-plate moment connection (4-bolt and 8-bolt)
5. Flush end-plate moment connection
6. Column base plate (pinned and moment)
7. Bracing gusset plate (Whitmore section, block shear)
8. Column splice (bolted flange plate)
9. Beam splice (bolted web and flange)
10. Beam-to-beam connection (coped beam, simple framing)

**Failure modes checked per connection:**
- Bolt shear, bolt bearing, bolt tension, bolt slip (for slip-critical)
- Plate bending (yield line), plate shear yielding, plate shear rupture
- Weld throat stress (fillet and CJP)
- Block shear (Ubs factor)
- Column web panel zone shear
- Prying action (for T-stub moment connections)

**What Dedaliano already provides:**
- Section catalog for beam/column selection
- Internal forces (M, V, N) at each joint from the solver
- 3D rendering for connection visualization with bolts, plates, welds
- KaTeX for displaying code check equations

New code: ~15,000-25,000 LOC. Estimated effort: 1.5-2 dev-months for initial 10 types.

### 2.2 Rust/WASM solver

The TypeScript solver handles ~500 free DOFs in real time. Beyond that, the O(n³) dense linear system solver and O(n⁴) Jacobi eigenvalue solver become bottlenecks. For a 200-element 3D frame (~1,200 DOFs), modal analysis takes seconds, not milliseconds.

Integration path:
1. Compile the Rust solver in `engine/` to WASM via wasm-pack
2. Expose `solve(json) -> json` callable from TypeScript
3. Fall back to TypeScript solver if WASM unavailable
4. Same Rust binary compiles as native executable for server-side computation

Additionally: sparse matrix storage (CSR/CSC) and iterative solvers (conjugate gradient for SPD systems, implicitly restarted Lanczos for eigenvalue problems). Sparse solvers scale O(nnz) per iteration instead of O(n³), making 5,000+ DOF models feasible.

Estimated effort: 1.5-2 dev-months.

### 2.3 Real-time collaboration

Multiple engineers on the same model simultaneously. This does not exist in any commercial structural analysis tool (SAP2000, ETABS, Robot, RSTAB, RFEM are all single-user desktop).

**Architecture: CRDTs over WebSocket.**

CRDTs (Conflict-free Replicated Data Types) guarantee convergence by construction, regardless of message ordering, with no transformation functions. Dedaliano's model is a set of Maps (`Map<id, Node>`, `Map<id, Element>`, etc.) which maps directly to CRDT Map types.

| Store data | CRDT type | Conflict policy |
|---|---|---|
| Nodes | LWW-Map (Last Writer Wins per key) | Two users edit same node: last write wins |
| Elements | LWW-Map | Same element edited: last write wins |
| Materials, sections | LWW-Map | Rarely contested |
| Loads | Add-Wins Set | Both users add loads: both kept |
| ID generation | Client-ID + Lamport clock | No conflicts by design |

**Library: Yjs.** Most mature CRDT library. Native Y.Map, Y.Array, Y.Text types. Awareness protocol for cursor/presence sharing. UndoManager for per-user undo stacks.

**Networking: WebSocket primary, WebRTC optional.** y-websocket for relay through server. y-webrtc for optional lower-latency P2P cursor sync. Both providers on the same Y.Doc — Yjs deduplicates.

```
Client A <--WebSocket--> Server <--WebSocket--> Client B
   |                                               |
   +-------------WebRTC (optional)------------------+
```

**Server: Go.** Accept WebSocket connections, relay Yjs deltas, persist documents to database, check auth tokens. A few hundred lines. Goroutines handle thousands of concurrent connections.

**CRDTs are valuable even with a server** because they enable local-first editing: every edit applies instantly to the local replica (0ms latency), delta sent in background. Also provides offline editing for free.

**What changes in Dedaliano:**
1. ID generation: replace auto-increment counters with UUIDs or Yjs client-ID + Lamport clock
2. Model store CRDT bridge: replace `$state` Maps in `model.svelte.ts` with wrappers over Yjs Y.Maps. Mutations go through Yjs. Observe callbacks propagate into Svelte reactivity. Incremental: one entity type at a time
3. Undo/redo: replace snapshot-based history with Yjs UndoManager (per-user stacks)
4. Referential integrity: validation pass after every remote sync (detect elements pointing to deleted nodes)
5. Awareness and presence: colored cursors, user list, "user X is editing element Y" in 2D and 3D viewports
6. Server: y-websocket relay (~30 LOC), persistence (~50 LOC), JWT auth (~50 LOC)

**Solver in multiplayer:** each client runs the solver locally on its own CRDT replica. Results are not synced — they are ephemeral, derived from the model.

Estimated effort: 1.5-2 dev-months.

### 2.4 REST API

Expose the solver as an HTTP endpoint. Input: JSON model definition. Output: JSON results (displacements, reactions, internal forces, stress checks). The Go server hosts the API. The Rust solver (native binary) handles computation.

Use cases:
- Parametric studies: Python script varies parameters, calls API for each case, plots results in Jupyter
- Automated verification: CI pipeline checks that a model still passes after a design change
- Auto-grading: professor's script compares student submissions against reference solutions
- Integration: BIM software, optimization scripts, custom dashboards

Estimated effort: 0.5-1 dev-months.

### 2.5 Foundation design

Analyze and design shallow foundations from column reactions.

**Scope:**
- Isolated footings: sizing for bearing pressure, one-way and two-way shear, flexure, reinforcement
- Combined footings: two columns on one footing, trapezoidal pressure distribution
- Strap footings: two footings connected by a strap beam
- Mat foundations: Winkler springs (already in solver as spring supports), plate on elastic foundation

**Soil parameters:** bearing capacity, modulus of subgrade reaction, friction angle, cohesion as additional material properties.

Code checks per ACI 318 or Eurocode 2 (footing design) + geotechnical bearing capacity (Terzaghi/Meyerhof or Eurocode 7).

Estimated effort: 0.5-1 dev-months.

### 2.6 Education platform

Interactive problem sets with auto-grading for university courses.

**Features:**
- Professor defines a structure and reference solution
- Student solves by hand, enters values (reactions, displacements, internal forces)
- Tool compares each value against solver's solution: correct/incorrect with expected value and error
- Quiz mode: hide selected results, student computes them
- Real-time visualization: change a load and see the moment diagram update
- LMS integration: Moodle, Canvas, Google Classroom via LTI protocol
- Embed mode: iframe-friendly `?embed=true` URL parameter, hides toolbar/tabs/panels, shows only viewport with pre-loaded model

**Revenue model:** $50-100/student/year. A 200-student course = $10,000-20,000/year from one department.

Estimated effort: 1-1.5 dev-months.

### 2.7 AI assistant (Phase 2 features)

- **Natural language model editing**: "add a 20kN point load at midspan of beam B3", "change all W14 columns to W16" — parsed into model mutations. No menu navigation
- **Auto-sizing optimizer**: describe constraints and let AI iterate analysis + code checks to find optimal member sizes
- **Code provision lookup**: ask "what's the minimum reinforcement ratio for this beam per ACI 318-19?" and get the exact clause, formula, and computed value

Estimated effort: 1 dev-month.

### 2.8 Revit/Tekla live link plugin

Bidirectional sync between Dedaliano and BIM tools:
- Import analytical model from Revit/Tekla into Dedaliano
- Run analysis and design in Dedaliano
- Push results (utilization ratios, reactions, member sizes, reinforcement) back to the BIM model
- Keep models synchronized as either side changes

Not just IFC export — a live link that reflects changes in real time. This is how ETABS and Robot win deals in firms that use Revit. We must match this integration to compete for those firms.

Estimated effort: 1-1.5 dev-months.

---

## Phase 3 — Second market, more materials (months 8-14, 4-5 devs)

**Business goal:** double the addressable market with the second code system. Expand to timber and prestressed concrete — underserved niches with high willingness to pay.

**Total: 8-11 dev-months.**

### 3.1 Second design code

Add Eurocode 2+3 if Phase 1 chose AISC/ACI, or AISC 360 + ACI 318 if Phase 1 chose Eurocode. This doubles the addressable market from ~25-30% to ~50-60% of global structural engineering.

Scope: steel member design, concrete member design, load determination, connection design — all re-implemented for the second code. The solver and UI are shared; only the code check functions differ.

Estimated effort: 3-4 dev-months. Second code is always faster — patterns from the first implementation are reused.

### 3.2 Timber design

NDS (US) or Eurocode 5 (EU). Residential construction in US/Canada/Scandinavia is almost entirely wood. Mass timber (CLT buildings up to 18 stories) is creating new demand. ClearCalcs' most popular calculators are timber.

**Unique concerns:**
- Moisture-adjusted strength: duration of load factors (CD), wet service factors (CM), temperature factors (Ct), size factors
- Notch effects: reduced shear capacity at notched ends
- Connection capacity: Johansen yield model (European Yield Model) — bolt, nail, screw, dowel connections
- Lateral-torsional buckling: different slenderness formulas and beam stability factors than steel
- Section catalog: standard lumber sizes (2x4 through 2x12, 4x4 through 12x12), glulam layups, CLT panel properties, LVL/PSL/LSL engineered wood

**Design checks:** flexure (Fb'), compression (Fc'), tension (Ft'), shear (Fv'), bearing (Fc⊥'), combined bending and axial, deflection limits.

Estimated effort: 1-1.5 dev-months per code.

### 3.3 Prestressed and post-tensioned concrete

ADAPT (Trimble) charges $5,000+/yr. Every mid-rise and high-rise concrete building uses post-tensioning. Very specialized, very expensive incumbents, very underserved.

**Scope:**
- Tendon profile geometry: parabolic, harped, straight. Layout in plan and elevation
- Prestress losses: elastic shortening, friction (wobble + curvature), anchorage set, creep, shrinkage, relaxation
- Staged analysis: transfer (initial prestress), service (long-term losses), ultimate
- Hyperstatic (secondary) effects from tendon profile
- Design checks: stress limits at transfer and service (ACI 318 §24.5 or Eurocode 2 §5.10), ultimate flexural capacity with bonded/unbonded tendons, shear with prestress contribution

Estimated effort: 1.5-2 dev-months per code.

### 3.4 Additional connection types

Expand from 10 to 20 steel connection types. Add:
- Stiffened seated connection
- Single-plate (extended shear tab)
- Hanger connection
- Moment frame panel zone doubler plate design
- Hollow section connections (HSS-to-HSS)
- Truss gusset plate (multi-member)

Plus concrete connections (corbels, brackets, anchor bolt groups per ACI 318 Appendix D / Eurocode 2 anchorage) and timber connections (bolted, nailed, screwed per NDS Chapter 12 or Eurocode 5 Chapter 8).

Estimated effort: 1-1.5 dev-months.

---

## Phase 4 — Full platform (months 14-22, 5-8 devs)

**Business goal:** cover every common structural material and analysis type. Revenue from Phases 1-3 funds the team. After this phase, Dedaliano handles ~80-85% of everything a structural engineer does (the remaining ~15-20% is construction drawings — Revit/Tekla territory).

**Total: 18-26 dev-months.**

### 4.1 Cold-formed steel

AISI S100 (US), Eurocode 3 Part 1-3 (EU). Light-gauge steel framing for residential and low-rise commercial. Fast-growing market. Almost no independent browser-based options.

Unique analysis: effective width method, distortional buckling (Direct Strength Method — DSM), local buckling interaction. Thinner sections mean more complex stability behavior than hot-rolled steel. Section catalog: C-studs, tracks, Z-purlins, hat channels, custom cold-formed shapes.

Estimated effort: 1-1.5 dev-months per code.

### 4.2 Composite steel-concrete

AISC 360 Chapter I (US), Eurocode 4 (EU). Composite beams (steel beam + concrete slab acting together), composite slabs with metal decking. Used in virtually every multi-story steel building.

Scope: shear stud calculation (Qn), effective slab width, plastic moment capacity of composite section, partial composite interaction, construction stage (unshored beam carrying wet concrete), deflection with partial interaction, vibration check.

Estimated effort: 0.5-1 dev-months per code.

### 4.3 Masonry design

TMS 402 (US), Eurocode 6 (EU). Bearing walls, shear walls, lintels, arches. Common in low-rise construction in Latin America, Europe, Middle East.

Scope: flexural capacity of grouted/ungrouted walls, shear capacity (in-plane and out-of-plane), axial-flexural interaction, slenderness effects, reinforcement requirements, bond beam design.

Estimated effort: 0.5-1 dev-months per code.

### 4.4 Plates and shells

Triangular and quadrilateral plate/shell elements for floor slabs, walls, tanks, and shell structures. The Discrete Kirchhoff Triangle (DKT) for bending combined with the Constant Strain Triangle (CST) for membrane action.

This is the single largest expansion in scope. Plate analysis covers reinforced concrete slab design, steel deck design, shear wall analysis, and foundation mat analysis. Requires a 2D mesh generator (Delaunay triangulation or advancing front).

Estimated effort: 2-3 dev-months. Hardest item — new element formulations and mesh generation.

### 4.5 Advanced analysis types

Features that advanced users and seismic engineers expect:

**Nonlinear time history analysis:** direct integration of equations of motion (Newmark-beta, HHT-alpha) with applied ground motion records. Required for performance-based seismic design of tall buildings and critical facilities. Input: acceleration time history (from PEER NGA database). Output: response history, peak inter-story drifts, peak floor accelerations, residual drifts.

**Staged/phased construction analysis:** model construction sequence — pour floor 1, shore, pour floor 2, remove shores, apply finishes. Each stage has different geometry and loads. Critical for post-tensioned concrete, tall buildings, and bridges. Tracks cumulative creep, shrinkage, and relaxation across stages.

**Nonlinear material models:**
- Fiber analysis for concrete columns: divide cross-section into fibers, each with its own stress-strain curve. Captures concrete cracking, steel yielding, confined vs unconfined concrete behavior. Required for accurate P-M interaction and ductility assessment
- Concrete cracking and tension stiffening models
- Steel hardening models (kinematic, isotropic) for cyclic analysis

**Cable and tension structures:** cable elements with large-displacement geometric nonlinearity. Form-finding algorithms (force density method, dynamic relaxation). Applications: suspension bridges, cable-stayed structures, tensile membrane roofs, guy wires.

Estimated effort: 3-4 dev-months total.

### 4.6 Bridge design

AASHTO LRFD (US), Eurocode 1-2 (EU). Bridge load rating for existing bridges, permit load analysis. Government infrastructure market with reliable budgets.

Scope: HL-93 live load model (lane + truck/tandem), distribution factors, fatigue limit state, load rating factors (RF), prestressed girder design, bridge deck design.

Estimated effort: 1.5-2 dev-months.

### 4.7 Fire design

Structural fire engineering. Temperature-dependent material properties, fire resistance verification per ISO 834 fire curves. Eurocode fire parts (EN 1992-1-2 for concrete, EN 1993-1-2 for steel, EN 1995-1-2 for timber).

Growing regulatory requirement. Critical for timber/mass timber buildings where fire is often the governing design case. Very few tools exist at any price.

Estimated effort: 1-1.5 dev-months.

### 4.8 Seismic retrofit and rehabilitation

Existing building assessment and retrofit design. Huge market — millions of buildings worldwide need seismic upgrades.

**Scope:**
- Assessment per ASCE 41 (US) or Eurocode 8-3 (EU): nonlinear static (pushover) analysis for performance evaluation
- FRP wrapping: flexural and shear strengthening of concrete members with fiber-reinforced polymers
- Steel jacketing: concrete column strengthening with steel plates
- Base isolation: isolation bearings (lead-rubber, friction pendulum) — modify support conditions, nonlinear analysis of isolated structure
- Supplemental damping: viscous dampers, buckling-restrained braces as retrofit elements

Estimated effort: 1.5-2 dev-months.

### 4.9 Geotechnical analysis

Slope stability (method of slices: Bishop, Janbu, Spencer) and retaining wall design (gravity walls, cantilever walls, sheet pile walls). 2D problems that fit Dedaliano's existing 2D viewport.

Slope stability requires a soil profile (layers with different properties) and a search algorithm for the critical slip surface (circular or non-circular). Output: factor of safety and geometry of critical failure surface.

Deep foundations: single piles (axial capacity from SPT/CPT), pile groups (group efficiency, settlement), laterally loaded piles (p-y curves).

Estimated effort: 1-1.5 dev-months for slope stability, 1-1.5 for deep foundations.

### 4.10 Fatigue analysis

For bridges, crane girders, offshore structures, wind turbine towers. S-N curves, Miner's rule cumulative damage, stress range counting (rainflow method). AISC 360 Appendix 3, Eurocode 3-1-9.

Estimated effort: 0.5-1 dev-months.

### 4.11 Floor vibration and serviceability

Footfall analysis, vibration from equipment or pedestrian traffic. Required for hospitals, labs, offices with sensitive equipment. SCI P354 (UK), AISC Design Guide 11 (US).

Natural frequency calculation (already in modal analysis), damping estimation, response factor computation, acceleration limits.

Estimated effort: 0.5 dev-months.

### 4.12 Scaffolding and temporary works

Formwork, shoring, scaffolding design. Required on every construction site. Usually done poorly or with rules of thumb. High liability. No good browser tool.

Covers: falsework design for concrete pours, scaffold load capacity, bracing requirements, foundation checks for temporary supports.

Estimated effort: 0.5-1 dev-months.

### 4.13 Detailing (partial)

Full construction drawings are out of scope (CAD engine). Two feasible slices:

**Rebar schedules and bar bending shapes:** take concrete design output (required As at each section) and generate bar marks, cut lengths, bending shapes per standard shapes (ACI Detailing Manual or Eurocode 2 standard bends). Standalone product in some markets.

**Connection detail sheets:** take connection design output and generate plan/elevation drawing with bolt pattern, weld symbols, plate dimensions. Not a full shop drawing but enough for the calculation report and for the fabricator to start from.

Estimated effort: 1.5-2 dev-months.

---

## Phase 5 — Global code expansion (months 22-30)

**Business goal:** expand from 50-60% to 80-90% of the global structural engineering market by adding the most important regional design codes.

Four code families (AISC/ACI + Eurocode) cover the US, EU, and most countries that adopt either system. But several large markets have their own codes:

| Region | Code | Market size | Priority |
|---|---|---|---|
| India | IS 456 (concrete), IS 800 (steel) | ~200,000 structural engineers, fast-growing construction market | High |
| China | GB 50010 (concrete), GB 50017 (steel) | Largest construction market by volume | High |
| Japan | AIJ (Architectural Institute of Japan) standards | High seismicity, sophisticated engineering market | Medium |
| Brazil | NBR 6118 (concrete), NBR 8800 (steel) | Largest Latin American market | Medium |
| Australia/NZ | AS 3600 (concrete), AS 4100 (steel) | Wealthy market, underserved by incumbents | Medium |
| Canada | CSA A23.3 (concrete), CSA S16 (steel) | Close to US codes but distinct | Medium |
| South Korea | KBC (Korean Building Code) | Advanced construction market | Lower |

**Strategy:** each code implementation reuses the same architecture — shared solver, shared UI, only code check functions differ. After the first two code families, adding a new code is primarily a formula translation exercise.

**Indian codes (IS 456 + IS 800):** India has ~200,000 structural engineers and a construction boom. IS codes are based on older British standards but diverging. Most Indian engineers currently use STAAD.Pro or pirated copies of ETABS. Browser-based tool at $49-99/month is transformative for this market.

**Chinese codes (GB 50010 + GB 50017):** China builds more structures annually than any other country. Chinese design codes have unique provisions (seismic intensity vs PGA, different concrete grades, different partial safety factors). Major opportunity but requires Mandarin localization.

**Other codes:** prioritize based on market size, willingness to pay, and similarity to existing implementations. Japanese AIJ codes are the most different from Western codes. Canadian CSA codes are closest to AISC/ACI.

Estimated effort: 1-2 dev-months per code pair (steel + concrete). Faster with each additional code as the pattern is established.

**Total: 8-14 dev-months for 4-5 additional code families.**

---

## Additional features

### Conceptual / preliminary design

Span tables and rules of thumb for initial member sizing. Structural system selection guide (moment frames vs braced frames vs shear walls based on height, span, seismicity). Quick cost comparison between steel and concrete framing.

### Nonlinear pushover analysis

Incremental static analysis with monotonically increasing lateral loads. Capacity curve (base shear vs roof displacement) for performance-based seismic design. Extends the existing 2D plastic analysis with a load control algorithm (force-controlled or displacement-controlled arc-length method).

### Import/export improvements

**DXF layer mapping wizard:** when importing DXF, show a dialog listing all layers and let the user map each to a structural role ("these lines are elements", "these points are supports"). Currently all lines are imported as elements with default properties.

**IFC round-trip:** currently import-only via web-ifc WASM. Adding export enables round-trip BIM interoperability: import from Revit/Tekla, analyze in Dedaliano, export back with results as IFC property sets.

### UX and accessibility

- Responsive/mobile: pinch-to-zoom, swipe to pan, long-press for context menu, collapsible panels on small screens
- Light mode and high contrast theme alongside current dark theme
- Keyboard accessibility for all toolbar actions
- Localization: English, Spanish, Portuguese, Mandarin, Hindi (matching global code expansion markets)

### Visualization improvements

- Load case comparison split view: two panels showing the same structure with different load cases, synchronized zoom/pan
- Animated influence lines: unit load moves along the structure, deformed shape and influence line update in real time
- SVG export: resolution-independent viewport export for technical reports and papers
- 3D overlay comparison (superimposing results from different cases, not yet implemented for 3D)

---

## Research-driven features

### Topology optimization

Ground structure optimization: start with a dense mesh of candidate bars, remove bars carrying negligible force. Returns the lightest truss satisfying stress and displacement constraints. Useful for long-span roofs, transfer structures, bridge preliminary design.

### Physics-Informed Neural Networks (PINNs) as surrogate models

Train a small neural network to approximate the solver for a parametric structure family. Drag a slider to change span from 6m to 12m and see the moment diagram update at 60fps. Training data from Dedaliano's own solver. Inference in browser via ONNX Runtime WASM or TensorFlow.js.

### Digital twin integration

Connect a Dedaliano model to real-time sensor data (strain gauges, accelerometers, displacement transducers). Live stress and displacement overlays. Use cases: structural health monitoring, proof load testing, construction monitoring. Depends on REST API and Rust/WASM solver.

### Optimization as a service

User defines design variables (section sizes, member topology, support locations), constraints (stress limits, displacement limits, code checks), and an objective (minimize weight, cost, or deflection). Optimizer (genetic algorithm, gradient-based, or hybrid) calls solver repeatedly. Natural use case for server-side computation paid tier.

---

## Defending against incumbent response

When Dedaliano gains traction, incumbents will respond. Anticipate and prepare:

**CSI (ETABS/SAP2000) moving to cloud:** CSI has been slowly building web interfaces. Their advantage is 30 years of verified analysis and design. Their disadvantage is a monolithic C/C++ codebase that is extremely difficult to port to the browser. Our response: move faster. By the time they ship a browser version, we should have feature parity on the design code coverage they offer.

**Dlubal (RFEM) price drops:** Dlubal already has aggressive pricing in some markets. They may match our $99/month. Our response: the free tier. If they match our price, we still have a free tier they can't match without destroying their business model. Also: our AI features create value they cannot replicate without rewriting their software.

**Autodesk bundling:** Autodesk may bundle Robot Structural Analysis more aggressively with their AEC Collection. Our response: Robot is widely considered the weakest analysis tool. Bundling a weak product doesn't make it stronger. Focus on being the best tool, not the bundled tool.

**SkyCiv/ClearCalcs competing:** Other browser-based tools are closest competitors. Our response: cover more of the pipeline (they don't do connections, foundations, or advanced analysis), be open source (they're closed), and push AI features they haven't built.

**Key insight:** incumbents cannot easily add AI-native features to 30-year-old desktop C++ codebases. They cannot easily add real-time collaboration. They cannot easily match browser-native deployment (zero install, works on any device). Our structural advantages compound over time.

---

## Total effort

All estimates assume AI-generated code with human review on every PR and commit (see Development methodology above).

| Category | Dev-months |
|---|---|
| Phase 1: one-code design tool + AI assistant | 7-10 |
| Phase 2: connections + server + collaboration + AI | 7-10 |
| Phase 3: second code + timber + prestressed | 8-11 |
| Phase 4: full platform (all materials + advanced analysis) | 18-26 |
| Phase 5: global code expansion (4-5 additional code families) | 8-14 |
| **Total** | **48-71** |

| Team size | Time to Phase 4 complete | Time to Phase 5 complete |
|---|---|---|
| 3 developers + reviewers | 14-19 months | 17-24 months |
| 5 developers + reviewers | 8-11 months | 10-14 months |
| 10 developers + reviewers | 5-6 months | 6-8 months |

Phase 1 alone (sellable product) with 3 developers: **2-3 months**.

### Revenue milestones

| Milestone | When | Revenue potential |
|---|---|---|
| Phase 1 ships | Month 4 | First paying customers. $99/month All-in-One. Target: 50-100 users = $5,000-10,000/month |
| Phase 2 ships | Month 8 | Connection design adds upsell. Education platform. API. Target: 200-500 users = $20,000-50,000/month |
| Phase 3 ships | Month 14 | Second code doubles TAM. Timber captures residential market. Target: 500-1,000 users = $50,000-100,000/month |
| Phase 4 ships | Month 22 | Full platform. Competes with ETABS/SAP2000 head-on. Target: 1,000-5,000 users |
| Phase 5 ships | Month 30 | Global coverage. India and China markets open. Target: 5,000-20,000 users |

### The bottleneck: human review

AI generates code fast. The bottleneck is the human review process — expert structural engineers verifying every design code formula, coefficient, and edge case, and expert software engineers reviewing architecture and correctness. This is the right bottleneck. For engineering software, the review is where the value is.

Design codes update every 3-6 years (ACI 318: 2019, AISC 360: 2022, Eurocode: rewriting 2025-2028). This is permanent maintenance — but AI makes updating faster too, since it can diff old and new code provisions and generate the changes for human review.

### What we don't build

**Construction drawings (Revit/Tekla territory):** generating plans, sections, and shop drawings requires a full CAD/BIM engine with drafting tools, dimensioning, annotation, sheet management, and print layout. This is a separate product category worth billions of dollars and decades of development. We integrate with these tools via IFC round-trip and live link plugins — we do not replace them. The boundary is clear: Dedaliano does engineering calculations, Revit/Tekla does construction documents.

This means Dedaliano covers ~80-85% of what a structural engineer does. The remaining ~15-20% is BIM modeling and drafting, which we connect to rather than replace. No single tool covers 100% — but owning the engineering calculation pipeline from loads to report is where the intellectual value and the revenue are.
