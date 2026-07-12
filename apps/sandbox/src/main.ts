/**
 * Entry point for the caturra sandbox document. It runs inside a hidden,
 * cross-origin iframe that the editor embeds; its only job is to start
 * the engine host and let it service the editor over a MessagePort. All
 * engine state — and every SharedArrayBuffer — stays within this origin,
 * partitioned away from the editor's cookies and storage.
 */
import { startSandboxHost } from '@caturra/core';

const configured = import.meta.env.VITE_EDITOR_ORIGINS ?? '';
const allowedOrigins = configured
  .split(',')
  .map((origin) => origin.trim())
  .filter((origin) => origin.length > 0);

if (allowedOrigins.length === 0) {
  console.error(
    'caturra sandbox: VITE_EDITOR_ORIGINS is empty; every handshake will be ' +
      'rejected. Set it to the editor origin(s) allowed to drive this sandbox.',
  );
}

startSandboxHost({ allowedOrigins });
