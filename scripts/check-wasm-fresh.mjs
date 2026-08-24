/**
 * Is the WASM engine on disk older than the Rust it was built from?
 *
 * `packages/core/src/wasm/generated` is a build artifact, not a checked-in one
 * (it is gitignored), and `pnpm dev` serves whatever happens to be there. So an
 * engine change with no rebuild is INVISIBLE: the playground runs the old one,
 * and the compatibility page — whose whole job is to catch a claim that has
 * gone false — reports failures that the source tree does not have. That is
 * how seven of its ninety-three features came to "fail" against an engine that
 * passes all of them.
 *
 * A timestamp is a coarse test and the right one here: it cannot say the
 * artifact is CORRECT, only that it was built after the sources it is built
 * from, which is the thing a developer forgets.
 */
import { readdirSync, statSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const artifact = join(root, 'packages/core/src/wasm/generated/caturra_bg.wasm');

let built;
try {
  built = statSync(artifact).mtimeMs;
} catch {
  console.error(
    '\nThe WASM engine has not been built yet.\n' +
      '  packages/core/src/wasm/generated/caturra_bg.wasm is missing.\n' +
      '  Run:  pnpm build:wasm\n',
  );
  process.exit(1);
}

/** The newest source file the engine is compiled from. */
const newest = (dir) => {
  let latest = { time: 0, path: '' };
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === 'target' || entry.name === 'node_modules') continue;
    const path = join(dir, entry.name);
    if (entry.isDirectory()) {
      const deeper = newest(path);
      if (deeper.time > latest.time) latest = deeper;
    } else if (entry.name.endsWith('.rs') || entry.name.endsWith('.java')) {
      const time = statSync(path).mtimeMs;
      if (time > latest.time) latest = { time, path };
    }
  }
  return latest;
};

const source = newest(join(root, 'crates'));
if (source.time > built) {
  const stale = Math.round((source.time - built) / 60_000);
  console.error(
    `\nThe WASM engine is ${stale} minute(s) older than the Rust it is built from.\n` +
      `  newest source: ${source.path.slice(root.length + 1)}\n` +
      '  The playground would run the OLD engine, and the compatibility page\n' +
      '  would report failures this source tree does not have.\n' +
      '  Run:  pnpm build:wasm\n',
  );
  process.exit(1);
}
