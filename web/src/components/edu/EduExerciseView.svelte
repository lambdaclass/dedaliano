<script lang="ts">
  import { resultsStore } from '../../lib/store';
  import type { EduExercise } from './exercises';
  import { t } from '../../lib/i18n';

  interface Props {
    exercise: EduExercise;
  }

  let { exercise }: Props = $props();

  // Student answers for reactions
  type ReactionAnswer = Record<string, string>; // dof → value string
  let reactionAnswers = $state<ReactionAnswer[]>(
    exercise.supports.map(s => {
      const ans: ReactionAnswer = {};
      for (const dof of s.dofs) ans[dof] = '';
      return ans;
    })
  );

  // Student answers for characteristic values
  let charAnswers = $state<string[]>(exercise.characteristics.map(() => ''));

  // Verification state
  type VerifState = 'pending' | 'correct' | 'incorrect';
  let reactionVerif = $state<Array<Record<string, VerifState>>>(
    exercise.supports.map(s => {
      const v: Record<string, VerifState> = {};
      for (const dof of s.dofs) v[dof] = 'pending';
      return v;
    })
  );
  let charVerif = $state<VerifState[]>(exercise.characteristics.map(() => 'pending'));
  let hints = $state<string[]>([]);

  const TOLERANCE = 0.05; // 5% tolerance

  function getCorrectReaction(supportIndex: number, dof: string): number | null {
    const results = resultsStore.results;
    if (!results) return null;
    const reactions = results.reactions;
    if (supportIndex >= reactions.length) return null;
    const r = reactions[supportIndex];
    switch (dof) {
      case 'Rx': return r.fx;
      case 'Ry': return r.fy;
      case 'M': return r.mz ?? 0;
      default: return null;
    }
  }

  function verifyReactions() {
    hints = [];
    const newVerif = reactionVerif.map(v => ({ ...v }));

    for (let i = 0; i < exercise.supports.length; i++) {
      const sup = exercise.supports[i];
      for (const dof of sup.dofs) {
        const studentVal = parseFloat(reactionAnswers[i][dof].replace(',', '.'));
        const correct = getCorrectReaction(i, dof);

        if (correct === null || isNaN(studentVal)) {
          newVerif[i][dof] = 'pending';
          continue;
        }

        const absCorrect = Math.abs(correct);
        const tolerance = absCorrect > 0.01 ? absCorrect * TOLERANCE : 0.1;

        if (Math.abs(studentVal - correct) <= tolerance) {
          newVerif[i][dof] = 'correct';
        } else {
          newVerif[i][dof] = 'incorrect';
          // Generate hints
          if (Math.abs(studentVal) - Math.abs(correct) < tolerance && Math.sign(studentVal) !== Math.sign(correct)) {
            hints.push(`${sup.label}, ${dof}: ${t('edu.hintSign')}`);
          } else if (Math.abs(Math.abs(studentVal) - Math.abs(correct)) / (absCorrect || 1) > 0.5) {
            hints.push(`${sup.label}, ${dof}: ${t('edu.hintFarOff')}`);
          } else {
            hints.push(`${sup.label}, ${dof}: ${t('edu.hintClose')}`);
          }
        }
      }
    }

    reactionVerif = newVerif;
  }

  function verifyCharacteristics() {
    const results = resultsStore.results;
    if (!results) return;

    const newVerif = [...charVerif];
    const forces = results.elementForces;

    // Extract max values from element forces
    let maxM = 0, maxV = 0;
    for (const ef of forces) {
      const absM = Math.max(Math.abs(ef.mi), Math.abs(ef.mj));
      const absV = Math.max(Math.abs(ef.vi), Math.abs(ef.vj));
      if (absM > Math.abs(maxM)) maxM = absM;
      if (absV > Math.abs(maxV)) maxV = absV;
    }

    for (let i = 0; i < exercise.characteristics.length; i++) {
      const ch = exercise.characteristics[i];
      const studentVal = parseFloat(charAnswers[i].replace(',', '.'));
      if (isNaN(studentVal)) { newVerif[i] = 'pending'; continue; }

      let correct: number | null = null;
      if (ch.label.startsWith('Mmax')) correct = maxM;
      else if (ch.label.startsWith('Vmax')) correct = maxV;

      if (correct === null) { newVerif[i] = 'pending'; continue; }

      const tolerance = Math.abs(correct) > 0.01 ? Math.abs(correct) * TOLERANCE : 0.1;
      newVerif[i] = Math.abs(Math.abs(studentVal) - Math.abs(correct)) <= tolerance ? 'correct' : 'incorrect';
    }

    charVerif = newVerif;
  }

  function verifClass(state: VerifState): string {
    if (state === 'correct') return 'verif-correct';
    if (state === 'incorrect') return 'verif-incorrect';
    return '';
  }

  const allCorrect = $derived(
    reactionVerif.every(v => Object.values(v).every(s => s === 'correct')) &&
    charVerif.every(s => s === 'correct')
  );
</script>

<div class="exercise-view">
  <div class="exercise-description">
    <p>{exercise.description}</p>
  </div>

  <!-- Step 1: Reactions -->
  <section class="step-section">
    <h3 class="step-title">{t('edu.step1Title')}</h3>

    {#each exercise.supports as sup, i}
      <div class="support-row">
        <span class="support-label">{sup.label}</span>
        <div class="dof-inputs">
          {#each sup.dofs as dof}
            <label class="dof-input {verifClass(reactionVerif[i][dof])}">
              <span class="dof-name">{dof} =</span>
              <input
                type="text"
                inputmode="decimal"
                placeholder="0.00"
                bind:value={reactionAnswers[i][dof]}
                class={verifClass(reactionVerif[i][dof])}
              />
              <span class="dof-unit">{dof === 'M' ? 'kN·m' : 'kN'}</span>
            </label>
          {/each}
        </div>
      </div>
    {/each}

    <button class="verify-btn" onclick={verifyReactions}>{t('edu.verifyReactions')}</button>

    {#if hints.length > 0}
      <div class="hints">
        {#each hints as hint}
          <p class="hint">💡 {hint}</p>
        {/each}
      </div>
    {/if}
  </section>

  <!-- Step 2: Diagrams placeholder -->
  <section class="step-section">
    <h3 class="step-title">{t('edu.step2Title')}</h3>
    <p class="step-info">
      {t('edu.step2Desc')}
      {t('edu.step2Then')}
    </p>
    <div class="diagram-placeholder">
      <div class="diagram-label">N(x)</div>
      <div class="diagram-canvas"></div>
      <div class="diagram-label">V(x)</div>
      <div class="diagram-canvas"></div>
      <div class="diagram-label">M(x)</div>
      <div class="diagram-canvas"></div>
      <p class="diagram-note">{t('edu.diagramsComingSoon')}</p>
    </div>
  </section>

  <!-- Step 3: Characteristic values -->
  <section class="step-section">
    <h3 class="step-title">{t('edu.step3Title')}</h3>

    <div class="char-inputs">
      {#each exercise.characteristics as ch, i}
        <label class="char-input {verifClass(charVerif[i])}">
          <span class="char-name">{ch.label} =</span>
          <input
            type="text"
            inputmode="decimal"
            placeholder="0.00"
            bind:value={charAnswers[i]}
            class={verifClass(charVerif[i])}
          />
          <span class="char-unit">{ch.unit}</span>
        </label>
      {/each}
    </div>

    <button class="verify-btn" onclick={verifyCharacteristics}>{t('edu.verifyValues')}</button>
  </section>

  {#if allCorrect}
    <div class="success-banner">
      {t('edu.exerciseSolved')}
    </div>
  {/if}
</div>

<style>
  .exercise-view {
    padding: 12px 14px;
    overflow-y: auto;
    flex: 1;
  }

  .exercise-description {
    background: #0f2840;
    border: 1px solid #1a4a7a;
    border-radius: 6px;
    padding: 10px 14px;
    margin-bottom: 16px;
  }

  .exercise-description p {
    font-size: 0.78rem;
    color: #bbb;
    margin: 0;
    line-height: 1.5;
  }

  .step-section {
    margin-bottom: 20px;
  }

  .step-title {
    font-size: 0.82rem;
    font-weight: 600;
    color: #4ecdc4;
    margin: 0 0 10px;
    padding-bottom: 4px;
    border-bottom: 1px solid #1a3a5a;
  }

  .step-info {
    font-size: 0.72rem;
    color: #888;
    margin: 0 0 10px;
    line-height: 1.4;
  }

  .support-row {
    margin-bottom: 10px;
  }

  .support-label {
    font-size: 0.72rem;
    font-weight: 600;
    color: #aaa;
    display: block;
    margin-bottom: 4px;
  }

  .dof-inputs {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .dof-input, .char-input {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 0.72rem;
  }

  .dof-name, .char-name {
    color: #aaa;
    font-weight: 500;
    min-width: 28px;
  }

  .dof-input input, .char-input input {
    width: 70px;
    padding: 4px 6px;
    background: #0a1628;
    border: 1px solid #334;
    border-radius: 4px;
    color: #eee;
    font-size: 0.75rem;
    font-family: monospace;
    text-align: right;
  }

  .dof-input input:focus, .char-input input:focus {
    outline: none;
    border-color: #4ecdc4;
  }

  .dof-unit, .char-unit {
    color: #666;
    font-size: 0.65rem;
  }

  .verif-correct input, input.verif-correct {
    border-color: #4caf50 !important;
    background: #0a200a;
  }

  .verif-incorrect input, input.verif-incorrect {
    border-color: #e94560 !important;
    background: #200a0a;
  }

  .verify-btn {
    margin-top: 8px;
    padding: 6px 16px;
    background: #0f3460;
    border: 1px solid #1a4a7a;
    border-radius: 4px;
    color: #4ecdc4;
    font-size: 0.72rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }

  .verify-btn:hover {
    background: #1a4a7a;
  }

  .hints {
    margin-top: 8px;
  }

  .hint {
    font-size: 0.7rem;
    color: #f0a500;
    margin: 2px 0;
    line-height: 1.4;
  }

  .char-inputs {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .diagram-placeholder {
    background: #0a1628;
    border: 1px dashed #334;
    border-radius: 6px;
    padding: 12px;
  }

  .diagram-label {
    font-size: 0.7rem;
    font-weight: 600;
    color: #888;
    margin-bottom: 2px;
  }

  .diagram-canvas {
    height: 40px;
    background: #0f1a30;
    border: 1px solid #223;
    border-radius: 3px;
    margin-bottom: 8px;
  }

  .diagram-note {
    font-size: 0.65rem;
    color: #555;
    text-align: center;
    margin: 4px 0 0;
    font-style: italic;
  }

  .success-banner {
    background: #1a3a2a;
    border: 1px solid #4caf50;
    border-radius: 6px;
    padding: 12px 16px;
    text-align: center;
    font-size: 0.85rem;
    font-weight: 600;
    color: #4caf50;
  }
</style>
