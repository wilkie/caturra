import js from '@eslint/js';
import tseslint from 'typescript-eslint';
import prettier from 'eslint-config-prettier';

export default tseslint.config(
  {
    ignores: [
      'target/',
      '**/dist/',
      '**/coverage/',
      'packages/core/src/wasm/generated/',
      'e2e/test-results/',
      'e2e/playwright-report/',
    ],
  },
  js.configs.recommended,
  tseslint.configs.strictTypeChecked,
  tseslint.configs.stylisticTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
  {
    // Config files at the root aren't part of a tsconfig project.
    files: ['*.mjs', '*.js', '**/*.config.ts', 'playwright.config.ts'],
    extends: [tseslint.configs.disableTypeChecked],
  },
  prettier,
);
