import { describe, expect, it } from 'vitest';

describe('store import', () => {
  it('can import the normal store bundle without browser globals', async () => {
    const mod = await import('../index');
    expect(mod.modelStore).toBeTruthy();
    expect(mod.resultsStore).toBeTruthy();
    expect(mod.uiStore).toBeTruthy();
    expect(mod.historyStore).toBeTruthy();
  });
});
