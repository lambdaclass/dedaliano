<script lang="ts">
  import { modelStore } from '../../lib/store';
  import { t } from '../../lib/i18n';
  import { FAMILY_LIST, PROFILE_FAMILIES } from '../../lib/data/steel-profiles';
  import type { ProfileFamily, SteelProfile } from '../../lib/data/steel-profiles';

  type SectionType = 'rect' | 'circular' | 'steel' | 'custom';

  let secType = $state<SectionType>('rect');
  let selectedFamily = $state<ProfileFamily>('IPE');
  let selectedProfileName = $state<string>('');
  let secName = $state('');
  let secB = $state('');  // m
  let secH = $state('');  // m
  let secD = $state('');  // diameter m
  // Custom properties
  let secA = $state('');
  let secIz = $state('');
  let secIy = $state('');
  let secJ = $state('');

  const sections = $derived([...modelStore.sections.values()]);

  function computeRect(b: number, h: number) {
    return {
      a: b * h,
      iz: (b * h * h * h) / 12,
      iy: (h * b * b * b) / 12,
      j: b * h * (b * b + h * h) / 12,
    };
  }

  function computeCircular(d: number) {
    const r = d / 2;
    return {
      a: Math.PI * r * r,
      iz: (Math.PI * r * r * r * r) / 4,
      iy: (Math.PI * r * r * r * r) / 4,
      j: (Math.PI * r * r * r * r) / 2,
    };
  }

  const familyProfiles = $derived(PROFILE_FAMILIES[selectedFamily] ?? []);

  function shapeForFamily(f: ProfileFamily): 'I' | 'H' | 'U' | 'L' | 'RHS' | 'CHS' {
    if (f === 'IPE' || f === 'IPN') return 'I';
    if (f === 'HEB' || f === 'HEA') return 'H';
    if (f === 'UPN') return 'U';
    if (f === 'L') return 'L';
    if (f === 'RHS') return 'RHS';
    return 'CHS';
  }

  function addSteelProfile() {
    const profile = familyProfiles.find(p => p.name === selectedProfileName);
    if (!profile) return;
    // Convert: profile has mm for h/b, cm² for a, cm⁴ for iy/iz
    // Store uses: m for b/h, m² for a, m⁴ for iz/iy
    modelStore.addSection({
      name: profile.name,
      a: profile.a * 1e-4,           // cm² → m²
      iz: profile.iz * 1e-8,         // cm⁴ → m⁴
      iy: profile.iy * 1e-8,         // cm⁴ → m⁴
      b: profile.b / 1000,           // mm → m
      h: profile.h / 1000,           // mm → m
      shape: shapeForFamily(profile.family),
      tw: profile.tw ? profile.tw / 1000 : undefined,
      tf: profile.tf ? profile.tf / 1000 : undefined,
      t: profile.t ? profile.t / 1000 : undefined,
    });
  }

  function addSection() {
    if (!secName.trim()) return;

    if (secType === 'rect') {
      const b = parseFloat(secB);
      const h = parseFloat(secH);
      if (isNaN(b) || isNaN(h) || b <= 0 || h <= 0) return;
      const props = computeRect(b, h);
      modelStore.addSection({ name: secName.trim(), a: props.a, iz: props.iz, iy: props.iy, j: props.j, b, h, shape: 'rect' });
    } else if (secType === 'circular') {
      const d = parseFloat(secD);
      if (isNaN(d) || d <= 0) return;
      const props = computeCircular(d);
      modelStore.addSection({ name: secName.trim(), a: props.a, iz: props.iz, iy: props.iy, j: props.j, shape: 'CHS' });
    } else {
      const a = parseFloat(secA);
      const iz = parseFloat(secIz);
      const iy = parseFloat(secIy);
      const j = parseFloat(secJ);
      if (isNaN(a) || isNaN(iz)) return;
      modelStore.addSection({
        name: secName.trim(),
        a,
        iz,
        iy: isNaN(iy) ? iz : iy,
        j: isNaN(j) ? 0 : j,
        shape: 'generic',
      });
    }

    secName = ''; secB = ''; secH = ''; secD = ''; secA = ''; secIz = ''; secIy = ''; secJ = '';
  }

  function removeSec(id: number) {
    modelStore.removeSection(id);
  }

  // Mini preview dimensions
  function fmtNum(n: number): string {
    if (n === 0) return '0';
    if (n < 0.001) return n.toExponential(2);
    return n.toPrecision(4);
  }
</script>

<div class="pro-sec">
  <div class="pro-sec-header">
    <span class="pro-sec-count">{t('pro.nSections').replace('{n}', String(sections.length))}</span>
  </div>

  <!-- Add section form -->
  <div class="pro-sec-form">
    <div class="pro-sec-type-row">
      <button class="pro-type-btn" class:active={secType === 'rect'} onclick={() => secType = 'rect'}>{t('pro.rectangular')}</button>
      <button class="pro-type-btn" class:active={secType === 'circular'} onclick={() => secType = 'circular'}>{t('pro.circular')}</button>
      <button class="pro-type-btn" class:active={secType === 'steel'} onclick={() => secType = 'steel'}>{t('pro.steelProfile')}</button>
      <button class="pro-type-btn" class:active={secType === 'custom'} onclick={() => secType = 'custom'}>{t('pro.custom')}</button>
    </div>

    <div class="pro-sec-inputs">
      <input type="text" bind:value={secName} placeholder="Nombre (ej: V 20x40)" class="pro-sec-name" />

      {#if secType === 'rect'}
        <div class="pro-sec-dims">
          <label>b (m): <input type="text" bind:value={secB} placeholder="0.20" /></label>
          <label>h (m): <input type="text" bind:value={secH} placeholder="0.40" /></label>
        </div>
        <!-- Mini preview -->
        {#if secB && secH}
          {@const b = parseFloat(secB)}
          {@const h = parseFloat(secH)}
          {#if !isNaN(b) && !isNaN(h) && b > 0 && h > 0}
            {@const scale = Math.min(60 / Math.max(b, h), 200)}
            <div class="pro-sec-preview">
              <svg width={b * scale + 20} height={h * scale + 20} viewBox="-10 -10 {b * scale + 20} {h * scale + 20}">
                <rect x="0" y="0" width={b * scale} height={h * scale} fill="none" stroke="#4ecdc4" stroke-width="1.5" />
                <text x={b * scale / 2} y={h * scale + 10} text-anchor="middle" fill="#888" font-size="9">{(b * 100).toFixed(0)} cm</text>
                <text x={b * scale + 8} y={h * scale / 2} text-anchor="start" fill="#888" font-size="9" dominant-baseline="middle">{(h * 100).toFixed(0)} cm</text>
              </svg>
            </div>
          {/if}
        {/if}
      {:else if secType === 'circular'}
        <div class="pro-sec-dims">
          <label>Ø (m): <input type="text" bind:value={secD} placeholder="0.40" /></label>
        </div>
        {#if secD}
          {@const d = parseFloat(secD)}
          {#if !isNaN(d) && d > 0}
            {@const r = Math.min(30, d * 100)}
            <div class="pro-sec-preview">
              <svg width={r * 2 + 20} height={r * 2 + 20}>
                <circle cx={r + 10} cy={r + 10} r={r} fill="none" stroke="#4ecdc4" stroke-width="1.5" />
                <text x={r + 10} y={r * 2 + 18} text-anchor="middle" fill="#888" font-size="9">Ø{(d * 100).toFixed(0)} cm</text>
              </svg>
            </div>
          {/if}
        {/if}
      {:else if secType === 'steel'}
        <div class="pro-sec-dims">
          <label>{t('pro.family')}:
            <select class="pro-sel" bind:value={selectedFamily}>
              {#each FAMILY_LIST as f}
                <option value={f}>{f}</option>
              {/each}
            </select>
          </label>
          <label>{t('pro.profile')}:
            <select class="pro-sel pro-sel-wide" bind:value={selectedProfileName}>
              <option value="">{t('pro.chooseProfile')}</option>
              {#each familyProfiles as p}
                <option value={p.name}>{p.name} ({p.h}×{p.b}, {p.weight} kg/m)</option>
              {/each}
            </select>
          </label>
        </div>
        {#if selectedProfileName}
          {@const p = familyProfiles.find(pr => pr.name === selectedProfileName)}
          {#if p}
            <div class="pro-profile-info">
              <span>A = {p.a.toFixed(1)} cm²</span>
              <span>Iy = {p.iy.toFixed(0)} cm⁴</span>
              <span>Iz = {p.iz.toFixed(0)} cm⁴</span>
              <span>{p.weight.toFixed(1)} kg/m</span>
            </div>
          {/if}
        {/if}
      {:else}
        <div class="pro-sec-dims">
          <label>A (m²): <input type="text" bind:value={secA} placeholder="0.08" /></label>
          <label>Iz (m⁴): <input type="text" bind:value={secIz} placeholder="1.067e-3" /></label>
          <label>Iy (m⁴): <input type="text" bind:value={secIy} placeholder="Iy" /></label>
          <label>J (m⁴): <input type="text" bind:value={secJ} placeholder="J" /></label>
        </div>
      {/if}

      {#if secType === 'steel'}
        <button class="pro-btn pro-add-sec-btn" onclick={addSteelProfile} disabled={!selectedProfileName}>{t('pro.addProfile')}</button>
      {:else}
        <button class="pro-btn pro-add-sec-btn" onclick={addSection}>{t('pro.addSection')}</button>
      {/if}
    </div>
  </div>

  <!-- Sections table -->
  <div class="pro-sec-table-wrap">
    <table class="pro-sec-table">
      <thead>
        <tr>
          <th>ID</th>
          <th>{t('pro.thName')}</th>
          <th>A (m²)</th>
          <th>Iz (m⁴)</th>
          <th>Iy (m⁴)</th>
          <th>J (m⁴)</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {#each sections as s}
          <tr>
            <td class="col-id">{s.id}</td>
            <td>{s.name}</td>
            <td class="col-num">{fmtNum(s.a)}</td>
            <td class="col-num">{fmtNum(s.iz)}</td>
            <td class="col-num">{fmtNum(s.iy ?? 0)}</td>
            <td class="col-num">{fmtNum(s.j ?? 0)}</td>
            <td><button class="pro-delete-btn" onclick={() => removeSec(s.id)}>×</button></td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

<style>
  .pro-sec {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .pro-sec-header {
    padding: 8px 10px;
    border-bottom: 1px solid #1a3050;
  }

  .pro-sec-count {
    font-size: 0.82rem;
    color: #4ecdc4;
    font-weight: 600;
  }

  .pro-sec-form {
    padding: 8px 10px;
    border-bottom: 1px solid #1a3050;
  }

  .pro-sec-type-row {
    display: flex;
    gap: 4px;
    margin-bottom: 8px;
  }

  .pro-type-btn {
    padding: 5px 10px;
    font-size: 0.75rem;
    font-weight: 500;
    color: #888;
    background: #0f2840;
    border: 1px solid #1a3050;
    border-radius: 4px;
    cursor: pointer;
  }

  .pro-type-btn:hover { color: #ccc; background: #1a3860; }
  .pro-type-btn.active { color: #fff; background: #1a4a7a; border-color: #4ecdc4; }

  .pro-sec-inputs {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .pro-sec-name {
    width: 100%;
    padding: 5px 8px;
    background: #0f2840;
    border: 1px solid #1a3050;
    border-radius: 4px;
    color: #ddd;
    font-size: 0.78rem;
  }

  .pro-sec-name:focus { border-color: #1a4a7a; outline: none; }

  .pro-sec-dims {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .pro-sec-dims label {
    font-size: 0.75rem;
    color: #888;
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .pro-sec-dims input {
    width: 75px;
    padding: 4px 6px;
    background: #0f2840;
    border: 1px solid #1a3050;
    border-radius: 3px;
    color: #ddd;
    font-size: 0.78rem;
    font-family: monospace;
  }

  .pro-sec-dims input:focus { border-color: #1a4a7a; outline: none; }

  .pro-sec-preview {
    display: flex;
    justify-content: center;
    padding: 6px 0;
  }

  .pro-sel {
    padding: 4px 6px;
    background: #0f2840;
    border: 1px solid #1a3050;
    border-radius: 3px;
    color: #ccc;
    font-size: 0.75rem;
    cursor: pointer;
  }

  .pro-sel-wide { min-width: 180px; }

  .pro-profile-info {
    display: flex;
    gap: 12px;
    font-size: 0.72rem;
    color: #4ecdc4;
    font-family: monospace;
    padding: 6px 0;
  }

  .pro-add-sec-btn {
    align-self: flex-start;
    padding: 5px 14px;
    font-size: 0.75rem;
    color: #ccc;
    background: #0f3460;
    border: 1px solid #1a4a7a;
    border-radius: 4px;
    cursor: pointer;
  }

  .pro-add-sec-btn:hover { background: #1a4a7a; color: #fff; }

  .pro-sec-table-wrap {
    flex: 1;
    overflow: auto;
  }

  .pro-sec-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.78rem;
  }

  .pro-sec-table thead { position: sticky; top: 0; z-index: 1; }

  .pro-sec-table th {
    padding: 6px 8px;
    text-align: left;
    font-size: 0.7rem;
    font-weight: 600;
    color: #888;
    text-transform: uppercase;
    background: #0a1a30;
    border-bottom: 1px solid #1a4a7a;
  }

  .pro-sec-table td {
    padding: 5px 8px;
    border-bottom: 1px solid #0f2030;
    color: #ccc;
  }

  .col-id { width: 34px; color: #666; font-family: monospace; text-align: center; }
  .col-num { font-family: monospace; text-align: right; font-size: 0.75rem; }

  .pro-delete-btn {
    background: none; border: none; color: #555; font-size: 1rem; cursor: pointer; padding: 0;
  }
  .pro-delete-btn:hover { color: #ff6b6b; }
</style>
