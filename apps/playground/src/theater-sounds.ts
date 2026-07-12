/**
 * Sound bridge for the theater visualizer.
 *
 * Audio plays on the editor (where the Run gesture unlocks Web Audio), fed by
 * the theater command log plus companion VFS files the engine and editor
 * exchange. Two flows meet here:
 *
 *   playSound(double[])   engine writes samples to `__caturra_pcm_<id>`,
 *                         the editor reads and plays them.
 *   SoundLoader.read(name) the editor decodes a bundled asset and preloads its
 *                         samples to `__caturra_sound_<name>` before the run,
 *                         so `read` returns real audio to manipulate.
 *
 * Both use one text format, shared with the VM stdlib (theater.java): the
 * sample count, then that many space-separated signed 16-bit ints. See
 * specs/EXECUTION.md.
 */

/** Bundled, same-origin sound assets (COEP-safe to fetch and decode). */
export const BUNDLED_SOUNDS = ['beatbox.wav'] as const;

/** URL of a bundled sound, honoring the app's base path. */
export function soundUrl(name: string): string {
  return `${import.meta.env.BASE_URL}sounds/${name}`;
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
