# Competitive Displacement by Roadmap Step

## Purpose

This document maps each solver roadmap step to the commercial and open-source tools it displaces, the license cost savings for engineering firms, and what specifically enables the switch. The goal is to help users understand when Stabileo becomes a viable replacement for their current toolchain.

Pricing is based on publicly available information as of early 2025. Actual costs vary by region, reseller, edition, and negotiation.

## Competitor Pricing Reference

| Software | Vendor | Annual cost per seat | Notes |
|----------|--------|---------------------|-------|
| Karamba3D Pro | Karamba3D | ~$1,200 | Grasshopper/Rhino plugin for parametric structural |
| CalculiX | Open source | Free | GPL, no commercial support |
| OpenSees | UC Berkeley | Free | BSD license, research-focused |
| RFEM 6 | Dlubal | ~$3,000 | Base subscription; add-on modules extra |
| SAP2000 | CSI | ~$5,000-$15,600 initial + $350-$2,730/yr maintenance | Standard to Ultimate editions |
| ETABS | CSI | ~$5,000-$15,600 initial + $875-$2,730/yr maintenance | Standard to Ultimate editions |
| Robot Structural Analysis | Autodesk | ~$3,675 | Only available inside AEC Collection |
| STAAD.Pro | Bentley | ~$3,200-$4,400 | Virtuosity subscription |
| Abaqus | Dassault Systemes | ~$18,000-$19,000/yr lease | Perpetual ~$31,000-$37,000 + ~$8,500/yr maintenance |
| OptiStruct / Tosca | Altair / Dassault | ~$15,000-$30,000+/yr (est.) | Enterprise pricing, not public |

Sources: CSI reseller sites, Dlubal webshop, Autodesk AEC Collection pricing, Bentley Virtuosity, Fidelis FEA cost guides, Altair units documentation, forum reports.

## Displacement Timeline

### Step 3 — Structured Diagnostics

**Displaces:** Karamba3D

**Why:** Karamba3D has no solver diagnostics, no reproducibility, and no structured warnings. Stabileo becomes the only browser-native structural tool with real solver trust signals, design code checks, and a validated multi-family shell stack. Students and parametric designers who outgrow Karamba's simplified analysis have a direct upgrade path.

**Savings per seat:** ~$1,200/yr
**5-seat firm:** ~$6,000/yr

---

### Steps 4-5 — Runtime Dominance + Verification Moat

**Displaces:** CalculiX (for structural work)

**Why:** CalculiX has no browser path, no design codes, a weaker shell stack, and no verification transparency. Stabileo is faster on structural-size models with better shells and visible proof of correctness. Engineers who use CalculiX because "it's free and it works" get a better free option.

**Savings per seat:** $0 (CalculiX is free)
**Value:** better UX, shells, design codes, and verification — same price

---

### Steps 6-7 — Nonlinear Hardening + Shell Maturity

**Displaces:** RFEM / Dlubal (basic workflows)

**Why:** For firms doing linear and second-order steel/concrete frames with shell floors, Stabileo now covers the common 80% of daily work: better shells, design codes (AISC, ACI, EC2, EC3, CIRSOC), structured diagnostics, zero cost, browser access. RFEM retains advantages on reports, auto load generation, and Wood-Armer moments — but those are addressed in later steps.

**Savings per seat:** ~$3,000/yr
**5-seat firm:** ~$15,000/yr

---

### Step 8 — Dynamic Analysis

**Displaces:** OpenSees (education and common linear/mildly-nonlinear time-history)

**Why:** For the common 60% of earthquake engineering education and practice (linear and mildly-nonlinear time-history on frame structures), Stabileo now works — with a visual interface, in a browser. University courses switch. New earthquake engineers learn Stabileo first. OpenSees retains deep nonlinear, force-based beams, and 30 years of material models.

**Savings per seat:** $0 (OpenSees is free)
**Value:** visual interface, zero-install, real-time feedback — replaces weeks of Tcl/Python scripting setup

---

### Step 9 — Nonlinear Materials

**Displaces:** OpenSees (80% of seismic practice)

**Why:** RC columns with confined concrete (Mander), steel frames with Bauschinger effect (Menegotto-Pinto), fiber sections with biaxial bending. The common earthquake engineering workflows now run in-browser with visual feedback. OpenSees retains researchers doing exotic materials and massive parametric studies.

**Savings per seat:** $0 (OpenSees is free)
**Value:** eliminates the scripting expertise barrier that limits OpenSees adoption

---

### Step 10 — Pushover Analysis

**Displaces:** SAP2000 / ETABS (seismic assessment)

**Why:** Pushover (capacity spectrum, N2, MPA) is the bread-and-butter of seismic evaluation firms. SAP2000/ETABS do it, but cost $5,000-$15,000/yr per seat and run on Windows. Stabileo does it free, in-browser, with transparent solver math. Seismic assessment firms — especially in Latin America, Southeast Asia, and Southern Europe where license costs are a significant burden — can switch.

**Savings per seat:** ~$5,000-$15,000/yr
**5-seat firm:** ~$25,000-$75,000/yr

---

### Step 11 — Advanced Element Library

**Displaces:** OpenSees (completely for structural engineering), Robot (partial)

**Why:** The force-based beam-column element was OpenSees' last unique advantage for nonlinear frame analysis. With seismic isolators, BRBs, and shell triangles (MITC3), performance-based design is fully covered. OpenSees survives only as a research scripting platform. Robot's advantage was Autodesk integration and meshing flexibility — the meshing gap (quad-only) is now closed with triangles.

**Savings per seat:** ~$3,675/yr (Robot)
**5-seat firm:** ~$18,375/yr (Robot)

---

### Step 12 — Native / Server Execution

**Displaces:** Robot Structural Analysis (completely)

**Why:** Engineering firms need batch processing and local execution for large projects. With native desktop (Tauri) + browser + identical solver, Stabileo replaces Robot's desktop workflow while keeping the browser advantage.

**Savings per seat:** ~$3,675/yr
**5-seat firm:** ~$18,375/yr

---

### Steps 13-14 — Thermal/Fire/Fatigue + Auto Load Generation

**Displaces:** RFEM / Dlubal (completely), STAAD.Pro

**Why:** The last RFEM advantages were auto load generation and specialized analysis (fire, fatigue). With automatic wind/seismic/snow load generation and fire/fatigue analysis, the full RFEM workflow is covered. STAAD.Pro's remaining value was code-based load generation and broad code support — now matched.

**Savings per seat:** ~$3,000-$4,400/yr
**5-seat firm:** ~$15,000-$22,000/yr

---

### Step 15 — Performance at Scale

**Displaces:** Abaqus (structural problems)

**Why:** Large shell/solid structural models (bridges, dams, offshore structures) that previously required Abaqus for scale now run in Stabileo with iterative solvers (AMG), multi-frontal solver, and WebGPU acceleration. Abaqus retains contact-heavy manufacturing workflows and multiphysics. Pure structural firms using Abaqus because "nothing else scales" can switch.

**Savings per seat:** ~$18,000-$19,000/yr
**5-seat firm:** ~$90,000-$95,000/yr

---

### Steps 18-19 — Contact Depth + Design Post-Processing

**Displaces:** Abaqus (structural contact), RFEM (slab design)

**Why:** Mortar contact + shell-to-solid coupling + embedded rebar covers structural contact use cases (connections, composite sections, RC detailing). Wood-Armer moments + punching shear + crack width estimation completes RC slab design from shell models. Stress linearization per ASME/EN 13445 opens pressure vessel assessment.

**Savings per seat:** included in Abaqus seat above
**New markets opened:** pressure vessel assessment, RC slab design from shell analysis

---

## Cumulative Savings (5-seat firm using multiple tools)

| After step | Tools replaced | Cumulative savings/yr |
|------------|---------------|----------------------|
| 3 | Karamba3D | ~$6,000 |
| 7 | + RFEM (basic) | ~$21,000 |
| 10 | + SAP2000 or ETABS | ~$46,000-$96,000 |
| 12 | + Robot | ~$64,000-$114,000 |
| 14 | + STAAD.Pro | ~$79,000-$136,000 |
| 15 | + Abaqus | ~$169,000-$231,000 |

## Additional Value Beyond License Savings

License cost is only part of the equation:

- **No IT overhead** — no license servers, no Windows-only machines, no annual renewal negotiations, no vendor lock-in
- **Instant onboarding** — new engineers open a browser tab instead of waiting for IT to provision a license
- **Unlimited seats** — open source means the 6th engineer doesn't cost another $5,000-$19,000
- **Transparent solver** — engineers can trace every computation, which matters for peer review and regulatory approval
- **Educational value** — the step-by-step DSM wizard has no equivalent in any tool at any price

## Related Docs

- `SOLVER_ROADMAP.md` — solver step definitions and done criteria
- `PRODUCT_ROADMAP.md` — product sequencing
- `research/open_source_solver_comparison.md` — detailed comparison with OpenSees, Code_Aster, Kratos
- `POSITIONING.md` — market framing and competitive strategy
