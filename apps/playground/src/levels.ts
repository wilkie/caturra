import {
  CSA_UNITS,
  type CsaLevelData,
  type CsaLevelFile,
  type CsaLevelMeta,
  type CsaUnitMeta,
} from './csa-index.js';

// Per-unit level content is code-split: each csa-units/unit-N.ts becomes its
// own chunk, fetched the first time a level in that unit is opened. Keeps the
// initial bundle to the light index (unit/level names) instead of ~3.6 MB of
// start sources and validators.
const unitLoaders = import.meta.glob<{ LEVELS: CsaLevelData[] }>('./csa-units/unit-*.ts');

// Complete solutions (Solve button) are dev-only and git-ignored; the hosted
// build has no such file, so this resolves to `{}` and no solutions ship.
const solutionModules = import.meta.glob<{
  SOLUTIONS?: Record<string, CsaLevelFile[]>;
}>('./csa-solutions.local.ts', { eager: true });
const SOLUTIONS = Object.values(solutionModules)[0]?.SOLUTIONS;

export interface CsaLevel extends CsaLevelData {
  solutionFiles: CsaLevelFile[];
}

/** Load a level's full content (start sources, validators, data, and — in a dev
 *  checkout — its solution), fetching the unit's chunk on demand. */
export async function loadLevel(
  unitIndex: number,
  levelIndex: number,
): Promise<CsaLevel | undefined> {
  const loader = unitLoaders[`./csa-units/unit-${String(unitIndex)}.ts`];
  if (!loader) {
    return undefined;
  }
  const data = (await loader()).LEVELS[levelIndex];
  if (!data) {
    return undefined;
  }
  return {
    ...data,
    solutionFiles: SOLUTIONS?.[`${String(unitIndex)}:${String(levelIndex)}`] ?? [],
  };
}

/** Whether a level has a solution available (dev overlay only), known
 *  synchronously from the eager solutions map — no chunk load needed. */
export function levelHasSolution(unitIndex: number, levelIndex: number): boolean {
  return !!SOLUTIONS?.[`${String(unitIndex)}:${String(levelIndex)}`];
}

export { CSA_UNITS };
export type { CsaLevelData, CsaLevelFile, CsaLevelMeta, CsaUnitMeta };
