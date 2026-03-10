<script lang="ts">
  import { modelStore, uiStore, resultsStore } from '../../lib/store';
  import { runGlobalSolve } from '../../lib/engine/live-calc';

  let solveError = $state<string | null>(null);
  let includeSelfWeight = $state(false);
  let solving = $state(false);

  const results = $derived(resultsStore.results3D);
  const hasModel = $derived(modelStore.nodes.size > 0 && modelStore.elements.size > 0);
  const hasCombinations = $derived(resultsStore.hasCombinations3D);

  // View mode
  type ViewMode = 'single' | 'combo' | 'envelope';
  let viewMode = $state<ViewMode>('single');
  let selectedCaseId = $state<number | null>(null);
  let selectedComboId = $state<number | null>(null);

  function handleSolve() {
    solveError = null;
    solving = true;
    uiStore.includeSelfWeight = includeSelfWeight;
    try {
      // First solve single (all loads)
      runGlobalSolve();
      if (!resultsStore.results3D) {
        solveError = 'No se obtuvieron resultados';
        solving = false;
        return;
      }

      // Now solve combinations if load cases exist
      if (modelStore.loadCases.length > 0 && modelStore.combinations.length > 0) {
        try {
          const comboResult = modelStore.solveCombinations3D(includeSelfWeight);
          if (typeof comboResult === 'string') {
            console.warn('Combinations warning:', comboResult);
          } else if (comboResult) {
            resultsStore.setCombinationResults3D(comboResult.perCase, comboResult.perCombo, comboResult.envelope);
            viewMode = 'envelope';
          }
        } catch (comboErr: any) {
          console.warn('Combinations 3D failed (results still available):', comboErr);
        }
      }
    } catch (e: any) {
      console.error('PRO solve error:', e);
      solveError = e?.message || String(e) || 'Error desconocido';
    }
    solving = false;
  }

  function switchView(mode: ViewMode) {
    viewMode = mode;
    if (mode === 'envelope') {
      resultsStore.activeView = 'envelope';
    } else if (mode === 'combo' && selectedComboId !== null) {
      resultsStore.activeComboId = selectedComboId;
      resultsStore.activeView = 'combo';
    } else if (mode === 'single') {
      if (selectedCaseId !== null) {
        resultsStore.activeCaseId = selectedCaseId;
      } else {
        resultsStore.activeView = 'single';
      }
    }
  }

  function onCaseChange(e: Event) {
    const id = Number((e.target as HTMLSelectElement).value);
    selectedCaseId = id;
    resultsStore.activeCaseId = id;
  }

  function onComboChange(e: Event) {
    const id = Number((e.target as HTMLSelectElement).value);
    selectedComboId = id;
    resultsStore.activeComboId = id;
    resultsStore.activeView = 'combo';
  }

  function fmtNum(n: number): string {
    if (n === 0) return '0';
    if (Math.abs(n) < 0.001) return n.toExponential(2);
    if (Math.abs(n) < 1) return n.toFixed(4);
    return n.toFixed(2);
  }

  const caseKeys = $derived([...resultsStore.perCase3D.keys()]);
  const comboKeys = $derived([...resultsStore.perCombo3D.keys()]);

</script>

<div class="pro-res">
  <div class="pro-res-header">
    <div class="pro-res-solve-row">
      <button class="pro-solve-btn" onclick={handleSolve} disabled={!hasModel || solving}>
        {solving ? 'Calculando...' : 'Calcular'}
      </button>
      <label class="pro-sw-label">
        <input type="checkbox" bind:checked={includeSelfWeight} />
        Peso propio
      </label>
    </div>
    {#if solveError}
      <div class="pro-solve-error">{solveError}</div>
    {/if}
    {#if results}
      <span class="pro-res-status">Resuelto — {results.reactions.length} reacciones, {results.elementForces.length} barras</span>
    {/if}
  </div>

  {#if results}
    <!-- 3D Visualization controls -->
    <div class="pro-viz-section">
      <div class="pro-viz-row">
        <label class="pro-viz-label">Diagrama:</label>
        <select class="pro-viz-sel" bind:value={resultsStore.diagramType}>
          <option value="none">Ninguno</option>
          <option value="deformed">Deformada</option>
          <option value="momentY">My</option>
          <option value="momentZ">Mz</option>
          <option value="shearY">Vy</option>
          <option value="shearZ">Vz</option>
          <option value="axial">N</option>
          <option value="torsion">T</option>
          <option value="axialColor">Color axil</option>
          <option value="colorMap">Mapa de color</option>
        </select>
      </div>

      {#if resultsStore.diagramType === 'colorMap'}
        <div class="pro-viz-row">
          <label class="pro-viz-label">Variable:</label>
          <select class="pro-viz-sel" bind:value={resultsStore.colorMapKind}>
            <option value="moment">Momento</option>
            <option value="shear">Corte</option>
            <option value="axial">Axil</option>
            <option value="stressRatio">Resistencia (σ/fy)</option>
            <option value="vonMises">Von Mises (σ)</option>
            <option value="shellVonMises">Shell σ Von Mises</option>
          </select>
        </div>
      {/if}

      {#if resultsStore.diagramType === 'deformed'}
        <div class="pro-viz-row">
          <label class="pro-viz-label">Escala:</label>
          <input type="range" class="pro-viz-range" min={1} max={500} bind:value={resultsStore.deformedScale} />
          <span class="pro-viz-val">{resultsStore.deformedScale}×</span>
        </div>
      {/if}

      <div class="pro-viz-row">
        <label class="pro-viz-check">
          <input type="checkbox" bind:checked={resultsStore.showReactions} />
          Reacciones en 3D
        </label>
      </div>
    </div>

    <!-- View mode selector -->
    {#if hasCombinations}
      <div class="pro-view-selector">
        <button class="pro-view-btn" class:active={viewMode === 'single'} onclick={() => switchView('single')}>Caso</button>
        <button class="pro-view-btn" class:active={viewMode === 'combo'} onclick={() => switchView('combo')}>Combo</button>
        <button class="pro-view-btn" class:active={viewMode === 'envelope'} onclick={() => switchView('envelope')}>Envolvente</button>

        {#if viewMode === 'single' && caseKeys.length > 0}
          <select class="pro-view-sel" onchange={onCaseChange}>
            {#each caseKeys as cid}
              {@const lc = modelStore.loadCases.find(c => c.id === cid)}
              <option value={cid}>{lc ? lc.name : `Caso ${cid}`}</option>
            {/each}
          </select>
        {/if}

        {#if viewMode === 'combo' && comboKeys.length > 0}
          <select class="pro-view-sel" onchange={onComboChange}>
            {#each comboKeys as cid}
              {@const cb = modelStore.combinations.find(c => c.id === cid)}
              <option value={cid}>{cb ? cb.name : `Combo ${cid}`}</option>
            {/each}
          </select>
        {/if}
      </div>
    {/if}

    <!-- Results tables — each collapsible -->
    <div class="pro-res-scroll">

      <details class="res-detail" open>
        <summary class="pro-res-section-title">Reacciones <span class="res-count">({results.reactions.length})</span></summary>
        <div class="pro-res-table-wrap">
          <table class="pro-res-table">
            <thead>
              <tr>
                <th>Nodo</th>
                <th>Fx (kN)</th>
                <th>Fy (kN)</th>
                <th>Fz (kN)</th>
                <th>Mx (kN·m)</th>
                <th>My (kN·m)</th>
                <th>Mz (kN·m)</th>
              </tr>
            </thead>
            <tbody>
              {#each results.reactions as r}
                <tr>
                  <td class="col-id">{r.nodeId}</td>
                  <td class="col-num">{fmtNum(r.fx)}</td>
                  <td class="col-num">{fmtNum(r.fy)}</td>
                  <td class="col-num">{fmtNum(r.fz)}</td>
                  <td class="col-num">{fmtNum(r.mx)}</td>
                  <td class="col-num">{fmtNum(r.my)}</td>
                  <td class="col-num">{fmtNum(r.mz)}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </details>

      <details class="res-detail" open>
        <summary class="pro-res-section-title">Solicitaciones <span class="res-count">({results.elementForces.length} barras)</span></summary>
        <div class="pro-res-table-wrap">
          <table class="pro-res-table">
            <thead>
              <tr>
                <th>Elem</th>
                <th>Ext.</th>
                <th>N</th>
                <th>Vy</th>
                <th>Vz</th>
                <th>T</th>
                <th>My</th>
                <th>Mz</th>
              </tr>
            </thead>
            <tbody>
              {#each results.elementForces as ef}
                <tr>
                  <td class="col-id" rowspan="2">{ef.elementId}</td>
                  <td class="col-end">i</td>
                  <td class="col-num">{fmtNum(ef.nStart)}</td>
                  <td class="col-num">{fmtNum(ef.vyStart)}</td>
                  <td class="col-num">{fmtNum(ef.vzStart)}</td>
                  <td class="col-num">{fmtNum(ef.mxStart)}</td>
                  <td class="col-num">{fmtNum(ef.myStart)}</td>
                  <td class="col-num">{fmtNum(ef.mzStart)}</td>
                </tr>
                <tr>
                  <td class="col-end">j</td>
                  <td class="col-num">{fmtNum(ef.nEnd)}</td>
                  <td class="col-num">{fmtNum(ef.vyEnd)}</td>
                  <td class="col-num">{fmtNum(ef.vzEnd)}</td>
                  <td class="col-num">{fmtNum(ef.mxEnd)}</td>
                  <td class="col-num">{fmtNum(ef.myEnd)}</td>
                  <td class="col-num">{fmtNum(ef.mzEnd)}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </details>

      <details class="res-detail">
        <summary class="pro-res-section-title">Desplazamientos <span class="res-count">({results.displacements.length} nodos)</span></summary>
        <div class="pro-res-table-wrap">
          <table class="pro-res-table">
            <thead>
              <tr>
                <th>Nodo</th>
                <th>ux (m)</th>
                <th>uy (m)</th>
                <th>uz (m)</th>
                <th>&#x03B8;x</th>
                <th>&#x03B8;y</th>
                <th>&#x03B8;z</th>
              </tr>
            </thead>
            <tbody>
              {#each results.displacements as d}
                <tr>
                  <td class="col-id">{d.nodeId}</td>
                  <td class="col-num">{fmtNum(d.ux)}</td>
                  <td class="col-num">{fmtNum(d.uy)}</td>
                  <td class="col-num">{fmtNum(d.uz)}</td>
                  <td class="col-num">{fmtNum(d.rx)}</td>
                  <td class="col-num">{fmtNum(d.ry)}</td>
                  <td class="col-num">{fmtNum(d.rz)}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </details>

      {#if results.plateStresses?.length || results.quadStresses?.length}
        <details class="res-detail" open>
          <summary class="pro-res-section-title">Tensiones losas/muros <span class="res-count">({(results.plateStresses?.length ?? 0) + (results.quadStresses?.length ?? 0)})</span></summary>
          <div class="pro-res-table-wrap">
            {#if results.plateStresses?.length}
              <table class="pro-res-table">
                <thead><tr>
                  <th>Elem</th><th>&sigma;xx</th><th>&sigma;yy</th><th>&tau;xy</th>
                  <th>mx</th><th>my</th><th>mxy</th><th>Von Mises</th>
                </tr></thead>
                <tbody>
                  {#each results.plateStresses as ps}
                    <tr>
                      <td class="col-id">{ps.elementId}</td>
                      <td class="col-num">{fmtNum(ps.sigmaXx)}</td>
                      <td class="col-num">{fmtNum(ps.sigmaYy)}</td>
                      <td class="col-num">{fmtNum(ps.tauXy)}</td>
                      <td class="col-num">{fmtNum(ps.mx)}</td>
                      <td class="col-num">{fmtNum(ps.my)}</td>
                      <td class="col-num">{fmtNum(ps.mxy)}</td>
                      <td class="col-num">{fmtNum(ps.vonMises)}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            {/if}
            {#if results.quadStresses?.length}
              <table class="pro-res-table">
                <thead><tr>
                  <th>Elem</th><th>&sigma;xx</th><th>&sigma;yy</th><th>&tau;xy</th>
                  <th>mx</th><th>my</th><th>mxy</th><th>Von Mises</th>
                </tr></thead>
                <tbody>
                  {#each results.quadStresses as qs}
                    <tr>
                      <td class="col-id">{qs.elementId}</td>
                      <td class="col-num">{fmtNum(qs.sigmaXx)}</td>
                      <td class="col-num">{fmtNum(qs.sigmaYy)}</td>
                      <td class="col-num">{fmtNum(qs.tauXy)}</td>
                      <td class="col-num">{fmtNum(qs.mx)}</td>
                      <td class="col-num">{fmtNum(qs.my)}</td>
                      <td class="col-num">{fmtNum(qs.mxy)}</td>
                      <td class="col-num">{fmtNum(qs.vonMises)}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            {/if}
          </div>
        </details>
      {/if}

      {#if results.constraintForces?.length}
        <details class="res-detail">
          <summary class="pro-res-section-title">Fuerzas en vínculos <span class="res-count">({results.constraintForces.length})</span></summary>
          <div class="pro-res-table-wrap">
            <table class="pro-res-table">
              <thead><tr>
                <th>Nodo</th><th>DOF</th><th>Fuerza (kN / kN·m)</th>
              </tr></thead>
              <tbody>
                {#each results.constraintForces as cf}
                  <tr>
                    <td class="col-id">{cf.nodeId}</td>
                    <td>{cf.dof}</td>
                    <td class="col-num">{fmtNum(cf.force)}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </details>
      {/if}

      {#if results.diagnostics?.length}
        <details class="res-detail">
          <summary class="pro-res-section-title">Diagnósticos <span class="res-count">({results.diagnostics.length})</span></summary>
          <div class="pro-res-table-wrap">
            <table class="pro-res-table">
              <thead><tr>
                <th>Elem</th><th>Métrica</th><th>Valor</th><th>Umbral</th><th>Mensaje</th>
              </tr></thead>
              <tbody>
                {#each results.diagnostics as diag}
                  <tr>
                    <td class="col-id">{diag.elementId}</td>
                    <td>{diag.metric}</td>
                    <td class="col-num">{fmtNum(diag.value)}</td>
                    <td class="col-num">{fmtNum(diag.threshold)}</td>
                    <td style="font-size:0.6rem">{diag.message}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </details>
      {/if}

    </div>
  {:else}
    <div class="pro-empty">
      {#if hasModel}
        Presiona "Calcular" para resolver la estructura
      {:else}
        Defini nodos, elementos, materiales, secciones y apoyos primero
      {/if}
    </div>
  {/if}


</div>

<style>
  .pro-res { display: flex; flex-direction: column; height: 100%; overflow: hidden; }

  .pro-res-header {
    padding: 8px 10px;
    border-bottom: 1px solid #1a3050;
    flex-shrink: 0;
  }

  .pro-res-solve-row {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .pro-solve-btn {
    padding: 6px 20px;
    font-size: 0.8rem;
    font-weight: 600;
    color: #fff;
    background: linear-gradient(135deg, #e94560, #c73e54);
    border: 1px solid #e94560;
    border-radius: 4px;
    cursor: pointer;
  }

  .pro-solve-btn:hover { background: linear-gradient(135deg, #ff5a75, #e94560); }
  .pro-solve-btn:disabled { opacity: 0.4; cursor: not-allowed; }

  .pro-sw-label {
    font-size: 0.65rem;
    color: #888;
    display: flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
  }

  .pro-sw-label input { cursor: pointer; }

  .pro-solve-error {
    margin-top: 6px;
    padding: 4px 8px;
    font-size: 0.7rem;
    color: #ff8a9e;
    background: rgba(233, 69, 96, 0.1);
    border-radius: 3px;
  }

  .pro-res-status {
    display: block;
    margin-top: 6px;
    font-size: 0.72rem;
    color: #4ecdc4;
    font-weight: 500;
  }

  /* Visualization controls */
  .pro-viz-section {
    padding: 6px 10px;
    border-bottom: 1px solid #1a3050;
    display: flex;
    flex-direction: column;
    gap: 5px;
    background: #0d1b33;
    flex-shrink: 0;
  }

  .pro-viz-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .pro-viz-label {
    font-size: 0.62rem;
    font-weight: 600;
    color: #888;
    min-width: 55px;
  }

  .pro-viz-sel {
    padding: 2px 4px;
    font-size: 0.64rem;
    background: #0f2840;
    border: 1px solid #1a3050;
    border-radius: 3px;
    color: #ccc;
    cursor: pointer;
    flex: 1;
  }

  .pro-viz-range {
    flex: 1;
    height: 14px;
    accent-color: #e94560;
  }

  .pro-viz-val {
    font-size: 0.6rem;
    font-family: monospace;
    color: #4ecdc4;
    min-width: 36px;
    text-align: right;
  }

  .pro-viz-check {
    font-size: 0.64rem;
    color: #aaa;
    display: flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
  }

  .pro-viz-check input { cursor: pointer; }

  /* View mode selector */
  .pro-view-selector {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 5px 10px;
    border-bottom: 1px solid #1a3050;
    flex-wrap: wrap;
    flex-shrink: 0;
  }

  .pro-view-btn {
    padding: 3px 10px;
    font-size: 0.64rem;
    font-weight: 600;
    color: #888;
    background: #0f2840;
    border: 1px solid #1a3050;
    border-radius: 3px;
    cursor: pointer;
  }

  .pro-view-btn:hover { color: #ccc; background: #1a3860; }
  .pro-view-btn.active { color: #fff; background: #1a4a7a; border-color: #4ecdc4; }

  .pro-view-sel {
    padding: 3px 6px;
    font-size: 0.64rem;
    background: #0f2840;
    border: 1px solid #1a3050;
    border-radius: 3px;
    color: #ccc;
    cursor: pointer;
    margin-left: 4px;
  }

  /* Scrollable results area */
  .pro-res-scroll {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
  }

  /* Collapsible result sections */
  .res-detail {
    border-bottom: 1px solid #1a3050;
  }

  .res-detail > summary {
    cursor: pointer;
    user-select: none;
    list-style: none;
  }

  .res-detail > summary::-webkit-details-marker { display: none; }

  .res-detail > summary::before {
    content: '▸ ';
    font-size: 0.55rem;
    color: #666;
  }

  .res-detail[open] > summary::before {
    content: '▾ ';
  }

  .pro-res-section-title {
    padding: 5px 10px;
    font-size: 0.62rem;
    font-weight: 600;
    color: #4ecdc4;
    text-transform: uppercase;
    background: #0a1a30;
    border-bottom: 1px solid #1a3050;
  }

  .res-count {
    font-weight: 400;
    color: #666;
    font-size: 0.58rem;
  }

  .pro-res-table-wrap { overflow-x: auto; }

  .pro-res-table { width: 100%; border-collapse: collapse; font-size: 0.68rem; }
  .pro-res-table thead { position: sticky; top: 0; z-index: 1; }
  .pro-res-table th {
    padding: 4px 5px; text-align: left; font-size: 0.56rem; font-weight: 600;
    color: #888; text-transform: uppercase; background: #0a1a30; border-bottom: 1px solid #1a4a7a;
  }
  .pro-res-table td { padding: 3px 5px; border-bottom: 1px solid #0f2030; color: #ccc; }
  .col-id { width: 30px; color: #666; font-family: monospace; text-align: center; }
  .col-num { font-family: monospace; text-align: right; font-size: 0.66rem; }
  .col-end { font-size: 0.6rem; color: #888; font-weight: 600; text-align: center; width: 20px; }

  .pro-empty {
    text-align: center;
    color: #555;
    font-style: italic;
    padding: 40px 10px;
  }

</style>
