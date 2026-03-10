/**
 * Predefined exercises for Educational mode.
 * Each exercise defines a structure (nodes, elements, supports, loads)
 * that the solver resolves internally. The student must find the answers.
 */

import type { SupportType } from '../../lib/store/ui.svelte';
import { t } from '../../lib/i18n';

export interface EduExerciseAPI {
  addNode: (x: number, y: number) => number;
  addElement: (nI: number, nJ: number) => number;
  addSupport: (nodeId: number, type: SupportType) => void;
  addNodalLoad: (nodeId: number, fx: number, fy: number, mz?: number) => void;
  addDistributedLoad: (elementId: number, qI: number, qJ?: number) => void;
}

export interface EduExercise {
  id: string;
  title: string;
  description: string;
  difficulty: 'easy' | 'medium' | 'hard';
  build: (api: EduExerciseAPI) => void;
  supports: Array<{
    label: string;
    nodeIndex: number;
    dofs: ('Rx' | 'Ry' | 'M')[];
  }>;
  characteristics: Array<{
    label: string;
    unit: string;
  }>;
}

export function getExercises(): EduExercise[] {
  return [
  {
    id: 'simply-supported-point',
    title: t('edu.ex1Title'),
    description: t('edu.ex1Desc'),
    difficulty: 'easy',
    build(api) {
      const n1 = api.addNode(0, 0);
      const n2 = api.addNode(3, 0);
      const n3 = api.addNode(6, 0);
      api.addElement(n1, n2);
      api.addElement(n2, n3);
      api.addSupport(n1, 'pinned');
      api.addSupport(n3, 'rollerX');
      api.addNodalLoad(n2, 0, -10);
    },
    supports: [
      { label: t('edu.ex1SupportA'), nodeIndex: 0, dofs: ['Rx', 'Ry'] },
      { label: t('edu.ex1SupportB'), nodeIndex: 2, dofs: ['Ry'] },
    ],
    characteristics: [
      { label: 'Mmax', unit: 'kN·m' },
      { label: 'Vmax', unit: 'kN' },
    ],
  },
  {
    id: 'simply-supported-distributed',
    title: t('edu.ex2Title'),
    description: t('edu.ex2Desc'),
    difficulty: 'easy',
    build(api) {
      const n1 = api.addNode(0, 0);
      const n2 = api.addNode(8, 0);
      const e1 = api.addElement(n1, n2);
      api.addSupport(n1, 'pinned');
      api.addSupport(n2, 'rollerX');
      api.addDistributedLoad(e1, -5);
    },
    supports: [
      { label: t('edu.ex2SupportA'), nodeIndex: 0, dofs: ['Rx', 'Ry'] },
      { label: t('edu.ex2SupportB'), nodeIndex: 1, dofs: ['Ry'] },
    ],
    characteristics: [
      { label: 'Mmax', unit: 'kN·m' },
      { label: 'Vmax', unit: 'kN' },
    ],
  },
  {
    id: 'portal-frame',
    title: t('edu.ex3Title'),
    description: t('edu.ex3Desc'),
    difficulty: 'medium',
    build(api) {
      const n1 = api.addNode(0, 0);
      const n2 = api.addNode(0, 3);
      const n3 = api.addNode(4, 3);
      const n4 = api.addNode(4, 0);
      api.addElement(n1, n2);
      api.addElement(n2, n3);
      api.addElement(n3, n4);
      api.addSupport(n1, 'fixed');
      api.addSupport(n4, 'fixed');
      api.addNodalLoad(n2, 8, 0);
    },
    supports: [
      { label: t('edu.ex3SupportA'), nodeIndex: 0, dofs: ['Rx', 'Ry', 'M'] },
      { label: t('edu.ex3SupportB'), nodeIndex: 3, dofs: ['Rx', 'Ry', 'M'] },
    ],
    characteristics: [
      { label: t('edu.ex3MmaxCol'), unit: 'kN·m' },
      { label: t('edu.ex3MmaxBeam'), unit: 'kN·m' },
    ],
  },
];
}
