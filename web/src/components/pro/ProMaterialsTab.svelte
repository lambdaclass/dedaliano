<script lang="ts">
  import { modelStore } from '../../lib/store';
  import { t } from '../../lib/i18n';

  const presets = [
    { name: 'Hormigón H-21', e: 25740, nu: 0.2, rho: 24 },
    { name: 'Hormigón H-25', e: 27806, nu: 0.2, rho: 24 },
    { name: 'Hormigón H-30', e: 30000, nu: 0.2, rho: 24 },
    { name: 'Acero ADN 420', e: 200000, nu: 0.3, rho: 78.5 },
  ];

  let newName = $state('');
  let newE = $state('');
  let newNu = $state('');
  let newRho = $state('');

  const materials = $derived([...modelStore.materials.values()]);

  function addPreset(p: typeof presets[0]) {
    modelStore.addMaterial({ name: p.name, e: p.e, nu: p.nu, rho: p.rho });
  }

  function addCustom() {
    const e = parseFloat(newE);
    const nu = parseFloat(newNu);
    const rho = parseFloat(newRho);
    if (!newName.trim() || isNaN(e) || isNaN(nu) || isNaN(rho)) return;
    modelStore.addMaterial({ name: newName.trim(), e, nu, rho });
    newName = ''; newE = ''; newNu = ''; newRho = '';
  }

  function removeMat(id: number) {
    modelStore.removeMaterial(id);
  }
</script>

<div class="pro-mat">
  <div class="pro-mat-header">
    <span class="pro-mat-count">{t('pro.nMaterials').replace('{n}', String(materials.length))}</span>
  </div>

  <div class="pro-mat-presets">
    <span class="pro-label">{t('pro.presets')}</span>
    {#each presets as p}
      <button class="pro-preset-btn" onclick={() => addPreset(p)}>{p.name}</button>
    {/each}
  </div>

  <div class="pro-mat-table-wrap">
    <table class="pro-mat-table">
      <thead>
        <tr>
          <th>ID</th>
          <th>{t('pro.thName')}</th>
          <th>E (MPa)</th>
          <th>ν</th>
          <th>γ (kN/m³)</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {#each materials as m}
          <tr>
            <td class="col-id">{m.id}</td>
            <td>{m.name}</td>
            <td class="col-num">{m.e.toLocaleString()}</td>
            <td class="col-num">{m.nu}</td>
            <td class="col-num">{m.rho}</td>
            <td><button class="pro-delete-btn" onclick={() => removeMat(m.id)}>×</button></td>
          </tr>
        {/each}
        <tr class="add-row">
          <td class="col-id">+</td>
          <td><input type="text" bind:value={newName} placeholder={t('pro.thName')} /></td>
          <td><input type="text" bind:value={newE} placeholder="E" /></td>
          <td><input type="text" bind:value={newNu} placeholder="ν" /></td>
          <td><input type="text" bind:value={newRho} placeholder="γ" /></td>
          <td><button class="pro-add-btn" onclick={addCustom}>+</button></td>
        </tr>
      </tbody>
    </table>
  </div>
</div>

<style>
  .pro-mat {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .pro-mat-header {
    padding: 8px 10px;
    border-bottom: 1px solid #1a3050;
  }

  .pro-mat-count {
    font-size: 0.82rem;
    color: #4ecdc4;
    font-weight: 600;
  }

  .pro-mat-presets {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 10px 12px;
    border-bottom: 1px solid #1a3050;
    align-items: center;
  }

  .pro-label {
    font-size: 0.75rem;
    color: #888;
    margin-right: 4px;
  }

  .pro-preset-btn {
    padding: 5px 10px;
    font-size: 0.72rem;
    color: #aaa;
    background: #0f3460;
    border: 1px solid #1a4a7a;
    border-radius: 4px;
    cursor: pointer;
  }

  .pro-preset-btn:hover {
    background: #1a4a7a;
    color: #fff;
  }

  .pro-mat-table-wrap {
    flex: 1;
    overflow: auto;
  }

  .pro-mat-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.78rem;
  }

  .pro-mat-table thead {
    position: sticky;
    top: 0;
    z-index: 1;
  }

  .pro-mat-table th {
    padding: 6px 8px;
    text-align: left;
    font-size: 0.7rem;
    font-weight: 600;
    color: #888;
    text-transform: uppercase;
    background: #0a1a30;
    border-bottom: 1px solid #1a4a7a;
  }

  .pro-mat-table td {
    padding: 5px 8px;
    border-bottom: 1px solid #0f2030;
    color: #ccc;
  }

  .col-id {
    width: 30px;
    color: #666;
    font-family: monospace;
    text-align: center;
  }

  .col-num {
    font-family: monospace;
    text-align: right;
  }

  .add-row td {
    background: rgba(78, 205, 196, 0.03);
  }

  .add-row input {
    width: 100%;
    padding: 4px 6px;
    background: #0f2840;
    border: 1px solid #1a3050;
    border-radius: 3px;
    color: #ddd;
    font-size: 0.75rem;
  }

  .add-row input:focus {
    border-color: #1a4a7a;
    outline: none;
  }

  .pro-delete-btn {
    background: none;
    border: none;
    color: #555;
    font-size: 1rem;
    cursor: pointer;
    padding: 0;
  }

  .pro-delete-btn:hover { color: #ff6b6b; }

  .pro-add-btn {
    background: #0f3460;
    border: 1px solid #1a4a7a;
    color: #4ecdc4;
    font-size: 0.85rem;
    border-radius: 3px;
    cursor: pointer;
    padding: 1px 6px;
  }

  .pro-add-btn:hover { background: #1a4a7a; }
</style>
