import { CSA_UNITS, type CsaLevel, type CsaLevelFile, type CsaUnit } from './csa-levels.js';
// Teacher-authored JUnit validators are committed (so the hosted Test button
// works). The `.local.ts` fallback lets a dev regenerate them under the old
// name without a rebuild breaking; either resolves to `{}` if absent.
import { VALIDATORS as COMMITTED_VALIDATORS } from './csa-validators.js';

// Complete solutions (Solve button) are NOT committed and never shipped: the
// hosted build runs with `csa-solutions.local.ts` absent, so `import.meta.glob`
// resolves to `{}` and no solutions enter the bundle. A dev checkout that has
// generated the file gets the Solve button locally.
const solutionModules = import.meta.glob<{
  SOLUTIONS?: Record<string, CsaLevelFile[]>;
}>('./csa-solutions.local.ts', { eager: true });
const localSolutions = Object.values(solutionModules)[0]?.SOLUTIONS;

CSA_UNITS.forEach((unit, unitIndex) => {
  unit.levels.forEach((level, levelIndex) => {
    const key = `${String(unitIndex)}:${String(levelIndex)}`;
    const validators = COMMITTED_VALIDATORS[key];
    if (validators) {
      level.validationFiles = validators;
    }
    const solution = localSolutions?.[key];
    if (solution) {
      level.solutionFiles = solution;
    }
  });
});

export { CSA_UNITS };
export type { CsaLevel, CsaLevelFile, CsaUnit };
