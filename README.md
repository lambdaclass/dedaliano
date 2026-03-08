<p align="center">
  <img src="Logo.png" alt="Dedaliano" width="180" />
</p>

<h1 align="center">Dedaliano</h1>

<p align="center">
  <strong>Open-source structural analysis for the browser.</strong><br>
  Model, solve, and visualize frame structures in 2D and 3D. No installation required.
</p>

<p align="center">
  <a href="https://dedaliano.com">Try it now</a> ·
  <a href="#what-is-structural-analysis">What is this</a> ·
  <a href="#why-dedaliano">Why it exists</a> ·
  <a href="#features">Features</a> ·
  <a href="#documentation-map">Docs</a> ·
  <a href="#getting-started">Getting started</a>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-AGPL--3.0-blue.svg" alt="License"></a>
</p>

<p align="center">
  <img src="screenshots/3d-deformed.jpg" alt="3D industrial warehouse showing deformed shape under load" width="100%" />
</p>
<p align="center"><sub>3D model of an industrial warehouse with Pratt roof trusses and a crane bridge. The orange overlay shows the deformed shape under applied loads, exaggerated for visibility. 216 nodes, 538 elements, 30 supports.</sub></p>

<p align="center">
  <img src="screenshots/3d-colormap.jpg" alt="Same structure with stress utilization color map" width="100%" />
</p>
<p align="center"><sub>Same structure with a stress utilization color map (sigma/fy). Blue elements are lightly loaded. Green and yellow elements are at moderate utilization. Red elements are approaching their yield strength.</sub></p>

---

## Documentation map

Use the docs by question:

- [`BENCHMARKS.md`](/Users/unbalancedparen/projects/dedaliano/BENCHMARKS.md)
  source of truth for solver capability, validation coverage, benchmark status, and remaining solver gaps
- [`ROADMAP.md`](/Users/unbalancedparen/projects/dedaliano/ROADMAP.md)
  product roadmap plus solver-first priority ordering and difficulty ladder
- [`POSITIONING.md`](/Users/unbalancedparen/projects/dedaliano/POSITIONING.md)
  product/business framing, adjacent markets, and platform direction
- [`engine/README.md`](/Users/unbalancedparen/projects/dedaliano/engine/README.md)
  Rust solver engine surface, analysis types, and engine-focused validation summary

This README is intentionally the short repo-level entry point.
It should explain what Dedaliano is and where to read next, not duplicate the benchmark ledger, roadmap tables, or market strategy in full.

## Current state

Dedaliano is an `open-source structural solver` with a growing structural engineering platform around it.

What exists today at a high level:

- browser-native modeling and visualization
- a substantial Rust structural solver in `engine/`
- 2D and 3D linear, second-order, dynamic, staged, cable, and nonlinear analysis paths
- design-check and postprocess modules for steel, concrete, timber, masonry, serviceability, connections, and foundations
- a large benchmark and validation program tracked in [`BENCHMARKS.md`](/Users/unbalancedparen/projects/dedaliano/BENCHMARKS.md)

The solver currently combines:

- broad structural analysis coverage
- serious nonlinear capability
- contact and soil-structure interaction
- staged construction and prestress / PT workflows
- warping torsion
- dynamic analysis
- strong postprocessing and design layers
- broad validation coverage

The main remaining differentiators are:

- robustness on hard real models
- performance at scale
- benchmark credibility
- workflow and modeling quality

A reasonable description today is:

`Dedaliano is a strong open structural solver with broad analysis coverage and a large public validation program.`

## What is structural analysis

Structural analysis computes how a structure deforms and what internal forces it develops under load.

In practice that means:

- modeling geometry, supports, materials, and loads
- solving for displacements and reactions
- recovering axial forces, shears, moments, stresses, modes, and related response quantities

For simple members this can be done by hand. For real frames, trusses, plates, staged systems, and dynamic problems, the structure is solved numerically, typically with the **[Direct Stiffness Method](https://en.wikipedia.org/wiki/Direct_stiffness_method)** and related finite-element formulations.

---

## Why Dedaliano

The dominant structural analysis tools today are commercial desktop applications: [SAP2000](https://www.csiamerica.com/products/sap2000) and [ETABS](https://www.csiamerica.com/products/etabs) from CSI, [Robot Structural Analysis](https://www.autodesk.com/products/robot-structural-analysis) from Autodesk, [RSTAB](https://www.dlubal.com/en/products/rstab-beam-structures/what-is-rstab) and [RFEM](https://www.dlubal.com/en/products/rfem-fea-software/what-is-rfem) from Dlubal. A professional license costs thousands of dollars per year. They run on Windows. They require installation, license servers, and IT support. Their source code is closed.

For students, this creates a gap. You learn the theory in class (equilibrium, compatibility, constitutive relations, the stiffness method), but you never see the inside of the machine. The commercial tools are black boxes: you input a model, press solve, and get results. You cannot inspect the stiffness matrix, see how it was assembled, verify a single entry, or understand why a particular element is failing. If the results look wrong, you have no way to trace the computation.

Dedaliano is an attempt to provide an alternative.

- **Browser-native.** Open [dedaliano.com](https://dedaliano.com) and start. No download, no license key, no account. Works offline after the first load.
- **Open source.** The entire codebase is here. Read the solver, trace the math, submit improvements.
- **Transparent computation.** A 9-step interactive wizard shows every intermediate result of the Direct Stiffness Method: the local stiffness matrix of each element, the coordinate transformation, the assembled global matrix, the partitioning, the solution, the back-substitution for reactions and internal forces. Every matrix is rendered with [KaTeX](https://katex.org).
- **Solver-first.** The project is organized around a real structural solver and a large public benchmark program, not only a modeling UI.
- **Real-time.** The solver runs on every edit. Move a node, change a load, resize a section, and the results update.

Originally built for structural engineering courses at [FIUBA](http://www.fi.uba.ar/) (University of Buenos Aires, School of Engineering).

Named after [Daedalus](https://en.wikipedia.org/wiki/Daedalus) (Daidalos), the architect of the labyrinth, who built wings to escape Crete.

---

## Features

### Modeling

Place nodes in 2D or 3D space. Connect them with frame elements (transmit axial force, shear, and bending moment) or truss elements (axial force only). Assign materials with preset or custom elastic properties. Assign cross-sections from a built-in catalog:

- **Steel profiles:** over 100 European sections (IPE, HEB, HEA, IPN, UPN, L, RHS, CHS)
- **Concrete sections:** rectangular, circular, T-beam, inverted L
- **Custom sections:** parametric builder for T, U, C, and compound shapes

Define supports: fixed (all DOF restrained), pinned (translations restrained, rotation free), roller (one translation restrained), spring (with custom stiffness in any DOF), inclined at any angle, or with prescribed displacements.

Internal hinges (moment releases) at element ends for modeling Gerber beams and other articulated structures. Hinges are implemented via static condensation: the released rotational DOF is eliminated from the element stiffness matrix before assembly.

In 3D mode, elements have configurable local axes. You can specify an orientation vector for the local Y axis or a roll angle (0, 90, 180, 270 degrees) to control the section rotation about the element axis.

### Loading

- **Nodal forces:** F_x, F_y, F_z, M_x, M_y, M_z at any node
- **Distributed loads:** uniform, trapezoidal, or partial along any element, in local or global directions
- **Point loads on elements:** concentrated force at any distance from the start node
- **Thermal loads:** uniform temperature change DeltaT (axial expansion: F = E A alpha DeltaT) and through-depth gradient DeltaT_g (curvature: M = E I alpha DeltaT_g / h)
- **Self-weight:** automatic calculation from material density, cross-section area, and gravity

Loads are organized into **load cases** following standard categories (dead D, live L, wind W, earthquake E, snow S, thermal T, roof live Lr, rain R, lateral earth H). Cases are combined into **load combinations** using LRFD factors per design codes (CIRSOC/ASCE 7). Default combinations include 1.2D + 1.6L, 1.4D, 1.2D + L + 1.6W, and 1.2D + L + E.

### Analysis

| Type | Description | 2D | 3D |
|---|---|:---:|:---:|
| **Linear static** | Solves **KU** = **F** for displacements and internal forces under applied loads | Yes | Yes |
| **P-Delta** | Iterative second-order analysis. Assembles geometric stiffness **K_G** from axial forces, solves (**K** + **K_G**)**U** = **F**, repeats until convergence (tolerance 1e-4, max 20 iterations). Returns amplification factor B_2. | Yes | |
| **Linear buckling** | Generalized eigenvalue problem **K** phi = lambda (-**K_G**) phi. Returns critical load factor, effective length, and slenderness for each element. | Yes | |
| **Modal** | Generalized eigenvalue problem **K** phi = omega^2 **M** phi. Returns natural frequencies, periods, mode shapes, participation factors, effective mass ratios. Consistent mass matrix from cubic shape functions. | Yes | |
| **Plastic** | Event-to-event incremental analysis. Detects when a section reaches M_p = f_y Z_p, inserts a hinge, redistributes forces. Continues until mechanism (collapse). Returns collapse load multiplier and hinge sequence. | Yes | |
| **Spectral** | Earthquake analysis using a response spectrum. Modal superposition with SRSS or CQC combination. Predefined CIRSOC 103 spectra (zones 1-4, soil types I-III) or user-defined. 5% default damping. | Yes | |
| **Moving loads** | Envelope of maximum/minimum effects from a train of loads traversing the structure | Yes | |
| **Influence lines** | Response at a specific location as a unit load moves across the structure | Yes | |

All analyses run in real time as the model is edited. The 2D solver operates at 3 DOF per node (u_x, u_y, theta_z). The 3D solver operates at 6 DOF per node (u_x, u_y, u_z, theta_x, theta_y, theta_z).

### Results and visualization

- **Deformed shape** with cubic Hermite interpolation between nodes (not just linear), giving smooth curvature even with coarse meshes. In 3D, deformation is interpolated independently in both bending planes (Y-plane for M_z/V_y and Z-plane for M_y/V_z) with particular solutions for intra-element loads computed via Simpson's rule integration (20 segments).
- **Support reactions** (forces and moments at every restrained DOF)
- **Internal force diagrams** plotted along every element: axial force N(x), shear force V(x), bending moment M(x) in 2D; additionally V_y, V_z, M_y, M_z, and torsion T in 3D. Each diagram is sampled at 21 points per element, with additional samples at discontinuity positions around point loads.
- **Envelope curves** showing the maximum and minimum of every response quantity across all load combinations, plotted as dual positive/negative curves
- **Load case overlay** for side-by-side comparison of different cases or combinations
- **Stress utilization color map** on the 3D model, colored by sigma/f_y or any other variable

### Cross-section stress analysis

For any element at any position along its length, Dedaliano computes the full stress state at the cross-section:

| Quantity | Method | Formula |
|---|---|---|
| Normal stress sigma | Navier | sigma = N/A +/- M y/I (2D); sigma = N/A + M_z y/I_z - M_y z/I_y (3D, sign from theta_y = -dw/dx convention) |
| Shear stress tau | Jourawski | tau = V Q(y) / (I b(y)), where Q is the first moment of area above fiber y and b is the width at y |
| Torsional shear tau_T | Saint-Venant (open sections) / Bredt (closed sections) | Open: tau_T = M_x t_max / J. Closed: tau_T = M_x / (2 A_m t), where A_m is the mean enclosed area |
| Principal stresses | Mohr's circle | sigma_1, sigma_3, tau_max from plane stress transformation (sigma_y = 0 for beam theory) |
| Failure check | Von Mises, Tresca, Rankine | sigma_vm = sqrt(sigma^2 + 3 tau^2) for 2D; sigma_vm = sqrt(sigma^2 + 3(tau_Vy^2 + tau_Vz^2 + tau_T^2)) for 3D |

Shear flow paths are computed per section type. For I/H sections: 5 paths (two top flanges converging inward, web downward, two bottom flanges diverging outward). For RHS: 6 paths with a closed-section correction q_0. For CHS: 2 semicircles with q_0 = 0 by symmetry.

Also computes and visualizes the **neutral axis** (the line of zero normal stress under biaxial bending: N/A - M_y y/I_z + M_z z/I_y = 0) and the **central core** (the region within which a compressive force produces no tension anywhere in the section).

### Educational tools

**DSM wizard.** A 9-step interactive walkthrough of the entire Direct Stiffness Method for the current model:

1. **DOF numbering.** Assign a global index to every free and supported degree of freedom.
2. **Local stiffness matrix.** Compute **k** for each element from E, A, I, and L.
3. **Coordinate transformation.** Rotate **k** to global coordinates: **K_e** = **T**^T **k** **T**.
4. **Global stiffness matrix.** Assemble all **K_e** into the global **K** by DOF mapping.
5. **Load vector.** Assemble nodal loads and fixed-end forces into **F**.
6. **Partitioning.** Separate **K** and **F** into free (f) and supported (s) subsets: **K_ff**, **K_fs**, **F_f**.
7. **Solution.** Solve **K_ff** U_f = **F_f** - **K_fs** U_s for the unknown displacements.
8. **Reactions.** Back-substitute: **R** = **K_sf** U_f + **K_ss** U_s - **F_s**.
9. **Internal forces.** For each element, recover local forces from global displacements: **f** = **k** **T** **u_e**.

Every matrix and vector is rendered with KaTeX. Entries are highlighted to show which element or DOF they correspond to.

**Kinematic analysis.** Automatic detection of mechanisms (structures with insufficient restraints or unfortunate hinge arrangements that allow rigid-body motion). The analysis checks the rank of the stiffness matrix, identifies collinear all-hinged configurations, and produces a diagnostic report explaining why the structure is unstable.

### Import and export

| Format | Import | Export | Notes |
|---|:---:|:---:|---|
| JSON | Yes | Yes | Native format. Preserves all model data, results, and UI state. |
| DXF | Yes | Yes | AutoCAD R12 (AC1009). Uses POLYLINE/VERTEX/SEQEND entities. Geometry on separate layers (Nodes, Elements, Supports, Loads). 2D only. |
| IFC | Yes | | BIM import via [web-ifc](https://github.com/ThatOpen/engine_web-ifc) WASM parser. Reads IFCBEAM, IFCCOLUMN, IFCMEMBER entities. Extracts placement matrices and ExtrudedAreaSolid geometry. 3D only. WASM module (3.5 MB) is dynamically imported only when the IFC dialog is opened, to avoid penalizing initial bundle size. |
| Excel | | Yes | Tabulated results in .xlsx format via [SheetJS](https://sheetjs.com). |
| PNG | | Yes | Viewport screenshot at current zoom and orientation. |
| URL | Yes | Yes | The full model state is serialized, compressed with [LZ-string](https://github.com/pieroxy/lz-string), and encoded in the URL hash. Share a link, and the recipient opens the exact model. Auto-solve on load is supported. |

---
## Getting started

**Use it now.** Open [dedaliano.com](https://dedaliano.com). Nothing to install. Works on any modern browser.

**Run locally.**

```bash
git clone https://github.com/lambdaclass/dedaliano.git
cd dedaliano/web
npm install
npm run dev       # http://localhost:4000
```

```bash
npm test          # run all tests (1,050+ cases)
npm run build     # production build -> web/dist/
```

Requires Node.js >= 18.

---

## Contributing

Pull requests are welcome. For major changes, open an issue first to discuss the approach.

## Security

To report a vulnerability, email security@lambdaclass.com.

## License

[AGPL-3.0](LICENSE)

---

<p align="center">
  <em>In honor of Daedalus, who built the labyrinth and dared to fly.</em>
</p>
