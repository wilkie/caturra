/// <reference types="vite/client" />

interface ImportMetaEnv {
  /**
   * Origin of the cross-origin sandbox host. When set, the engine runs in
   * a hidden iframe on this origin (isolating its state from the editor);
   * when unset, it runs in a same-origin Web Worker. See specs/EXECUTION.md.
   */
  readonly VITE_SANDBOX_ORIGIN?: string;
}
