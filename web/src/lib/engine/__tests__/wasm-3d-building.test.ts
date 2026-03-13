import { describe, expect, it, beforeAll } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';

import { createMockAPI } from './example-api-mock';
import { load3DExample } from '../../../lib/store/model-examples-3d';
import { buildSolverInput3D } from '../solver-service';
import { mapToObj } from '../wasm-solver';

let wasmSolve3d: ((json: string) => string) | null = null;

describe('WASM 3D building example', () => {
  beforeAll(async () => {
    const wasm = await import('../../wasm/dedaliano_engine.js');
    const wasmPath = path.resolve(__dirname, '../../wasm/dedaliano_engine_bg.wasm');
    const wasmBytes = fs.readFileSync(wasmPath);
    wasm.initSync({ module: wasmBytes });
    wasmSolve3d = wasm.solve_3d;
  });

  it('solves the full 3d-building example without a WASM trap', () => {
    const api = createMockAPI();
    const found = load3DExample('3d-building', api);
    expect(found).toBe(true);

    const modelData = api.getModelData();
    const input = buildSolverInput3D(modelData);
    expect(input).not.toBeNull();

    const directJson = JSON.stringify({
      nodes: mapToObj(input!.nodes),
      materials: mapToObj(input!.materials),
      sections: mapToObj(input!.sections),
      elements: mapToObj(input!.elements),
      supports: mapToObj(input!.supports),
      loads: input!.loads,
      leftHand: (input! as any).leftHand,
    });
    const direct = JSON.parse(wasmSolve3d!(directJson));
    expect(direct.displacements.length).toBeGreaterThan(0);
  });
});
