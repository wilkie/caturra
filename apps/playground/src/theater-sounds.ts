/**
 * Sound bridge for the theater visualizer.
 *
 * Audio plays on the editor (where the Run gesture unlocks Web Audio), fed by
 * the theater command log plus companion VFS files the engine and editor
 * exchange. Two flows meet here:
 *
 *   playSound(double[])    engine writes samples to `__caturra_pcm_<id>`,
 *                          the editor reads and plays them.
 *   SoundLoader.read(name) the editor decodes a named asset and preloads its
 *                          samples to `__caturra_sound_<name>` before the run,
 *                          so `read` returns real audio to manipulate.
 *
 * Both use one text format, shared with the VM stdlib (theater.java): the
 * sample count, then that many space-separated signed 16-bit ints.
 *
 * Named assets resolve through a manifest. `beatbox.wav` ships with the app;
 * the CSA levels' sounds (birds.wav, retrobeat.wav, …) are downloaded by
 * `pnpm sounds:fetch` (they are large, and Code.org serves them without
 * CORP/CORS so an isolated editor cannot fetch them cross-origin). Names with
 * no asset simply stay silent. See specs/EXECUTION.md.
 */

/** Sounds that ship with the app, as paths relative to the base URL. */
const DEFAULT_SOUNDS: Record<string, string> = {
  'beatbox.wav': 'sounds/beatbox.wav',
};

/** Manifest written by `pnpm sounds:fetch` (absent until it is run). */
const FETCHED_MANIFEST = 'sounds/level/manifest.json';

let manifest: Record<string, string> | undefined;

/** Absolute URL of a manifest path, honoring the app's base path. */
export function soundUrl(path: string): string {
  return `${import.meta.env.BASE_URL}${path}`;
}

/**
 * Name → asset path for every sound this build can play: the bundled defaults
 * plus anything `pnpm sounds:fetch` downloaded. Fetched once and cached; a
 * missing manifest just leaves the defaults.
 */
export async function soundManifest(): Promise<Record<string, string>> {
  if (manifest) {
    return manifest;
  }
  let fetched: Record<string, string> = {};
  try {
    const response = await fetch(soundUrl(FETCHED_MANIFEST));
    if (response.ok) {
      fetched = (await response.json()) as Record<string, string>;
    }
  } catch {
    // Not downloaded (or the dev server served index.html instead) — defaults only.
  }
  manifest = { ...DEFAULT_SOUNDS, ...fetched };
  return manifest;
}

/** The `*.wav` names a program names in string literals, e.g. `playSound("birds.wav")`. */
export function referencedSounds(source: string): string[] {
  const names = new Set<string>();
  for (const match of source.matchAll(/["']([\w.-]+\.wav)["']/g)) {
    const name = match[1];
    if (name !== undefined) {
      names.add(name);
    }
  }
  return [...names];
}

/** Serialize float samples in [-1, 1] as `count s0 s1 …` of signed 16-bit ints. */
export function encodeSamples(samples: Float32Array): string {
  const parts = new Array<string>(samples.length + 1);
  parts[0] = String(samples.length);
  for (let i = 0; i < samples.length; i++) {
    const clamped = Math.max(-1, Math.min(1, samples[i] ?? 0));
    parts[i + 1] = String(Math.round(clamped * 32767));
  }
  return parts.join(' ');
}

/** Parse the `count s0 s1 …` sample format back into floats in [-1, 1]. */
export function decodeSamples(text: string): Float32Array {
  const trimmed = text.trim();
  if (trimmed === '') {
    return new Float32Array(0);
  }
  const tokens = trimmed.split(/\s+/);
  const count = Number(tokens[0] ?? 0);
  const samples = new Float32Array(count);
  for (let i = 0; i < count; i++) {
    samples[i] = Number(tokens[i + 1] ?? 0) / 32768;
  }
  return samples;
}
