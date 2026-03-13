import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../wasm-solver', async () => {
  const actual = await vi.importActual<typeof import('../wasm-solver')>('../wasm-solver');
  return {
    ...actual,
    solve3D: vi.fn(() => {
      throw new Error('unreachable');
    }),
  };
});

vi.mock('../solver-3d', async () => {
  const actual = await vi.importActual<typeof import('../solver-3d')>('../solver-3d');
  return {
    ...actual,
    solve3D: vi.fn(() => ({
      displacements: [{ nodeId: 1, ux: 0, uy: 0, uz: 0, rx: 0, ry: 0, rz: 0 }],
      reactions: [],
      elementForces: [],
      diagnostics: [],
      solverDiagnostics: [],
    })),
  };
});

import { validateAndSolve3D } from '../solver-service';

describe('validateAndSolve3D fallback', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('falls back to the JS 3D solver when WASM throws', () => {
    const model = {
      nodes: new Map([
        [1, { id: 1, x: 0, y: 0, z: 0 }],
        [2, { id: 2, x: 0, y: 3, z: 0 }],
      ]),
      elements: new Map([
        [1, { id: 1, type: 'frame', nodeI: 1, nodeJ: 2, materialId: 1, sectionId: 1 }],
      ]),
      supports: new Map([
        [1, { id: 1, nodeId: 1, type: 'fixed3d' }],
      ]),
      loads: [],
      materials: new Map([
        [1, { id: 1, name: 'Steel', e: 200000, nu: 0.3, rho: 78.5 }],
      ]),
      sections: new Map([
        [1, { id: 1, name: 'Rect', a: 0.01, iy: 0.0001, iz: 0.0001, j: 0.0002 }],
      ]),
    };

    const result = validateAndSolve3D(model);
    expect(typeof result).not.toBe('string');
    expect(result).not.toBeNull();
    expect(result?.displacements).toHaveLength(1);
  });
});
