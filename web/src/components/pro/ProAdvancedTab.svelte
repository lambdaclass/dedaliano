<script lang="ts">
  import { modelStore, resultsStore } from '../../lib/store';
  import { t } from '../../lib/i18n';
  import {
    isSolverReady,
    solvePDelta3D,
    solveModal3D,
    solveBuckling3D,
    solveSpectral3D,
    solveTimeHistory3D,
    solvePlastic3D,
    solveCorotational3D,
    solveFiberNonlinear3D,
    solveWinkler3D,
    solveSSI3D,
    solveContact3D,
    solveStaged3D,
    solveCreepShrinkage3D,
  } from '../../lib/engine/wasm-solver';
  import { buildSolverInput3D } from '../../lib/engine/solver-service';
  import { cirsoc103Spectrum } from '../../lib/engine/spectral';
  import type { DesignSpectrum } from '../../lib/engine/spectral';
  import { applyRigidDiaphragm, detectFloorLevels } from '../../lib/engine/rigid-diaphragm';
  import { generateWindLoads } from '../../lib/engine/wind-loads';
  import type { WindParams } from '../../lib/engine/wind-loads';

  // Expose advanced results to parent via bindable props
  interface AdvancedResults3D {
    pdelta?: { converged: boolean; iterations: number; b2Factor?: number };
    modal?: { modes: Array<{ frequency: number; period: number; participationX?: number; participationY?: number; participationZ?: number }>; totalMass?: number };
    buckling?: { factors: number[] };
    spectral?: { baseShearX?: number; baseShearY?: number; baseShearZ?: number };
  }
  let { advancedResults = $bindable({}) }: { advancedResults: AdvancedResults3D } = $props();

  let solving = $state(false);
  let solveError = $state<string | null>(null);

  const hasModel = $derived(modelStore.nodes.size > 0 && modelStore.elements.size > 0);
  const wasmAvailable = $derived(isSolverReady());
  const elementIds = $derived([...modelStore.elements.keys()]);
  const nodeIds = $derived([...modelStore.nodes.keys()]);

  function fmtNum(n: number): string {
    if (n === 0) return '0';
    if (Math.abs(n) < 0.001) return n.toExponential(2);
    if (Math.abs(n) < 1) return n.toFixed(4);
    return n.toFixed(2);
  }

  // ─── Shared helpers ────────────────────────────────────────────

  let useDiaphragm = $state(false);
  let includeSelfWeight = $state(false);

  function buildInput() {
    const input = buildSolverInput3D(
      { nodes: modelStore.nodes, elements: modelStore.elements, supports: modelStore.supports,
        loads: modelStore.loads, materials: modelStore.materials, sections: modelStore.sections },
      includeSelfWeight,
    );
    if (!input) throw new Error('Modelo insuficiente para construir entrada del solver');
    return input;
  }

  function getMaterialDensities(): Map<number, number> {
    const densities = new Map<number, number>();
    for (const [id, mat] of modelStore.materials) {
      densities.set(id, (mat as any).rho ?? 0);
    }
    return densities;
  }

  function maybeApplyDiaphragm(input: any) {
    if (!useDiaphragm) return input;
    const levels = detectFloorLevels(input);
    if (!levels || levels.length === 0) return input;
    return applyRigidDiaphragm(input, { levels });
  }

  // ─── 1. P-Delta ─────────────────────────────────────────────────

  let pdeltaResult = $state<any | null>(null);

  function handlePDelta() {
    solveError = null;
    solving = true;
    try {
      let input = buildInput();
      input = maybeApplyDiaphragm(input);
      const res = solvePDelta3D(input);
      pdeltaResult = res;
      if (res.results) {
        resultsStore.setResults3D(res.results);
      }
      advancedResults = { ...advancedResults, pdelta: { converged: res.converged, iterations: res.iterations, b2Factor: res.b2Factor } };
    } catch (e: any) {
      solveError = `P-Delta: ${e.message ?? 'Error'}`;
    }
    solving = false;
  }

  // ─── 2. Modal ───────────────────────────────────────────────────

  let modalResult = $state<any | null>(null);
  let numModes = $state(6);

  function handleModal() {
    solveError = null;
    solving = true;
    try {
      let input = buildInput();
      input = maybeApplyDiaphragm(input);
      const densities = getMaterialDensities();
      const res = solveModal3D(input, densities, numModes);
      modalResult = res;
      if (res.modes || res.frequencies) {
        const modes = (res.modes ?? res.frequencies ?? []).map((m: any, i: number) => ({
          frequency: m.frequency ?? m.freq ?? (res.frequencies?.[i] ?? 0),
          period: m.period ?? (m.frequency ? 1 / m.frequency : 0),
          participationX: m.participationX ?? m.partX,
          participationY: m.participationY ?? m.partY,
          participationZ: m.participationZ ?? m.partZ,
        }));
        advancedResults = { ...advancedResults, modal: { modes, totalMass: res.totalMass } };
      }
    } catch (e: any) {
      solveError = `Modal: ${e.message ?? 'Error'}`;
    }
    solving = false;
  }

  // ─── 3. Spectral ───────────────────────────────────────────────

  let spectralResult = $state<any | null>(null);
  let spectralCombination = $state<'CQC' | 'SRSS'>('CQC');
  let seismicZone = $state<1 | 2 | 3 | 4>(3);
  let soilType = $state<'I' | 'II' | 'III'>('II');

  function handleSpectral() {
    solveError = null;
    solving = true;
    try {
      if (!modalResult) {
        solveError = 'Espectral: ejecutar análisis modal primero';
        solving = false;
        return;
      }
      let input = buildInput();
      input = maybeApplyDiaphragm(input);
      const densities = getMaterialDensities();
      const spectrum: DesignSpectrum = cirsoc103Spectrum(seismicZone, soilType);
      const res = solveSpectral3D({
        solver: input,
        densities,
        spectrum,
        directions: ['X', 'Y', 'Z'],
        combination: spectralCombination,
        numModes,
      });
      spectralResult = res;
      advancedResults = { ...advancedResults, spectral: { baseShearX: res.baseShearX ?? res.baseShear, baseShearY: res.baseShearY, baseShearZ: res.baseShearZ } };
    } catch (e: any) {
      solveError = `Espectral: ${e.message ?? 'Error'}`;
    }
    solving = false;
  }

  // ─── 4. Buckling ───────────────────────────────────────────────

  let bucklingResult = $state<any | null>(null);
  let numBucklingModes = $state(4);

  function handleBuckling() {
    solveError = null;
    solving = true;
    try {
      let input = buildInput();
      input = maybeApplyDiaphragm(input);
      const res = solveBuckling3D(input, numBucklingModes);
      bucklingResult = res;
      const factors = res.factors ?? res.eigenvalues ?? (res.modes?.map((m: any) => m.factor ?? m.eigenvalue) ?? []);
      advancedResults = { ...advancedResults, buckling: { factors } };
    } catch (e: any) {
      solveError = `Buckling: ${e.message ?? 'Error'}`;
    }
    solving = false;
  }

  // ─── 5. Wind loads ─────────────────────────────────────────────

  let windV = $state(45);
  let windExposure = $state<'B' | 'C' | 'D'>('B');
  let windWidth = $state(10);
  let windDir = $state<'X' | 'Y'>('X');
  let windResult = $state<any | null>(null);
  let windApplied = $state(false);

  function handleWindLoads() {
    solveError = null;
    try {
      const params: WindParams = { V: windV, exposure: windExposure };
      const res = generateWindLoads(
        modelStore.nodes as Map<number, { id: number; x: number; y: number; z?: number }>,
        params, windDir, windWidth,
      );
      windResult = res;
    } catch (e: any) {
      solveError = `Viento: ${e.message ?? 'Error'}`;
    }
  }

  function applyWindToModel() {
    if (!windResult?.nodalForces?.length) return;
    let windCaseId = modelStore.model.loadCases.find(c => c.type === 'W')?.id;
    if (!windCaseId) {
      windCaseId = modelStore.addLoadCase(`Viento ${windDir} (V=${windV})`, 'W');
    } else {
      modelStore.model.loads = modelStore.model.loads.filter(l => (l.data.caseId ?? 1) !== windCaseId);
    }
    for (const f of windResult.nodalForces) {
      modelStore.addNodalLoad3D(f.nodeId, f.Fx ?? 0, f.Fy ?? 0, f.Fz ?? 0, 0, 0, 0, windCaseId);
    }
    windApplied = true;
  }

  // ─── 6. Time History ──────────────────────────────────────────

  let thDt = $state(0.01);
  let thNSteps = $state(200);
  let thDir = $state<'X' | 'Y'>('X');
  let thDamping = $state(0.05);
  let thMethod = $state<'newmark' | 'hht'>('newmark');
  let thAccelText = $state('');
  let thResult = $state<any | null>(null);
  let thUseSine = $state(false);
  let thSineAmp = $state(0.3);
  let thSineFreq = $state(2.0);

  function generateSineAccel(): number[] {
    const vals: number[] = [];
    for (let i = 0; i < thNSteps; i++) {
      vals.push(thSineAmp * Math.sin(2 * Math.PI * thSineFreq * i * thDt));
    }
    return vals;
  }

  function parseAccelInput(): number[] {
    if (thUseSine) return generateSineAccel();
    return thAccelText.split(/[,\s]+/).filter(s => s.length > 0).map(Number).filter(n => !isNaN(n));
  }

  function handleTimeHistory() {
    solveError = null;
    solving = true;
    try {
      const groundAccel = parseAccelInput();
      if (groundAccel.length === 0) {
        solveError = 'Time History: se requiere al menos un valor de aceleración';
        solving = false;
        return;
      }
      let input = buildInput();
      input = maybeApplyDiaphragm(input);
      const densities: Record<string, number> = {};
      for (const [id, mat] of modelStore.materials) {
        densities[String(id)] = (mat as any).rho ?? 0;
      }
      const beta = 0.25;
      const gamma = 0.5;
      const res = solveTimeHistory3D({
        solver: input,
        densities,
        timeStep: thDt,
        nSteps: thNSteps,
        method: thMethod,
        beta,
        gamma,
        dampingXi: thDamping,
        groundAccel,
        groundDirection: thDir,
      });
      thResult = res;
    } catch (e: any) {
      solveError = `Time History: ${e.message ?? 'Error'}`;
    }
    solving = false;
  }

  // ─── 7. Nonlinear ─────────────────────────────────────────────

  let nlType = $state<'pushover' | 'corotational' | 'fiber'>('pushover');
  let nlMaxHinges = $state(20);
  let nlMaxIter = $state(50);
  let nlTol = $state(1e-6);
  let nlIncrements = $state(10);
  let nlFiberIntPts = $state(5);
  let nlResult = $state<any | null>(null);

  function handleNonlinear() {
    solveError = null;
    solving = true;
    try {
      let input = buildInput();
      input = maybeApplyDiaphragm(input);

      if (nlType === 'pushover') {
        const sections: Record<string, any> = {};
        for (const [id, sec] of modelStore.sections) {
          sections[String(id)] = {
            a: (sec as any).area ?? (sec as any).a ?? 0,
            iy: (sec as any).iy ?? (sec as any).Iy ?? 0,
            iz: (sec as any).iz ?? (sec as any).Iz ?? 0,
            materialId: (sec as any).materialId ?? 0,
            b: (sec as any).b ?? (sec as any).width ?? 0,
            h: (sec as any).h ?? (sec as any).height ?? 0,
          };
        }
        const materials: Record<string, any> = {};
        for (const [id, mat] of modelStore.materials) {
          materials[String(id)] = { fy: (mat as any).fy ?? 250 };
        }
        nlResult = solvePlastic3D({
          solver: input,
          sections,
          materials,
          maxHinges: nlMaxHinges,
        });
      } else if (nlType === 'corotational') {
        nlResult = solveCorotational3D(input, nlMaxIter, nlTol, nlIncrements);
      } else {
        const fiberSections: Record<string, any> = {};
        for (const [id, sec] of modelStore.sections) {
          fiberSections[String(id)] = {
            a: (sec as any).area ?? (sec as any).a ?? 0,
            iy: (sec as any).iy ?? (sec as any).Iy ?? 0,
            iz: (sec as any).iz ?? (sec as any).Iz ?? 0,
            materialId: (sec as any).materialId ?? 0,
            b: (sec as any).b ?? (sec as any).width ?? 0,
            h: (sec as any).h ?? (sec as any).height ?? 0,
          };
        }
        nlResult = solveFiberNonlinear3D({
          solver: input,
          fiberSections,
          nIntegrationPoints: nlFiberIntPts,
          maxIter: nlMaxIter,
          tolerance: nlTol,
          nIncrements: nlIncrements,
        });
      }
    } catch (e: any) {
      solveError = `No lineal: ${e.message ?? 'Error'}`;
    }
    solving = false;
  }

  // ─── 8. Winkler Foundation ─────────────────────────────────────

  let winklerElementId = $state<number | null>(null);
  let winklerKy = $state(1000);
  let winklerKz = $state(0);
  let winklerSprings = $state<{ elementId: number; ky: number; kz: number }[]>([]);
  let winklerResult = $state<any | null>(null);

  function addWinklerSpring() {
    if (winklerElementId == null) return;
    winklerSprings = [...winklerSprings, { elementId: winklerElementId, ky: winklerKy, kz: winklerKz }];
  }

  function removeWinklerSpring(idx: number) {
    winklerSprings = winklerSprings.filter((_, i) => i !== idx);
  }

  function handleWinkler() {
    solveError = null;
    solving = true;
    try {
      const input = buildInput();
      winklerResult = solveWinkler3D({
        solver: input,
        foundationSprings: winklerSprings.map(s => ({
          elementId: s.elementId,
          ...(s.ky ? { ky: s.ky } : {}),
          ...(s.kz ? { kz: s.kz } : {}),
        })),
      });
    } catch (e: any) {
      solveError = `Winkler: ${e.message ?? 'Error'}`;
    }
    solving = false;
  }

  // ─── 9. SSI ────────────────────────────────────────────────────

  let ssiNodeId = $state<number | null>(null);
  let ssiDirection = $state<'Y' | 'Z'>('Y');
  let ssiCurveType = $state<'softClay' | 'sand' | 'stiffClay' | 'custom'>('softClay');
  let ssiSu = $state(50);
  let ssiGamma = $state(18);
  let ssiDiameter = $state(0.6);
  let ssiDepth = $state(5);
  let ssiPhi = $state(30);
  let ssiTribLength = $state(1);
  let ssiMaxIter = $state(50);
  let ssiTolerance = $state(1e-4);
  let ssiSprings = $state<any[]>([]);
  let ssiResult = $state<any | null>(null);

  function addSsiSpring() {
    if (ssiNodeId == null) return;
    const params: any = { type: ssiCurveType };
    if (ssiCurveType === 'softClay' || ssiCurveType === 'stiffClay') {
      params.su = ssiSu; params.gamma = ssiGamma; params.d = ssiDiameter; params.depth = ssiDepth;
    } else if (ssiCurveType === 'sand') {
      params.phi = ssiPhi; params.gamma = ssiGamma; params.d = ssiDiameter; params.depth = ssiDepth;
    }
    ssiSprings = [...ssiSprings, { nodeId: ssiNodeId, direction: ssiDirection, curve: params, tributaryLength: ssiTribLength }];
  }

  function removeSsiSpring(idx: number) {
    ssiSprings = ssiSprings.filter((_, i) => i !== idx);
  }

  function handleSSI() {
    solveError = null;
    solving = true;
    try {
      const input = buildInput();
      ssiResult = solveSSI3D({
        solver: input,
        soilSprings: ssiSprings,
        maxIter: ssiMaxIter,
        tolerance: ssiTolerance,
      });
    } catch (e: any) {
      solveError = `SSI: ${e.message ?? 'Error'}`;
    }
    solving = false;
  }

  // ─── 10. Contact / Gap ─────────────────────────────────────────

  let contactBehaviors = $state<Map<number, 'normal' | 'tensionOnly' | 'compressionOnly'>>(new Map());
  let contactElementId = $state<number | null>(null);
  let contactBehavior = $state<'normal' | 'tensionOnly' | 'compressionOnly'>('tensionOnly');
  let contactResult = $state<any | null>(null);

  function setContactBehavior() {
    if (contactElementId == null) return;
    const next = new Map(contactBehaviors);
    next.set(contactElementId, contactBehavior);
    contactBehaviors = next;
  }

  function removeContactBehavior(eid: number) {
    const next = new Map(contactBehaviors);
    next.delete(eid);
    contactBehaviors = next;
  }

  const contactEntries = $derived([...contactBehaviors.entries()]);

  function handleContact() {
    solveError = null;
    solving = true;
    try {
      const input = buildInput();
      const elements: any[] = [];
      for (const [elementId, behavior] of contactBehaviors) {
        elements.push({ elementId, behavior });
      }
      contactResult = solveContact3D({ solver: input, contactElements: elements });
    } catch (e: any) {
      solveError = `Contacto: ${e.message ?? 'Error'}`;
    }
    solving = false;
  }

  // ─── 11. Staged Construction ───────────────────────────────────

  let stages = $state<{ name: string; addElements: number[]; removeElements: number[]; loadIndices: number[] }[]>([]);
  let stagedResult = $state<any | null>(null);

  function addStage() {
    stages = [...stages, { name: t('pro.stageN').replace('{n}', String(stages.length + 1)), addElements: [], removeElements: [], loadIndices: [] }];
  }

  function removeStage(idx: number) {
    stages = stages.filter((_, i) => i !== idx);
  }

  function handleStaged() {
    solveError = null;
    solving = true;
    try {
      const input = buildInput();
      stagedResult = solveStaged3D({
        solver: input,
        stages: stages.map(s => ({ addElements: s.addElements, removeElements: s.removeElements, loadIndices: s.loadIndices })),
      });
    } catch (e: any) {
      solveError = `Etapas: ${e.message ?? 'Error'}`;
    }
    solving = false;
  }

  // ─── 12. Creep & Shrinkage ─────────────────────────────────────

  let creepFc = $state(30);
  let creepRH = $state(60);
  let creepH0 = $state(200);
  let creepAge = $state(28);
  let creepCementClass = $state<'R' | 'N' | 'S'>('N');
  let creepTimeSteps = $state<{ time: number; additionalLoadFactor: number }[]>([
    { time: 365, additionalLoadFactor: 0 },
  ]);
  let creepResult = $state<any | null>(null);

  function addCreepStep() {
    const lastTime = creepTimeSteps.length > 0 ? creepTimeSteps[creepTimeSteps.length - 1].time : 0;
    creepTimeSteps = [...creepTimeSteps, { time: lastTime + 365, additionalLoadFactor: 0 }];
  }

  function removeCreepStep(idx: number) {
    creepTimeSteps = creepTimeSteps.filter((_, i) => i !== idx);
  }

  function handleCreep() {
    solveError = null;
    solving = true;
    try {
      const input = buildInput();
      creepResult = solveCreepShrinkage3D({
        solver: input,
        concrete: { fc: creepFc, rh: creepRH, h0: creepH0, ageAtLoading: creepAge, cementClass: creepCementClass },
        timeSteps: creepTimeSteps,
      });
    } catch (e: any) {
      solveError = `Fluencia: ${e.message ?? 'Error'}`;
    }
    solving = false;
  }
</script>

<div class="adv-tab">
  <!-- Global options -->
  <div class="adv-header">
    <label class="adv-check">
      <input type="checkbox" bind:checked={includeSelfWeight} />
      Peso propio
    </label>
    <label class="adv-check">
      <input type="checkbox" bind:checked={useDiaphragm} />
      Diafragma rígido
    </label>
    {#if !wasmAvailable}
      <span class="adv-wasm-warn">{t('pro.wasmNotReady')}</span>
    {/if}
  </div>

  {#if solveError}
    <div class="adv-error">{solveError}</div>
  {/if}

  <div class="adv-scroll">

    <!-- ── 1. P-Delta ── -->
    <div class="adv-group">
      <div class="adv-row">
        <button class="adv-run-btn" onclick={handlePDelta} disabled={!hasModel || solving || !wasmAvailable}>P-Delta</button>
        <span class="adv-desc">Efectos de segundo orden</span>
      </div>
      {#if pdeltaResult}
        <div class="adv-inline">
          {pdeltaResult.converged ? 'Convergió' : 'No convergió'} en {pdeltaResult.iterations} iter.
          {#if pdeltaResult.b2Factor != null} — B2 = {fmtNum(pdeltaResult.b2Factor)}{/if}
        </div>
      {/if}
    </div>

    <!-- ── 2. Modal ── -->
    <div class="adv-group">
      <div class="adv-row">
        <button class="adv-run-btn" onclick={handleModal} disabled={!hasModel || solving || !wasmAvailable}>Modal</button>
        <label class="adv-label">
          Modos:
          <input type="number" class="adv-num" bind:value={numModes} min={1} max={50} />
        </label>
      </div>
      {#if modalResult}
        <div class="adv-inline">
          {#if modalResult.totalMass != null}Masa: {fmtNum(modalResult.totalMass)} kg — {/if}
          {modalResult.modes?.length ?? 0} modos
        </div>
        <div class="adv-table-scroll">
          <table class="adv-table">
            <thead><tr><th>Modo</th><th>f (Hz)</th><th>T (s)</th><th>Part. X</th><th>Part. Y</th><th>Part. Z</th></tr></thead>
            <tbody>
              {#each modalResult.modes as mode, i}
                <tr>
                  <td class="col-id">{i + 1}</td>
                  <td class="col-num">{fmtNum(mode.frequency)}</td>
                  <td class="col-num">{fmtNum(mode.period)}</td>
                  <td class="col-num">{fmtNum(mode.participationX ?? 0)}</td>
                  <td class="col-num">{fmtNum(mode.participationY ?? 0)}</td>
                  <td class="col-num">{fmtNum(mode.participationZ ?? 0)}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>

    <!-- ── 3. Spectral ── -->
    <div class="adv-group">
      <div class="adv-row">
        <button class="adv-run-btn" onclick={handleSpectral} disabled={!hasModel || solving || !modalResult || !wasmAvailable}>Espectral</button>
        <label class="adv-label">
          <select class="adv-sel" bind:value={spectralCombination}>
            <option value="CQC">CQC</option>
            <option value="SRSS">SRSS</option>
          </select>
        </label>
        <label class="adv-label">
          Zona:
          <select class="adv-sel" bind:value={seismicZone}>
            <option value={1}>1</option><option value={2}>2</option><option value={3}>3</option><option value={4}>4</option>
          </select>
        </label>
        <label class="adv-label">
          Suelo:
          <select class="adv-sel" bind:value={soilType}>
            <option value="I">I</option><option value="II">II</option><option value="III">III</option>
          </select>
        </label>
      </div>
      {#if !modalResult}
        <div class="adv-hint">Requiere análisis modal previo</div>
      {/if}
      {#if spectralResult}
        <div class="adv-inline">
          {#if spectralResult.baseShear != null}
            Vb: X={fmtNum(spectralResult.baseShear.x ?? 0)}, Y={fmtNum(spectralResult.baseShear.y ?? 0)} kN
          {/if}
        </div>
        {#if spectralResult.perMode}
          <div class="adv-table-scroll">
            <table class="adv-table">
              <thead><tr><th>Modo</th><th>Sa (g)</th></tr></thead>
              <tbody>
                {#each spectralResult.perMode as pm, i}
                  <tr>
                    <td class="col-id">{i + 1}</td>
                    <td class="col-num">{fmtNum(pm.sa ?? pm.Sa ?? 0)}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      {/if}
    </div>

    <!-- ── 4. Buckling ── -->
    <div class="adv-group">
      <div class="adv-row">
        <button class="adv-run-btn" onclick={handleBuckling} disabled={!hasModel || solving || !wasmAvailable}>Pandeo</button>
        <label class="adv-label">
          Modos:
          <input type="number" class="adv-num" bind:value={numBucklingModes} min={1} max={20} />
        </label>
      </div>
      {#if bucklingResult}
        <div class="adv-table-scroll">
          <table class="adv-table">
            <thead><tr><th>Modo</th><th>&#x03BB;cr</th></tr></thead>
            <tbody>
              {#each bucklingResult.modes as mode, i}
                <tr>
                  <td class="col-id">{i + 1}</td>
                  <td class="col-num">{fmtNum(mode.loadFactor)}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>

    <!-- ── 5. Wind Loads ── -->
    <div class="adv-group">
      <div class="adv-row">
        <span class="adv-title">Viento (CIRSOC 102)</span>
      </div>
      <div class="adv-form">
        <label class="adv-label">V (m/s): <input type="number" class="adv-num" bind:value={windV} min={10} max={120} step={1} /></label>
        <label class="adv-label">Exp: <select class="adv-sel" bind:value={windExposure}><option value="B">B</option><option value="C">C</option><option value="D">D</option></select></label>
        <label class="adv-label">Ancho: <input type="number" class="adv-num" bind:value={windWidth} min={0.1} step={0.5} /></label>
        <label class="adv-label">Dir: <select class="adv-sel" bind:value={windDir}><option value="X">X</option><option value="Y">Y</option></select></label>
        <button class="adv-btn-sm" onclick={handleWindLoads} disabled={!hasModel}>Generar</button>
      </div>
      {#if windResult}
        <div class="adv-inline">
          {#if windResult.baseShear != null}Vb={fmtNum(windResult.baseShear)} kN{/if}
          {#if windResult.overturningMoment != null} — Mv={fmtNum(windResult.overturningMoment)} kN·m{/if}
          — {windResult.nodalForces?.length ?? 0} nodos
        </div>
        <button class="adv-btn-sm" onclick={applyWindToModel} disabled={windApplied}>
          {windApplied ? 'Aplicado' : 'Aplicar al modelo (caso W)'}
        </button>
        {#if windResult.steps?.length}
          <details>
            <summary class="adv-steps-toggle">Memoria de cálculo</summary>
            <div class="adv-steps-list">
              {#each windResult.steps as step}
                <div class="adv-step-line">{step}</div>
              {/each}
            </div>
          </details>
        {/if}
      {/if}
    </div>

    <!-- ── 6. Time History ── -->
    <details class="adv-group-details">
      <summary class="adv-title">Time History</summary>
      <div class="adv-panel">
        <div class="adv-form">
          <label class="adv-label">dt (s): <input type="number" class="adv-num" bind:value={thDt} min={0.001} max={1} step={0.001} /></label>
          <label class="adv-label">Pasos: <input type="number" class="adv-num adv-num-wide" bind:value={thNSteps} min={1} max={10000} /></label>
          <label class="adv-label">Dir: <select class="adv-sel" bind:value={thDir}><option value="X">X</option><option value="Y">Y</option></select></label>
          <label class="adv-label">&#x03BE;: <input type="number" class="adv-num" bind:value={thDamping} min={0} max={1} step={0.01} /></label>
          <label class="adv-label">Método: <select class="adv-sel" bind:value={thMethod}><option value="newmark">Newmark</option><option value="hht">HHT-&#x03B1;</option></select></label>
        </div>
        <label class="adv-check">
          <input type="checkbox" bind:checked={thUseSine} />
          Seno de prueba
        </label>
        {#if thUseSine}
          <div class="adv-form">
            <label class="adv-label">Amp (g): <input type="number" class="adv-num" bind:value={thSineAmp} min={0.01} step={0.05} /></label>
            <label class="adv-label">Freq (Hz): <input type="number" class="adv-num" bind:value={thSineFreq} min={0.1} step={0.1} /></label>
          </div>
        {:else}
          <div class="adv-accel-area">
            <label class="adv-label">Aceleración (separada por comas):</label>
            <textarea class="adv-textarea" bind:value={thAccelText} rows="2" placeholder="0.1, 0.25, 0.4, 0.3, -0.1, ..."></textarea>
          </div>
        {/if}
        <button class="adv-run-btn" onclick={handleTimeHistory} disabled={!hasModel || solving || !wasmAvailable}>Ejecutar</button>
      </div>
      {#if thResult}
        <div class="adv-inline">
          {#if thResult.peakDisplacement != null}δmax={fmtNum(thResult.peakDisplacement)} m{/if}
          {#if thResult.peakBaseShear != null} — Vb={fmtNum(thResult.peakBaseShear)} kN{/if}
          {#if thResult.timeAtPeak != null} — t={fmtNum(thResult.timeAtPeak)} s{/if}
        </div>
      {/if}
    </details>

    <!-- ── 7. Nonlinear ── -->
    <details class="adv-group-details">
      <summary class="adv-title">No lineal</summary>
      <div class="adv-panel">
        <div class="adv-form">
          <label class="adv-label">Tipo: <select class="adv-sel" bind:value={nlType}><option value="pushover">Pushover</option><option value="corotational">Corotacional</option><option value="fiber">Fibra</option></select></label>
          {#if nlType === 'pushover'}
            <label class="adv-label">Max rótulas: <input type="number" class="adv-num adv-num-wide" bind:value={nlMaxHinges} min={1} max={200} /></label>
          {:else}
            <label class="adv-label">Max iter: <input type="number" class="adv-num" bind:value={nlMaxIter} min={1} max={500} /></label>
            <label class="adv-label">Tol: <input type="number" class="adv-num adv-num-wide" bind:value={nlTol} min={1e-12} max={1} step={1e-6} /></label>
            <label class="adv-label">Incr: <input type="number" class="adv-num" bind:value={nlIncrements} min={1} max={200} /></label>
          {/if}
          {#if nlType === 'fiber'}
            <label class="adv-label">Pts int: <input type="number" class="adv-num" bind:value={nlFiberIntPts} min={2} max={20} /></label>
          {/if}
        </div>
        <button class="adv-run-btn" onclick={handleNonlinear} disabled={!hasModel || solving || !wasmAvailable}>Ejecutar</button>
      </div>
      {#if nlResult}
        <div class="adv-inline">
          {#if nlResult.converged != null}{nlResult.converged ? 'Convergió' : 'No convergió'}{/if}
          {#if nlResult.loadFactor != null} — λ={fmtNum(nlResult.loadFactor)}{/if}
          {#if nlResult.maxDisplacement != null} — δmax={fmtNum(nlResult.maxDisplacement)} m{/if}
          {#if nlResult.numHinges != null} — {nlResult.numHinges} rótulas{/if}
        </div>
      {/if}
    </details>

    <!-- ─── Divider: Modelado especial ─── -->
    <div class="adv-divider">Modelado especial</div>

    <!-- ── 8. Winkler Foundation ── -->
    <details class="adv-group-details">
      <summary class="adv-title">{t('pro.winklerFoundation')}</summary>
      <div class="adv-panel">
        <div class="adv-form">
          <label class="adv-label">{t('pro.element')}:
            <select class="adv-sel" bind:value={winklerElementId}>
              <option value={null}>--</option>
              {#each elementIds as eid}<option value={eid}>{eid}</option>{/each}
            </select>
          </label>
          <label class="adv-label">ky (kN/m/m): <input type="number" class="adv-num" bind:value={winklerKy} min={0} step={100} /></label>
          <label class="adv-label">kz: <input type="number" class="adv-num" bind:value={winklerKz} min={0} step={100} /></label>
          <button class="adv-btn-sm" onclick={addWinklerSpring} disabled={winklerElementId == null}>+</button>
        </div>
        {#if winklerSprings.length > 0}
          <table class="adv-table">
            <thead><tr><th>Elem</th><th>ky</th><th>kz</th><th></th></tr></thead>
            <tbody>
              {#each winklerSprings as s, i}
                <tr>
                  <td class="col-id">{s.elementId}</td>
                  <td class="col-num">{fmtNum(s.ky)}</td>
                  <td class="col-num">{fmtNum(s.kz)}</td>
                  <td><button class="adv-rm" onclick={() => removeWinklerSpring(i)}>x</button></td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
        <button class="adv-run-btn" onclick={handleWinkler} disabled={!hasModel || solving || !wasmAvailable || winklerSprings.length === 0}>{solving ? t('pro.solving') : t('pro.solveWinkler')}</button>
      </div>
      {#if winklerResult}
        <div class="adv-result-title">{t('pro.resultWinkler')}</div>
        <div class="adv-inline">
          <span>{t('pro.convergence')}: {winklerResult.converged ? t('pro.yes') : t('pro.no')}</span> — <span>{t('pro.iterations')}: {winklerResult.iterations ?? '?'}</span>
          {#if winklerResult.maxDisplacement != null} — <span>{t('pro.maxDisp')}: {fmtNum(winklerResult.maxDisplacement)} m</span>{/if}
        </div>
      {/if}
    </details>

    <!-- ── 9. SSI ── -->
    <details class="adv-group-details">
      <summary class="adv-title">{t('pro.ssiTitle')}</summary>
      <div class="adv-panel">
        <div class="adv-form">
          <label class="adv-label">{t('pro.ssiNode')}:
            <select class="adv-sel" bind:value={ssiNodeId}>
              <option value={null}>--</option>
              {#each nodeIds as nid}<option value={nid}>{nid}</option>{/each}
            </select>
          </label>
          <label class="adv-label">Dir: <select class="adv-sel" bind:value={ssiDirection}><option value="Y">Y</option><option value="Z">Z</option></select></label>
          <label class="adv-label">{t('pro.ssiCurve')}:
            <select class="adv-sel" bind:value={ssiCurveType}>
              <option value="softClay">{t('pro.softClay')}</option>
              <option value="stiffClay">{t('pro.stiffClay')}</option>
              <option value="sand">{t('pro.sand')}</option>
              <option value="custom">{t('pro.customCurve')}</option>
            </select>
          </label>
        </div>
        {#if ssiCurveType === 'softClay' || ssiCurveType === 'stiffClay'}
          <div class="adv-form">
            <label class="adv-label">su (kPa): <input type="number" class="adv-num" bind:value={ssiSu} min={0} step={5} /></label>
            <label class="adv-label">&#947; (kN/m3): <input type="number" class="adv-num" bind:value={ssiGamma} min={0} step={1} /></label>
            <label class="adv-label">d (m): <input type="number" class="adv-num" bind:value={ssiDiameter} min={0.1} step={0.1} /></label>
            <label class="adv-label">{t('pro.depth')}: <input type="number" class="adv-num" bind:value={ssiDepth} min={0} step={0.5} /></label>
          </div>
        {:else if ssiCurveType === 'sand'}
          <div class="adv-form">
            <label class="adv-label">&#966; (deg): <input type="number" class="adv-num" bind:value={ssiPhi} min={0} max={50} step={1} /></label>
            <label class="adv-label">&#947; (kN/m3): <input type="number" class="adv-num" bind:value={ssiGamma} min={0} step={1} /></label>
            <label class="adv-label">d (m): <input type="number" class="adv-num" bind:value={ssiDiameter} min={0.1} step={0.1} /></label>
            <label class="adv-label">{t('pro.depth')}: <input type="number" class="adv-num" bind:value={ssiDepth} min={0} step={0.5} /></label>
          </div>
        {/if}
        <div class="adv-form">
          <label class="adv-label">{t('pro.tribLength')}: <input type="number" class="adv-num" bind:value={ssiTribLength} min={0.1} step={0.5} /></label>
          <button class="adv-btn-sm" onclick={addSsiSpring} disabled={ssiNodeId == null}>{t('pro.addSpring')}</button>
        </div>
        {#if ssiSprings.length > 0}
          <table class="adv-table">
            <thead><tr><th>Nodo</th><th>Dir</th><th>Curva</th><th>L</th><th></th></tr></thead>
            <tbody>
              {#each ssiSprings as s, i}
                <tr>
                  <td class="col-id">{s.nodeId}</td>
                  <td class="col-num">{s.direction}</td>
                  <td class="col-num">{s.curve.type}</td>
                  <td class="col-num">{fmtNum(s.tributaryLength)}</td>
                  <td><button class="adv-rm" onclick={() => removeSsiSpring(i)}>x</button></td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
        <div class="adv-form">
          <label class="adv-label">{t('pro.maxIter')}: <input type="number" class="adv-num" bind:value={ssiMaxIter} min={1} max={500} /></label>
          <label class="adv-label">{t('pro.tolerance')}: <input type="number" class="adv-num" bind:value={ssiTolerance} min={1e-8} step={1e-5} /></label>
        </div>
        <button class="adv-run-btn" onclick={handleSSI} disabled={!hasModel || solving || !wasmAvailable || ssiSprings.length === 0}>{solving ? t('pro.solving') : t('pro.solveSsi')}</button>
      </div>
      {#if ssiResult}
        <div class="adv-result-title">{t('pro.resultSsi')}</div>
        <div class="adv-inline">
          <span>{t('pro.convergence')}: {ssiResult.converged ? t('pro.yes') : t('pro.no')}</span> — <span>{t('pro.iterations')}: {ssiResult.iterations ?? '?'}</span>
          {#if ssiResult.maxDisplacement != null} — <span>{t('pro.maxDisp')}: {fmtNum(ssiResult.maxDisplacement)} m</span>{/if}
        </div>
      {/if}
    </details>

    <!-- ── 10. Contact / Gap ── -->
    <details class="adv-group-details">
      <summary class="adv-title">{t('pro.contactGap')}</summary>
      <div class="adv-panel">
        <div class="adv-form">
          <label class="adv-label">{t('pro.element')}:
            <select class="adv-sel" bind:value={contactElementId}>
              <option value={null}>--</option>
              {#each elementIds as eid}<option value={eid}>{eid}</option>{/each}
            </select>
          </label>
          <label class="adv-label">{t('pro.behavior')}:
            <select class="adv-sel" bind:value={contactBehavior}>
              <option value="normal">{t('pro.normal')}</option>
              <option value="tensionOnly">{t('pro.tensionOnly')}</option>
              <option value="compressionOnly">{t('pro.compressionOnly')}</option>
            </select>
          </label>
          <button class="adv-btn-sm" onclick={setContactBehavior} disabled={contactElementId == null}>+</button>
        </div>
        {#if contactEntries.length > 0}
          <table class="adv-table">
            <thead><tr><th>Elem</th><th>Comportamiento</th><th></th></tr></thead>
            <tbody>
              {#each contactEntries as [eid, beh]}
                <tr>
                  <td class="col-id">{eid}</td>
                  <td class="col-num">{beh === 'tensionOnly' ? t('pro.tensionOnly') : beh === 'compressionOnly' ? t('pro.compressionOnly') : t('pro.normal')}</td>
                  <td><button class="adv-rm" onclick={() => removeContactBehavior(eid)}>x</button></td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
        <button class="adv-run-btn" onclick={handleContact} disabled={!hasModel || solving || !wasmAvailable || contactEntries.length === 0}>{solving ? t('pro.solving') : t('pro.solveContact')}</button>
      </div>
      {#if contactResult}
        <div class="adv-result-title">{t('pro.resultContact')}</div>
        <div class="adv-inline">
          <span>{t('pro.convergence')}: {contactResult.converged ? t('pro.yes') : t('pro.no')}</span> — <span>{t('pro.iterations')}: {contactResult.iterations ?? '?'}</span>
          {#if contactResult.deactivated} — <span>{t('pro.deactivatedElems')}: {contactResult.deactivated.length}</span>{/if}
        </div>
      {/if}
    </details>

    <!-- ── 11. Staged Construction ── -->
    <details class="adv-group-details">
      <summary class="adv-title">{t('pro.stagedConstruction')}</summary>
      <div class="adv-panel">
        <button class="adv-btn-sm" onclick={addStage}>{t('pro.addStage')}</button>
        {#each stages as stage, i}
          <div class="adv-stage-card">
            <div class="adv-stage-header">
              <input type="text" class="adv-stage-name" bind:value={stage.name} />
              <button class="adv-rm" onclick={() => removeStage(i)}>x</button>
            </div>
            <div class="adv-form">
              <label class="adv-label">{t('pro.addElemIds')} <input type="text" class="adv-text" value={stage.addElements.join(',')} oninput={(e) => { stage.addElements = (e.target as HTMLInputElement).value.split(',').map(Number).filter(n => !isNaN(n) && n > 0); stages = [...stages]; }} /></label>
            </div>
            <div class="adv-form">
              <label class="adv-label">{t('pro.removeElemIds')} <input type="text" class="adv-text" value={stage.removeElements.join(',')} oninput={(e) => { stage.removeElements = (e.target as HTMLInputElement).value.split(',').map(Number).filter(n => !isNaN(n) && n > 0); stages = [...stages]; }} /></label>
            </div>
            <div class="adv-form">
              <label class="adv-label">{t('pro.loadIndices')}: <input type="text" class="adv-text" value={stage.loadIndices.join(',')} oninput={(e) => { stage.loadIndices = (e.target as HTMLInputElement).value.split(',').map(Number).filter(n => !isNaN(n) && n >= 0); stages = [...stages]; }} /></label>
            </div>
          </div>
        {/each}
        <button class="adv-run-btn" onclick={handleStaged} disabled={!hasModel || solving || !wasmAvailable || stages.length === 0}>{solving ? t('pro.solving') : t('pro.solveStaged')}</button>
      </div>
      {#if stagedResult}
        <div class="adv-result-title">{t('pro.resultStaged')}</div>
        <div class="adv-inline">
          {#if stagedResult.stages}
            {#each stagedResult.stages as sr, i}
              <div>{t('pro.stageN').replace('{n}', String(i + 1))}: {sr.converged != null ? (sr.converged ? t('pro.ok') : t('pro.noConv')) : t('pro.solved')}</div>
            {/each}
          {/if}
          {#if stagedResult.totalDisplacement != null} <span>{t('pro.totalMaxDisp')}: {fmtNum(stagedResult.totalDisplacement)} m</span>{/if}
        </div>
      {/if}
    </details>

    <!-- ── 12. Creep & Shrinkage ── -->
    <details class="adv-group-details">
      <summary class="adv-title">{t('pro.creepShrinkage')}</summary>
      <div class="adv-panel">
        <div class="adv-form">
          <label class="adv-label">f'c (MPa): <input type="number" class="adv-num" bind:value={creepFc} min={10} max={100} step={5} /></label>
          <label class="adv-label">HR (%): <input type="number" class="adv-num" bind:value={creepRH} min={20} max={100} step={5} /></label>
          <label class="adv-label">h0 (mm): <input type="number" class="adv-num" bind:value={creepH0} min={50} max={2000} step={10} /></label>
        </div>
        <div class="adv-form">
          <label class="adv-label">{t('pro.loadingAge')}: <input type="number" class="adv-num" bind:value={creepAge} min={1} max={10000} /></label>
          <label class="adv-label">{t('pro.cementClass')}: <select class="adv-sel" bind:value={creepCementClass}><option value="R">{t('pro.cementR')}</option><option value="N">{t('pro.cementN')}</option><option value="S">{t('pro.cementS')}</option></select></label>
        </div>
        <div class="adv-sub-title">{t('pro.timeSteps')}</div>
        {#each creepTimeSteps as step, i}
          <div class="adv-form">
            <label class="adv-label">{t('pro.timeDays')}: <input type="number" class="adv-num" bind:value={step.time} min={1} /></label>
            <label class="adv-label">{t('pro.addLoadFactor')}: <input type="number" class="adv-num" bind:value={step.additionalLoadFactor} min={0} step={0.1} /></label>
            <button class="adv-rm" onclick={() => removeCreepStep(i)}>x</button>
          </div>
        {/each}
        <button class="adv-btn-sm" onclick={addCreepStep}>{t('pro.addStep')}</button>
        <button class="adv-run-btn" onclick={handleCreep} disabled={!hasModel || solving || !wasmAvailable || creepTimeSteps.length === 0}>{solving ? t('pro.calculating') : t('pro.calcCreep')}</button>
      </div>
      {#if creepResult}
        <div class="adv-result-title">{t('pro.resultCreep')}</div>
        <div class="adv-inline">
          {#if creepResult.phiCreep != null}{t('pro.creepCoeff')}: {fmtNum(creepResult.phiCreep)}{/if}
          {#if creepResult.shrinkageStrain != null} — {t('pro.shrinkageStrain')}: {creepResult.shrinkageStrain.toExponential(2)}{/if}
          {#if creepResult.maxDisplacement != null} — {t('pro.finalMaxDisp')}: {fmtNum(creepResult.maxDisplacement)} m{/if}
        </div>
      {/if}
    </details>

  </div>
</div>

<style>
  .adv-tab {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .adv-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    background: #0d1b33;
    border-bottom: 1px solid #1a3a5a;
    flex-shrink: 0;
    flex-wrap: wrap;
  }

  .adv-scroll {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
  }

  .adv-error {
    padding: 6px 12px;
    font-size: 0.72rem;
    color: #ff8a9e;
    background: rgba(233, 69, 96, 0.1);
    flex-shrink: 0;
  }

  .adv-wasm-warn {
    font-size: 0.65rem;
    color: #ffa726;
  }

  .adv-check {
    font-size: 0.7rem;
    color: #aaa;
    display: flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
  }

  .adv-check input { cursor: pointer; }

  /* Groups — each analysis type */
  .adv-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 8px 12px;
    border-bottom: 1px solid #1a3050;
  }

  .adv-group-details {
    border-bottom: 1px solid #1a3050;
    padding: 6px 12px;
  }

  .adv-group-details > summary {
    list-style: none;
    cursor: pointer;
  }

  .adv-group-details > summary::-webkit-details-marker { display: none; }

  .adv-group-details > summary::before {
    content: '▸ ';
    font-size: 0.55rem;
    color: #666;
  }

  .adv-group-details[open] > summary::before {
    content: '▾ ';
  }

  .adv-row {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .adv-run-btn {
    padding: 5px 14px;
    font-size: 0.72rem;
    font-weight: 600;
    color: #fff;
    background: linear-gradient(135deg, #1a4a7a, #1a3860);
    border: 1px solid #4ecdc4;
    border-radius: 4px;
    cursor: pointer;
    white-space: nowrap;
  }

  .adv-run-btn:hover { background: linear-gradient(135deg, #1a5a9a, #1a4a7a); }
  .adv-run-btn:disabled { opacity: 0.35; cursor: not-allowed; }

  .adv-btn-sm {
    padding: 3px 10px;
    font-size: 0.68rem;
    font-weight: 600;
    color: #ccc;
    background: #1a3050;
    border: 1px solid #1a3050;
    border-radius: 3px;
    cursor: pointer;
  }

  .adv-btn-sm:hover { color: #fff; border-color: #4ecdc4; }
  .adv-btn-sm:disabled { opacity: 0.35; cursor: not-allowed; }

  .adv-title {
    font-size: 0.72rem;
    font-weight: 600;
    color: #4ecdc4;
    user-select: none;
  }

  .adv-desc {
    font-size: 0.62rem;
    color: #666;
    font-style: italic;
  }

  .adv-hint {
    font-size: 0.6rem;
    color: #665;
    font-style: italic;
  }

  .adv-inline {
    font-size: 0.68rem;
    color: #aaa;
    padding: 2px 0;
    font-family: monospace;
  }

  .adv-divider {
    padding: 6px 12px;
    font-size: 0.6rem;
    font-weight: 600;
    color: #556;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    background: #0a1a30;
    border-bottom: 1px solid #1a3050;
  }

  /* Forms */
  .adv-form {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    align-items: center;
  }

  .adv-label {
    font-size: 0.68rem;
    color: #888;
    display: flex;
    align-items: center;
    gap: 4px;
    white-space: nowrap;
  }

  .adv-num {
    width: 55px;
    padding: 3px 5px;
    font-size: 0.68rem;
    background: #0a1a30;
    border: 1px solid #1a3050;
    border-radius: 3px;
    color: #ccc;
    text-align: right;
  }

  .adv-num-wide { width: 70px; }

  .adv-sel {
    padding: 3px 5px;
    font-size: 0.68rem;
    background: #0a1a30;
    border: 1px solid #1a3050;
    border-radius: 3px;
    color: #ccc;
    cursor: pointer;
  }

  .adv-text {
    width: 120px;
    padding: 3px 5px;
    font-size: 0.68rem;
    background: #0a1a30;
    border: 1px solid #1a3050;
    border-radius: 3px;
    color: #ccc;
  }

  .adv-panel {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 6px 0;
  }

  .adv-table-scroll {
    max-height: 150px;
    overflow-y: auto;
  }

  .adv-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.68rem;
  }

  .adv-table th {
    padding: 3px 5px;
    text-align: left;
    font-size: 0.6rem;
    font-weight: 600;
    color: #888;
    text-transform: uppercase;
    background: #0a1a30;
    border-bottom: 1px solid #1a3050;
  }

  .adv-table td {
    padding: 3px 5px;
    border-bottom: 1px solid #0f2030;
    color: #ccc;
  }

  .col-id { color: #666; font-family: monospace; text-align: center; }
  .col-num { font-family: monospace; text-align: right; font-size: 0.66rem; }

  .adv-rm {
    padding: 2px 6px;
    font-size: 0.62rem;
    color: #e94560;
    background: transparent;
    border: 1px solid #e94560;
    border-radius: 3px;
    cursor: pointer;
    line-height: 1;
  }

  .adv-rm:hover { background: rgba(233, 69, 96, 0.15); }

  .adv-accel-area {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .adv-textarea {
    width: 100%;
    padding: 4px 6px;
    font-size: 0.64rem;
    font-family: monospace;
    background: #0f2030;
    border: 1px solid #1a3050;
    border-radius: 3px;
    color: #ccc;
    resize: vertical;
    min-height: 32px;
  }

  .adv-textarea::placeholder { color: #555; }

  .adv-steps-toggle {
    font-size: 0.6rem;
    color: #8ba;
    cursor: pointer;
  }

  .adv-step-line {
    font-size: 0.58rem;
    color: #9ab;
    padding: 1px 0;
  }

  .adv-sub-title {
    font-size: 0.66rem;
    font-weight: 600;
    color: #aaa;
    margin-top: 4px;
  }

  .adv-stage-card {
    background: #0a1a30;
    border: 1px solid #1a3050;
    border-radius: 4px;
    padding: 6px 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .adv-stage-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .adv-stage-name {
    flex: 1;
    padding: 3px 5px;
    font-size: 0.68rem;
    font-weight: 600;
    background: transparent;
    border: none;
    border-bottom: 1px solid #1a3050;
    color: #ccc;
  }

  .adv-stage-name:focus { border-color: #4ecdc4; outline: none; }
</style>
