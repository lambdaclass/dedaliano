<script lang="ts">
  import { resultsStore, uiStore } from '../../lib/store';
  import { t } from '../../lib/i18n';
  import type { SolverDiagnostic } from '../../lib/engine/types';

  const diagnostics = $derived(resultsStore.diagnostics3D);
  const errors = $derived(diagnostics.filter(d => d.severity === 'error'));
  const warnings = $derived(diagnostics.filter(d => d.severity === 'warning'));
  const infos = $derived(diagnostics.filter(d => d.severity === 'info'));
  const hasAny = $derived(diagnostics.length > 0);

  function severityIcon(s: SolverDiagnostic['severity']): string {
    if (s === 'error') return '\u2717';
    if (s === 'warning') return '\u26A0';
    return '\u2139';
  }

  function severityClass(s: SolverDiagnostic['severity']): string {
    if (s === 'error') return 'sev-error';
    if (s === 'warning') return 'sev-warning';
    return 'sev-info';
  }

  function sourceLabel(s: SolverDiagnostic['source']): string {
    const map: Record<string, string> = {
      solver: 'Solver',
      assembly: 'Ensamblaje',
      kinematic: 'Cinemática',
      verification: 'Verificación',
      serviceability: 'Servicio',
      stability: 'Estabilidad',
    };
    return map[s] ?? s;
  }

  function handleClick(diag: SolverDiagnostic) {
    if (diag.elementIds && diag.elementIds.length > 0) {
      uiStore.selectedElements = new Set(diag.elementIds);
      uiStore.selectedNodes = new Set();
      window.dispatchEvent(new Event('dedaliano-zoom-to-fit'));
    } else if (diag.nodeIds && diag.nodeIds.length > 0) {
      uiStore.selectedNodes = new Set(diag.nodeIds);
      uiStore.selectedElements = new Set();
      window.dispatchEvent(new Event('dedaliano-zoom-to-fit'));
    }
  }

  function formatDetails(details: Record<string, unknown>): string {
    return Object.entries(details)
      .map(([k, v]) => `${k}: ${typeof v === 'number' ? (v as number).toFixed(3) : v}`)
      .join(' | ');
  }
</script>

<div class="diag-panel" data-tour="diagnostics-panel">
  {#if !hasAny}
    <div class="diag-empty">
      <span class="diag-check">&#10003;</span>
      <div>{t('diag.noIssues')}</div>
    </div>
  {:else}
    <!-- Summary bar -->
    <div class="diag-summary">
      {#if errors.length > 0}
        <span class="diag-badge sev-error">{errors.length} {t('diag.errors')}</span>
      {/if}
      {#if warnings.length > 0}
        <span class="diag-badge sev-warning">{warnings.length} {t('diag.warnings')}</span>
      {/if}
      {#if infos.length > 0}
        <span class="diag-badge sev-info">{infos.length} {t('diag.info')}</span>
      {/if}
    </div>

    <!-- Diagnostic list -->
    <div class="diag-list">
      {#each diagnostics as diag}
        <button
          class="diag-item {severityClass(diag.severity)}"
          onclick={() => handleClick(diag)}
          data-tour="diagnostic-item"
        >
          <span class="diag-icon {severityClass(diag.severity)}">{severityIcon(diag.severity)}</span>
          <span class="diag-source">{sourceLabel(diag.source)}</span>
          <span class="diag-msg">{t(diag.message) !== diag.message ? t(diag.message) : diag.message}</span>
          {#if diag.elementIds && diag.elementIds.length > 0}
            <span class="diag-refs">{t('diag.elements')}: {diag.elementIds.join(', ')}</span>
          {/if}
          {#if diag.nodeIds && diag.nodeIds.length > 0}
            <span class="diag-refs">{t('diag.nodes')}: {diag.nodeIds.join(', ')}</span>
          {/if}
          {#if diag.details}
            <span class="diag-details">{formatDetails(diag.details)}</span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .diag-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
  }

  .diag-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 40px 10px;
    color: #4ecdc4;
    font-size: 0.8rem;
  }

  .diag-check {
    font-size: 2rem;
    color: #4ecdc4;
  }

  .diag-summary {
    display: flex;
    gap: 8px;
    padding: 8px 10px;
    border-bottom: 1px solid #1a3050;
    flex-shrink: 0;
  }

  .diag-badge {
    padding: 4px 12px;
    border-radius: 10px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .diag-badge.sev-error { background: rgba(233, 69, 96, 0.2); color: #e94560; }
  .diag-badge.sev-warning { background: rgba(240, 165, 0, 0.2); color: #f0a500; }
  .diag-badge.sev-info { background: rgba(78, 205, 196, 0.2); color: #4ecdc4; }

  .diag-list {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .diag-item {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border: none;
    background: transparent;
    text-align: left;
    cursor: pointer;
    color: #ccc;
    font-size: 0.75rem;
    border-bottom: 1px solid #0f2030;
    width: 100%;
  }

  .diag-item:hover {
    background: rgba(78, 205, 196, 0.05);
  }

  .diag-icon {
    font-size: 0.85rem;
    flex-shrink: 0;
    width: 18px;
    text-align: center;
  }

  .diag-icon.sev-error { color: #e94560; }
  .diag-icon.sev-warning { color: #f0a500; }
  .diag-icon.sev-info { color: #4ecdc4; }

  .diag-source {
    padding: 2px 8px;
    border-radius: 3px;
    font-size: 0.65rem;
    font-weight: 600;
    text-transform: uppercase;
    background: rgba(255, 255, 255, 0.05);
    color: #888;
    flex-shrink: 0;
  }

  .diag-msg {
    flex: 1;
    min-width: 100px;
  }

  .diag-refs {
    font-family: monospace;
    font-size: 0.68rem;
    color: #666;
  }

  .diag-details {
    width: 100%;
    font-family: monospace;
    font-size: 0.65rem;
    color: #555;
    padding-left: 26px;
  }
</style>
