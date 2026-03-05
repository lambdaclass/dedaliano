/**
 * WASM solver wrapper — replaces the pure-TS solver pipeline.
 * Serializes SolverInput (with Maps) → JSON → Rust/WASM → JSON → AnalysisResults.
 */

import initWasm, {
  solve_2d as wasmSolve2d,
  solve_3d as wasmSolve3d,
  solve_pdelta_2d as wasmSolvePdelta2d,
  solve_buckling_2d as wasmSolveBuckling2d,
  solve_modal_2d as wasmSolveModal2d,
  solve_spectral_2d as wasmSolveSpectral2d,
  solve_plastic_2d as wasmSolvePlastic2d,
  solve_moving_loads_2d as wasmSolveMovingLoads2d,
} from '../wasm/dedaliano_engine';

import type { SolverInput, AnalysisResults } from './types';
import type { SolverInput3D, AnalysisResults3D } from './types-3d';

let wasmReady = false;
let wasmInitPromise: Promise<void> | null = null;

/** Initialize the WASM module. Call once at app startup. */
export async function initSolver(): Promise<void> {
  if (wasmReady) return;
  if (wasmInitPromise) return wasmInitPromise;
  wasmInitPromise = (async () => {
    await initWasm();
    wasmReady = true;
  })();
  return wasmInitPromise;
}

/** Check if WASM solver is ready. */
export function isSolverReady(): boolean {
  return wasmReady;
}

// ─── Serialization helpers ──────────────────────────────────────

/** Convert Map<number, T> to { "key": T } for JSON serialization. */
function mapToObj<T>(map: Map<number, T>): Record<string, T> {
  const obj: Record<string, T> = {};
  for (const [k, v] of map) {
    obj[String(k)] = v;
  }
  return obj;
}

/** Serialize SolverInput (with Maps) to JSON string for WASM. */
function serializeInput2D(input: SolverInput): string {
  return JSON.stringify({
    nodes: mapToObj(input.nodes),
    materials: mapToObj(input.materials),
    sections: mapToObj(input.sections),
    elements: mapToObj(input.elements),
    supports: mapToObj(input.supports),
    loads: input.loads,
  });
}

/** Serialize SolverInput3D (with Maps) to JSON string for WASM. */
function serializeInput3D(input: SolverInput3D): string {
  return JSON.stringify({
    nodes: mapToObj(input.nodes),
    materials: mapToObj(input.materials),
    sections: mapToObj(input.sections),
    elements: mapToObj(input.elements),
    supports: mapToObj(input.supports),
    loads: input.loads,
    leftHand: (input as any).leftHand,
  });
}

// ─── Solver functions ───────────────────────────────────────────

/** Solve 2D linear static analysis via WASM. */
export function solve(input: SolverInput): AnalysisResults {
  if (!wasmReady) throw new Error('WASM solver not initialized. Call initSolver() first.');
  const json = serializeInput2D(input);
  const resultJson = wasmSolve2d(json);
  return JSON.parse(resultJson);
}

/** Solve 3D linear static analysis via WASM. */
export function solve3D(input: SolverInput3D): AnalysisResults3D {
  if (!wasmReady) throw new Error('WASM solver not initialized. Call initSolver() first.');
  const json = serializeInput3D(input);
  const resultJson = wasmSolve3d(json);
  return JSON.parse(resultJson);
}

/** Solve 2D P-Delta analysis via WASM. */
export function solvePDelta(input: SolverInput, maxIter = 20, tolerance = 1e-4) {
  if (!wasmReady) throw new Error('WASM solver not initialized.');
  const json = serializeInput2D(input);
  const resultJson = wasmSolvePdelta2d(json, maxIter, tolerance);
  return JSON.parse(resultJson);
}

/** Solve 2D buckling analysis via WASM. */
export function solveBuckling(input: SolverInput, numModes = 4) {
  if (!wasmReady) throw new Error('WASM solver not initialized.');
  const json = serializeInput2D(input);
  const resultJson = wasmSolveBuckling2d(json, numModes);
  return JSON.parse(resultJson);
}

/** Solve 2D modal analysis via WASM. */
export function solveModal(
  input: SolverInput,
  densities: Map<number, number>,
  numModes = 6,
) {
  if (!wasmReady) throw new Error('WASM solver not initialized.');
  const payload = JSON.stringify({
    solver: {
      nodes: mapToObj(input.nodes),
      materials: mapToObj(input.materials),
      sections: mapToObj(input.sections),
      elements: mapToObj(input.elements),
      supports: mapToObj(input.supports),
      loads: input.loads,
    },
    densities: mapToObj(densities),
  });
  const resultJson = wasmSolveModal2d(payload, numModes);
  return JSON.parse(resultJson);
}

/** Solve 2D spectral analysis via WASM. */
export function solveSpectral(config: {
  solver: SolverInput;
  modes: any[];
  densities: Map<number, number>;
  spectrum: { name: string; points: { period: number; sa: number }[]; inG?: boolean };
  direction: 'X' | 'Y';
  rule?: 'SRSS' | 'CQC';
  xi?: number;
  importanceFactor?: number;
  reductionFactor?: number;
}) {
  if (!wasmReady) throw new Error('WASM solver not initialized.');
  const payload = JSON.stringify({
    solver: {
      nodes: mapToObj(config.solver.nodes),
      materials: mapToObj(config.solver.materials),
      sections: mapToObj(config.solver.sections),
      elements: mapToObj(config.solver.elements),
      supports: mapToObj(config.solver.supports),
      loads: config.solver.loads,
    },
    modes: config.modes,
    densities: mapToObj(config.densities),
    spectrum: config.spectrum,
    direction: config.direction,
    rule: config.rule,
    xi: config.xi,
    importanceFactor: config.importanceFactor,
    reductionFactor: config.reductionFactor,
  });
  const resultJson = wasmSolveSpectral2d(payload);
  return JSON.parse(resultJson);
}

/** Solve 2D plastic analysis via WASM. */
export function solvePlastic(config: {
  solver: SolverInput;
  sections: Map<number, { a: number; iz: number; materialId: number; b?: number; h?: number }>;
  materials: Map<number, { fy?: number }>;
  maxHinges?: number;
  mpOverrides?: Map<number, number>;
}) {
  if (!wasmReady) throw new Error('WASM solver not initialized.');
  const payload = JSON.stringify({
    solver: {
      nodes: mapToObj(config.solver.nodes),
      materials: mapToObj(config.solver.materials),
      sections: mapToObj(config.solver.sections),
      elements: mapToObj(config.solver.elements),
      supports: mapToObj(config.solver.supports),
      loads: config.solver.loads,
    },
    sections: mapToObj(config.sections),
    materials: mapToObj(config.materials),
    maxHinges: config.maxHinges,
    mpOverrides: config.mpOverrides ? mapToObj(config.mpOverrides) : undefined,
  });
  const resultJson = wasmSolvePlastic2d(payload);
  return JSON.parse(resultJson);
}

/** Solve 2D moving loads analysis via WASM. */
export function solveMovingLoads(config: {
  solver: SolverInput;
  train: { name: string; axles: { offset: number; weight: number }[] };
  step?: number;
  pathElementIds?: number[];
}) {
  if (!wasmReady) throw new Error('WASM solver not initialized.');
  const payload = JSON.stringify({
    solver: {
      nodes: mapToObj(config.solver.nodes),
      materials: mapToObj(config.solver.materials),
      sections: mapToObj(config.solver.sections),
      elements: mapToObj(config.solver.elements),
      supports: mapToObj(config.solver.supports),
      loads: config.solver.loads,
    },
    train: config.train,
    step: config.step,
    pathElementIds: config.pathElementIds,
  });
  const resultJson = wasmSolveMovingLoads2d(payload);
  return JSON.parse(resultJson);
}
