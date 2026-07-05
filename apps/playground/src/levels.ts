import { CSA_UNITS, type CsaLevel, type CsaLevelFile, type CsaUnit } from './csa-levels.js';

// Teacher-authored validators and solutions are not committed. When a dev has
// generated the git-ignored `csa-overlay.local.ts` from the corpus, merge it in
// so the Test and Solve buttons work; a fresh checkout / CI has no overlay, so
// those buttons degrade gracefully. `import.meta.glob` resolves to `{}` when
// the file is absent, so the import never fails to build.
const overlay = import.meta.glob<{
  VALIDATORS?: Record<string, CsaLevelFile[]>;
  SOLUTIONS?: Record<string, CsaLevelFile[]>;
}>('./csa-overlay.local.ts', { eager: true });
const local = Object.values(overlay)[0];
if (local) {
  CSA_UNITS.forEach((unit, unitIndex) => {
    unit.levels.forEach((level, levelIndex) => {
      const key = `${String(unitIndex)}:${String(levelIndex)}`;
      const validators = local.VALIDATORS?.[key];
      if (validators) {
        level.validationFiles = validators;
      }
      const solution = local.SOLUTIONS?.[key];
      if (solution) {
        level.solutionFiles = solution;
      }
    });
  });
}

export { CSA_UNITS };
export type { CsaLevel, CsaLevelFile, CsaUnit };
