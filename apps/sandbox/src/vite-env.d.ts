/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** Comma-separated editor origins permitted to drive this sandbox. */
  readonly VITE_EDITOR_ORIGINS?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
