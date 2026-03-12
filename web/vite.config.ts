import { defineConfig, type Plugin } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { existsSync } from 'fs';
import { resolve } from 'path';

/**
 * Stub out the WASM glue module when it hasn't been built yet.
 * This lets CI run `npm run build` without `wasm-pack build` first.
 */
function wasmStubPlugin(): Plugin {
  const wasmGlue = resolve(__dirname, 'src/lib/wasm/dedaliano_engine.js');
  return {
    name: 'wasm-stub',
    resolveId(id) {
      if (id.includes('wasm/dedaliano_engine') && !existsSync(wasmGlue)) {
        return '\0wasm-stub';
      }
    },
    load(id) {
      if (id === '\0wasm-stub') {
        // Stub all exports that wasm-solver.ts imports
        const noop = '() => "{}"';
        return [
          `export default function initWasm() { return Promise.resolve(); }`,
          `export function initSync() {}`,
          ...[
            'solve_2d', 'solve_3d', 'solve_pdelta_2d', 'solve_buckling_2d',
            'solve_modal_2d', 'solve_spectral_2d', 'solve_plastic_2d',
            'solve_moving_loads_2d', 'analyze_kinematics_2d', 'analyze_kinematics_3d',
            'combine_results_2d', 'combine_results_3d', 'compute_envelope_2d',
            'compute_envelope_3d', 'compute_influence_line',
            'compute_section_stress_2d', 'compute_section_stress_3d',
          ].map(fn => `export const ${fn} = ${noop};`),
        ].join('\n');
      }
    },
  };
}

export default defineConfig({
  plugins: [wasmStubPlugin(), svelte()],
  base: process.env.BASE_PATH || '/',
  server: {
    port: 4000,
  },
  worker: {
    format: 'es',
    plugins: () => [wasmStubPlugin()],
  },
  build: {
    target: 'esnext',
  },
  optimizeDeps: {
    exclude: ['dedaliano-engine', 'web-ifc'],
  },
});
