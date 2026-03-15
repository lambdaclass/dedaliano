# Dedaliano Product Roadmap

## Purpose

This is the product roadmap: app features, market sequencing, design/reporting layers, and distribution strategy. It is not the solver mechanics roadmap — for that, see `SOLVER_ROADMAP.md`. Historical progress belongs in `CHANGELOG.md`. This document should stay forward-looking.

## Vision

Become the world's best structural engineering software — open-source, browser-based, combining the analytical power of OpenSees/Code_Aster/CalculiX with zero-install accessibility, explainable diagnostics, and eventually the collaboration and AI layers no competitor can match.

## Competitive Moat

What we have that OpenSees/Code_Aster/CalculiX/SAP2000/ETABS will never have:

1. **Zero-install browser UX** — engineers open a URL and start working
2. **Visual model building** — competitors require scripting or clunky pre-processors
3. **Real-time feedback** — live calc, instant diagrams, interactive 3D
4. **Educational mode** — no competitor explains the math step by step
5. **Modern stack** — their codebases are Fortran/Tcl/C++ from the 90s
6. **Explainable diagnostics and review workflows** — warnings, provenance, result trust, comments, and guided fixes as first-class product surfaces
7. **AI-assisted engineering UX** — guided modeling, result queries, and code/design suggestions built on structured solver outputs
8. **CRDT collaboration** — later Figma-style real-time multi-user editing on top of a trusted structural core

## The Sequence

### 1. Solver-Led Product

Build trust through a reliable, accessible structural solver with strong diagnostics and first design outputs.

**What:**
- WASM path reliability — single trusted solver runtime in production
- RC beam design and reinforcement schedule (envelopes, required steel, selected bars, stirrups, schedule-ready output)
- Report and calculation-document foundations
- Onboarding and first-solve success
- Richer diagnostics UX — grouping, filtering, provenance, click-to-focus highlighting
- AI-assisted modeling and review — explain warnings, suggest missing supports/loads, flag suspicious patterns, guide first-fix actions
- Lightweight collaboration — comments, pinned annotations, shared links, model/version diff, reviewer read-only flows
- Constraint-force and governing-result presentation
- Shell-family recommendation and automatic defaults (MITC4/MITC9/SHB8-ANS)
- Public benchmark and acceptance-model presentation
- Performance feedback in the UI — progress bars, iteration counts, slow-phase visibility

**Done when:** An engineer can model a structure in the browser, get trustworthy results with clear diagnostics, produce an RC beam schedule, and share a read-only link with a reviewer.

### 2. Deliverable Layer

Turn analysis into paid engineering work with design checks, reports, and interoperability.

**What:**
- Graphical BBS drawing generation — bending-shape drawings, dimensions, hook semantics
- Multi-code design check UI — EC2, EC3, ACI 318, AISC 360, NDS, TMS 402, AISI S100 wired to unified code-selector with per-member utilization ratios
- Connections and foundations productization — auto-sizing and detail generation
- Automatic load generation — wind (EC1/ASCE 7), seismic ELF (EC8/ASCE 7), snow, live load patterns from code parameters
- Reports and calculation packages — PDF with LaTeX equations, project info, design checks, diagrams
- Interoperability — full IFC import/export, DXF 3D
- Project and template support — reusable workflows, firm standardization
- AI-powered section suggestion — suggest optimal sections from utilization ratios with code references
- AI-powered load combination from code selection — auto-generate required combinations including accidental torsion, pattern loading, combination factors
- Natural language result queries — "what's the max moment in beam 7?", "which column has the highest utilization?"

**Done when:** An engineer can run a design check against their national code, generate a submission-grade PDF report, import/export IFC, and ask the app questions about results in plain language.

### 3. Dynamic and Nonlinear Layer

Make the browser the go-to tool for earthquake engineering, replacing OpenSees for the common 80% of work.

**What:**
- Dynamic time-history UI (Newmark-beta, HHT-alpha, ground motion input)
- Pushover analysis (capacity spectrum, N2, MPA)
- Nonlinear material editors (concrete, steel, fiber sections)
- RC section builder (visual concrete shape + rebar layout)
- Moment-curvature and interaction diagrams
- Cyclic material testing with hysteresis visualization
- Construction staging UI
- Seismic workflow end-to-end (spectra, ground motion selection, IDA)
- AI-powered nonlinear/dynamic result interpretation — explain convergence, flag unusual hysteresis, detect soft-story mechanisms, suggest damping parameters
- AI ground motion selection — suggest appropriate records from site parameters and target spectrum

**Done when:** An earthquake engineer can run a pushover analysis, select ground motions, perform IDA, and get AI-explained results — all in the browser without writing a single line of Tcl.

### 4. Workflow and Ecosystem Layer

Fit into real firm workflows and broaden adoption with scripting, desktop, education, and additional codes.

**What:**
- SAP2000/ETABS import (.s2k), STAAD.Pro import (.std), Robot exchange format
- OpenSees import (.tcl parser subset) for migration
- Python scripting API (Pyodide in-browser) for batch runs and parametric studies
- REST API — headless solver for CI integration and automated analysis
- Tauri desktop packaging — same web app as a local desktop app for offline use, local files, native integration
- Education and benchmark explorer — university course integration, homework templates, interactive benchmark viewer
- Additional design codes — timber (EC5/NDS, CLT, glulam), masonry (EC6/TMS 402, confined masonry), composite (EC4/AISC, metal deck, headed studs, precast)
- Slab and floor design — punching shear (EC2/ACI), flat slab strips, post-tensioned tendon layout, waffle/ribbed slabs
- Architect-friendly conceptual mode — early-stage structural feedback with defaults, visual feedback, and guardrails

Desktop principles:
- Web remains the primary product surface
- Desktop is a shared shell, not a forked product
- Local file access, offline use, and native integration are the main value
- Auto-update from signed GitHub releases or equivalent signed update feed

**Done when:** A firm can import their existing SAP2000 models, run batch parametric studies via Python, work offline on the desktop app, and design timber/masonry/composite members against their national code.

### 5. Platform Layer

Turn the app into a real-time collaborative platform — the Figma of structural engineering.

**What:**
- CRDT-based real-time collaboration — structural model as CRDT document (Yjs or Automerge), structural-aware merge semantics, awareness protocol (live cursors, selection highlights), WebRTC peer-to-peer sync with WebSocket relay fallback, offline-first editing with automatic merge, per-user undo, branch and merge for models, operational history and audit trail
- User roles and permissions (viewer, editor, reviewer, approver)
- Comments and annotations pinned to nodes/elements/regions
- Visual diff between model versions
- Project management — version history, review workflow, project dashboard
- Cost estimation — material quantities to cost (steel tonnage, concrete volume, rebar weight, formwork)
- Enterprise controls — permissions, audit trail, administration
- Optimization and parametric design — size/shape/topology optimization (SIMP), parameter sweeps, multi-objective Pareto, code-constrained optimization
- Natural language to model — "8-storey RC frame, seismic zone 4, soft soil" generates a complete structural model
- Automated design iteration — AI runs hundreds of variants, presents Pareto-optimal designs (cost vs weight vs drift vs carbon)
- GNN/neural operator surrogates — train on solver output for 1000x parametric speedup
- PWA and offline — installable Progressive Web App, mobile-optimized 3D viewer, offline sync via CRDTs

**Done when:** A team of engineers can work on the same model simultaneously with live cursors, branch/merge design alternatives, and an AI can generate and rank hundreds of structural variants automatically.

### 6. Specialized Analysis

Cover the remaining 20% of specialized analysis that advanced users need.

**What:**
- Progressive collapse — GSA/UFC alternate path method, automatic member removal, dynamic amplification, catenary action
- Fire design — ISO 834/ASTM E119 curves, temperature-degraded material properties (EC2/EC3/EC4), thermal analysis, fire resistance rating, parametric fire curves (EC1-1-2)
- Performance-based seismic — IDA automation, fragility curves, FEMA P-58 loss estimation, ML-accelerated IDA surrogates, multi-stripe analysis
- Advanced elements — full catenary, form-finding (force density, dynamic relaxation), membrane/fabric/cable-net, tapered beams, curved beams, 3D solid elements (hex, tet)
- Reliability and probabilistic — Monte Carlo, FORM/SORM, subset simulation, polynomial chaos expansion
- Digital twins and SHM — sensor data ingestion API, Bayesian model updating
- Cloud solve — server-side WASM/native solver for 100k+ DOF models
- Performance at scale — WebGPU compute shaders, sparse iterative solvers (PCG/GMRES), Web Workers, IndexedDB, binary format

**Done when:** An engineer can run a progressive collapse check, a fire resistance analysis, and a probabilistic seismic risk assessment on a 100k+ DOF model — all within the same app.

### 7. Software Built On The Solver

Turn the solver from an application into a software stack structural firms can live inside.

**What:**
- RC design + BBS studio — analysis to required steel to selected bars to schedules to drawings
- Structural report OS — report-grade outputs, submission documents, issue-ready calculation books
- QA / peer-review assistant — structural review workflows, suspicious-result detection, model-quality review
- Firm workspace — standards, templates, reusable office defaults, collaboration, project memory
- Parametric configurator — building, industrial, and foundation generators
- Interoperability + cloud comparison — shared workspaces, batch runs, model diffing, scenario comparisons

**Done when:** A firm can run their entire structural workflow inside the app — from parametric generation through analysis, design, QA review, and submission-grade deliverables — without switching tools.

### 8. AI-Native Structural Engineering

Ship the AI capabilities that need the full solver depth (dynamic, materials, pushover, batch execution).

**What:**
- Full natural language to model — complex structures with nonlinear materials, staged construction, seismic design parameters
- AI design assistant at scale — watches modeling in real-time across hundreds of members, understands nonlinear behavior, suggests retrofit strategies
- Automated design iteration with Pareto — AI runs hundreds of variants using batch parametric runner, presents Pareto-optimal designs
- GNN surrogates in production — neural operators replacing full solves in design exploration, IDA acceleration, topology optimization
- LLM-powered code compliance — "does this design satisfy EC8 for ductility class high?" answered by reading the full nonlinear model and referencing specific code clauses
- Anomaly detection at depth — detect soft-story mechanisms, torsional irregularities, connection inadequacy, progressive collapse vulnerability
- Reinforcement learning for design — RL agent learns structural design by trial and error against the solver

**Done when:** The app can autonomously generate, analyze, and rank structural design alternatives while an engineer focuses on judgment, and an AI can answer complex code-compliance questions by reasoning over nonlinear analysis results.

### 9. Real-Time Collaborative Engineering

Make structural engineering a real-time team activity.

**What:**
- Incremental re-analysis — solver re-analyzes only the affected region when one user changes a node
- Structural-aware conflict resolution — CRDT merge semantics that understand structural dependencies
- Live review mode — senior engineer sees the junior's model updating in real-time with live utilization ratios, annotates and approves in-place
- Branching and what-if — branch a structural model like git, explore alternatives, merge with full diff visualization
- Multi-cursor design — multiple engineers on different parts of the same building simultaneously
- Async review workflows — comments pinned to elements/nodes/load cases, assigned review tasks, approval tracking

**Done when:** A senior engineer can watch a junior's model evolve in real-time, annotate issues, approve changes, and merge design branches — all without passing files back and forth.

### 10. Generative Structural Design

Shift design from "engineer proposes one solution and checks it" to "AI generates the solution space and engineer selects the best option."

**What:**
- System generation — "design me a 40m span roof" generates 50 topologically distinct structural systems, analyzes all, ranks by weight/cost/constructability/carbon
- Buildable topology optimization — SIMP/BESO that outputs real member sizes, connection feasibility, manufacturing constraints
- Parametric form-finding — architect drags a shape, structure optimizes in real-time
- Multi-objective Pareto exploration — interactive Pareto front trading off cost, weight, drift, carbon, constructability with live model preview
- Code-constrained generation — AI only proposes designs that satisfy the selected building code from the start

**Done when:** An engineer can describe a structural problem in plain language and receive dozens of code-compliant, buildable design alternatives ranked across multiple objectives.

### 11. Construction Intelligence

Bridge the gap between structural design and construction.

**What:**
- 4D BIM integration — structural model tied to construction schedule, staged loading simulation, construction sequence visualization
- Automated rebar detailing — analysis results to shop drawings with zero human intervention (schedules, placing drawings, splice locations, development lengths)
- Formwork optimization — minimize pours, optimize table reuse, plan striking sequence from early-age strength predictions
- Digital twin construction loop — site sensor data, Bayesian model updating, next-day deflection predictions, shoring adjustments
- As-built model calibration — compare surveyed geometry against design, flag deviations, update analysis with as-built dimensions

**Done when:** A contractor can feed site sensor data into the model, get daily deflection predictions, and receive automatically generated rebar shop drawings that account for as-built conditions.

### 12. Planetary-Scale Infrastructure

Help humanity build climate-resilient, low-carbon, reusable infrastructure.

**What:**
- Climate-resilient design — automated scenario generation from climate models (future wind speeds, flood levels, fire risk), structures that survive 2050/2080 climate
- Embodied carbon optimization — minimize CO2 alongside cost and safety, material passport integration, LCA built into the design loop
- Circular economy design — design for disassembly, reuse scoring for members, material bank integration
- Automated retrofit assessment — LiDAR/photogrammetry to FE model, seismic/wind vulnerability assessment, automated retrofit proposals
- Portfolio risk assessment — entire building portfolios for seismic/wind/flood risk, insurance-grade loss estimation at city scale

**Done when:** A city can upload its building portfolio, get a seismic/climate risk assessment, and receive prioritized retrofit recommendations with embodied carbon tradeoffs.

### 13. Education Platform

Replace static textbooks with interactive learning.

**What:**
- Interactive textbook mode — students see stiffness assembly, equation solving, force recovery step by step with explanations
- AI tutor — explains why a structure failed and what to change, teaches structural intuition
- Exam/homework mode — professor defines constraints, student designs, solver auto-grades
- Benchmark explorer — reproduce every published structural benchmark interactively
- Curriculum integration — pre-built course modules for structural analysis, steel design, RC design, dynamics

**Done when:** A professor can assign a structural design homework, students solve it interactively in the browser, and the app auto-grades while explaining the structural behavior step by step.

### 14. API Economy and Platform Ecosystem

Become the structural engineering operating system.

**What:**
- Stabileo as infrastructure — other applications call the solver via REST/WebSocket API
- Insurance and risk — seismic/wind risk on entire building portfolios through the API
- City planning — urban planning tools check structural feasibility in real-time
- Parametric design backends — Grasshopper, Dynamo, Blender use Stabileo as the analysis engine
- Plugin/extension marketplace — third-party developers build specialized tools on top
- Foundation models for structural engineering — pre-trained on millions of analyzed structures
- Autonomous inspection pipeline — drone damage capture, CV crack detection, Bayesian model updating, remaining life prediction

**Done when:** Third-party developers can build and sell specialized structural tools on a Stabileo marketplace, and insurance companies can run portfolio-scale risk assessments through the API.

## Non-Goals

Do not prioritize these before the core product is clearly trusted:

- Generic multiphysics expansion
- CFD or thermal-fluid products
- Overly broad enterprise features before single-user workflow quality is strong
- AI features that outrun solver trust
- Architect-friendly conceptual mode before onboarding, diagnostics, deliverables, and interoperability are stronger
- GPU sparse direct factorization (CPU sparse direct is correct for structural problem sizes)
- Isogeometric analysis (IGA shines for automotive/aerospace, not buildings)
- Meshfree methods (peridynamics, MPM — niche, out of scope for routine structural)

## Related Docs

- `README.md` — repo entry point and document map
- `SOLVER_ROADMAP.md` — solver mechanics and validation sequencing
- `POSITIONING.md` — market framing and competitive wedge
- `research/rc_design_and_bbs.md` — RC design and BBS research
- `research/beyond_roadmap_opportunities.md` — GNN surrogates, digital twins, UQ, advanced research opportunities
- `research/cypecad_parity_roadmap.md` — CYPECAD feature parity analysis and wiring plan
- `research/post_roadmap_software_stack.md` — platform adjacencies after roadmap execution
