/**
 * Starter-asset bridge for the theater visualizer: the named sounds and images
 * the CSA levels play and draw.
 *
 * Audio plays and images draw on the editor (where the Run gesture unlocks Web
 * Audio), fed by the theater command log plus companion VFS files the engine and
 * editor exchange:
 *
 *   playSound(double[])    engine writes samples to `__caturra_pcm_<id>`,
 *                          the editor reads and plays them.
 *   SoundLoader.read(name) the editor decodes a named asset and preloads its
 *                          samples to `__caturra_sound_<name>` before the run,
 *                          so `read` returns real audio to manipulate.
 *   drawImage(name, ...)   the editor draws the named asset onto the stage.
 *
 * Sample text format (shared with the VM stdlib, theater.java): the sample
 * count, then that many space-separated signed 16-bit ints.
 *
 * Names resolve through a manifest. `beatbox.wav` ships with the app; the levels'
 * assets are downloaded by `pnpm assets:fetch` (they are large, and Code.org
 * serves them without CORP/CORS so an isolated editor cannot fetch them
 * cross-origin). A name that resolves to nothing stays silent / draws a
 * placeholder. See specs/EXECUTION.md.
 */

/** Assets that ship with the app, as paths relative to the base URL. */
const DEFAULT_ASSETS: Record<string, string> = {
  'beatbox.wav': 'sounds/beatbox.wav',
};

/** Manifest written by `pnpm assets:fetch` (absent until it is run). */
const FETCHED_MANIFEST = 'level-assets/manifest.json';

let manifest: Record<string, string> | undefined;

/** Absolute URL of a manifest path, honoring the app's base path. */
export function assetUrl(path: string): string {
  return `${import.meta.env.BASE_URL}${path}`;
}

/**
 * Name → asset path for every starter asset this build can use: the bundled
 * defaults plus anything `pnpm assets:fetch` downloaded. Fetched once and
 * cached; a missing manifest just leaves the defaults.
 */
export async function assetManifest(): Promise<Record<string, string>> {
  if (manifest) {
    return manifest;
  }
  let fetched: Record<string, string> = {};
  try {
    const response = await fetch(assetUrl(FETCHED_MANIFEST));
    if (response.ok) {
      fetched = (await response.json()) as Record<string, string>;
    }
  } catch {
    // Not downloaded (or the dev server served index.html instead) — defaults only.
  }
  manifest = { ...DEFAULT_ASSETS, ...fetched };
  return manifest;
}

/** Whether a name is a sound (rather than an image). */
export function isSound(name: string): boolean {
  return name.toLowerCase().endsWith('.wav');
}

/**
 * The starter assets a program names in string literals — `playSound("birds.wav")`,
 * `drawImage("house.png", …)`. Asset names can contain spaces.
 */
export function referencedAssets(source: string): string[] {
  const names = new Set<string>();
  for (const match of source.matchAll(/["']([\w ()-]+\.(?:wav|png|jpe?g))["']/gi)) {
    const name = match[1];
    if (name !== undefined) {
      names.add(name);
    }
  }
  return [...names];
}

/**
 * The pixel-buffer format the engine and editor exchange for `org.code.media.Image`
 * (shared with the VM's native `System.__imageDims`/`__imagePixels`/`__writeImage`):
 * width and height as little-endian u32, then RGB triples, row-major. Binary
 * rather than text because a 400x400 image is 160k pixels — far too many to
 * ferry through the interpreter a token at a time.
 */
export function encodePixels(image: ImageData): Uint8Array {
  const { width, height, data } = image;
  const bytes = new Uint8Array(8 + width * height * 3);
  new DataView(bytes.buffer).setUint32(0, width, true);
  new DataView(bytes.buffer).setUint32(4, height, true);
  for (let i = 0, o = 8; i < width * height; i++, o += 3) {
    bytes[o] = data[i * 4] ?? 0;
    bytes[o + 1] = data[i * 4 + 1] ?? 0;
    bytes[o + 2] = data[i * 4 + 2] ?? 0;
  }
  return bytes;
}

/** Parse a pixel buffer the engine wrote back out (a drawn, possibly edited Image). */
export function decodePixels(bytes: Uint8Array): ImageData | undefined {
  if (bytes.byteLength < 8) {
    return undefined;
  }
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const width = view.getUint32(0, true);
  const height = view.getUint32(4, true);
  if (width === 0 || height === 0 || bytes.byteLength < 8 + width * height * 3) {
    return undefined;
  }
  const image = new ImageData(width, height);
  for (let i = 0, o = 8; i < width * height; i++, o += 3) {
    image.data[i * 4] = bytes[o] ?? 0;
    image.data[i * 4 + 1] = bytes[o + 1] ?? 0;
    image.data[i * 4 + 2] = bytes[o + 2] ?? 0;
    image.data[i * 4 + 3] = 255;
  }
  return image;
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
