import { CSA_UNITS, type CsaLevel, type CsaLevelFile, type CsaUnit } from './csa-levels.js';

// Teacher-authored validators are not committed. When a dev has generated the
// git-ignored `csa-validators.local.ts` overlay from the corpus, merge it in so
// the Test button works; a fresh checkout / CI has no overlay and the Test
// button reports "no tests". `import.meta.glob` resolves to `{}` when the file
// is absent, so the import never fails to build.
const overlay = import.meta.glob<{ VALIDATORS: Record<string, CsaLevelFile[]> }>(
  './csa-validators.local.ts',
  { eager: true },
);
const validators = Object.values(overlay)[0]?.VALIDATORS;
if (validators) {
  CSA_UNITS.forEach((unit, unitIndex) => {
    unit.levels.forEach((level, levelIndex) => {
      const files = validators[`${String(unitIndex)}:${String(levelIndex)}`];
      if (files) {
        level.validationFiles = files;
      }
    });
  });
}

export { CSA_UNITS };
export type { CsaLevel, CsaLevelFile, CsaUnit };
