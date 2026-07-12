/**
 * Theater visualizer: replays the `org.code.theater` draw-command log caturra
 * emits onto a 400×400 canvas — the browser-side equivalent of the real
 * javabuilder, which rasterizes the same commands to a GIF server-side with
 * `GraphicsHelper`/`GifWriter`.
 *
 * The bundled Scene records one command per line to `theater.log`; this module
 * parses and draws them, holding a frame at each `pause` for animation.
 */

const CANVAS = 400;
const SAMPLE_RATE = 44100;

/** Milliseconds a single `pause` frame holds, scaled to keep runs short. */
function stepMillis(pauseCount: number): number {
  return Math.min(400, Math.max(30, Math.floor(12_000 / Math.max(pauseCount, 1))));
}

export class TheaterViz {
  #ctx: CanvasRenderingContext2D;
  #run = 0;

  // Persisted drawing state (matches the real Scene's defaults).
  #fill: string | null = 'rgb(0,0,0)';
  #stroke: string | null = 'rgb(0,0,0)';
  #strokeWidth = 1;
  #textColor = 'rgb(0,0,0)';
  #textHeight = 20;
  #fontFamily = 'sans-serif';
  #fontStyle = '';

  // Web Audio for playNote, created lazily on the first note (after the Run
  // gesture, so autoplay policy is satisfied). Notes run through a per-run
  // master gain, so starting a new run silences whatever is still ringing.
  #audio: AudioContext | null = null;
  #master: GainNode | null = null;
  // Decoded bundled assets for playSound(name), and the current run's
  // playSound(double[]) sample buffers keyed by the log's pcm id.
  #assetBuffers = new Map<string, AudioBuffer>();
  #pcm = new Map<number, Float32Array>();
  // Decoded starter images for drawImage(name, …); names with no asset fall
  // back to the placeholder box.
  #images = new Map<string, ImageBitmap>();

  constructor(private readonly canvas: HTMLCanvasElement) {
    const ctx = canvas.getContext('2d');
    if (!ctx) {
      throw new Error('2d canvas context unavailable');
    }
    this.#ctx = ctx;
    canvas.width = CANVAS;
    canvas.height = CANVAS;
    this.reset();
  }

  /** Clear the canvas and reset drawing state to the Scene defaults. */
  reset(): void {
    this.#run += 1;
    this.silenceAudio();
    this.#fill = 'rgb(0,0,0)';
    this.#stroke = 'rgb(0,0,0)';
    this.#strokeWidth = 1;
    this.#textColor = 'rgb(0,0,0)';
    this.#textHeight = 20;
    this.#fontFamily = 'sans-serif';
    this.#fontStyle = '';
    this.#ctx.fillStyle = '#ffffff';
    this.#ctx.fillRect(0, 0, CANVAS, CANVAS);
  }

  /**
   * Replay a command log, animating pauses and playing sound. Cancels any
   * prior run. `pcm` maps each `sound pcm <id>` command to its samples (read
   * from the engine's VFS by the caller).
   */
  async play(log: string, pcm = new Map<number, Float32Array>()): Promise<void> {
    this.reset();
    this.#pcm = pcm;
    const run = this.#run;
    const commands = log.split('\n').filter((line) => line.length > 0);
    const pauseCount = commands.filter((c) => c.startsWith('pause ')).length;
    const hold = stepMillis(pauseCount);
    for (const command of commands) {
      if (this.#run !== run) {
        return; // superseded by a newer run
      }
      if (command.startsWith('pause ')) {
        await new Promise((resolve) => setTimeout(resolve, hold));
      } else {
        this.apply(command, true);
      }
    }
  }

  /** Apply every command instantly (a static final frame, for tests). */
  render(log: string): void {
    this.reset();
    for (const command of log.split('\n')) {
      if (command.length > 0 && !command.startsWith('pause ')) {
        this.apply(command, false);
      }
    }
  }

  private color(tokens: string[]): string | null {
    if (tokens[0] === 'none') {
      return null;
    }
    return `rgb(${tokens[0] ?? '0'},${tokens[1] ?? '0'},${tokens[2] ?? '0'})`;
  }

  private apply(command: string, audible: boolean): void {
    const ctx = this.#ctx;
    // `text "..." x y rot` needs the quoted body kept intact.
    const textMatch = /^text "(.*)" (-?\d+) (-?\d+) (-?[\d.]+)$/.exec(command);
    if (textMatch) {
      this.drawText(
        textMatch[1] ?? '',
        Number(textMatch[2]),
        Number(textMatch[3]),
        Number(textMatch[4]),
      );
      return;
    }
    const [op, ...rest] = command.split(' ');
    const n = (i: number): number => Number(rest[i] ?? 0);
    switch (op) {
      case 'clear':
        ctx.fillStyle = this.color(rest) ?? '#ffffff';
        ctx.fillRect(0, 0, CANVAS, CANVAS);
        break;
      case 'fillColor':
        this.#fill = this.color(rest);
        break;
      case 'strokeColor':
        this.#stroke = this.color(rest);
        break;
      case 'textColor':
        this.#textColor = this.color(rest) ?? 'rgb(0,0,0)';
        break;
      case 'strokeWidth':
        this.#strokeWidth = n(0);
        break;
      case 'textHeight':
        this.#textHeight = n(0);
        break;
      case 'textStyle':
        this.#fontFamily = rest[0] === 'MONO' ? 'monospace' : 'sans-serif';
        this.#fontStyle = rest[1] === 'BOLD' ? 'bold' : rest[1] === 'ITALIC' ? 'italic' : '';
        break;
      case 'rectangle':
        this.fillThenStroke(() => {
          ctx.rect(n(0), n(1), n(2), n(3));
        });
        break;
      case 'ellipse': {
        const [x, y, w, h] = [n(0), n(1), n(2), n(3)];
        this.fillThenStroke(() => {
          ctx.ellipse(x + w / 2, y + h / 2, w / 2, h / 2, 0, 0, Math.PI * 2);
        });
        break;
      }
      case 'line':
        if (this.#stroke) {
          ctx.strokeStyle = this.#stroke;
          ctx.lineWidth = this.#strokeWidth;
          ctx.beginPath();
          ctx.moveTo(n(0), n(1));
          ctx.lineTo(n(2), n(3));
          ctx.stroke();
        }
        break;
      case 'polygon': {
        const [x, y, sides, radius] = [n(0), n(1), n(2), n(3)];
        this.fillThenStroke(() => {
          const theta = (2 * Math.PI) / sides;
          for (let i = 0; i < sides; i++) {
            const px = Math.cos(theta * i) * radius + x;
            const py = Math.sin(theta * i) * radius + y;
            if (i === 0) ctx.moveTo(px, py);
            else ctx.lineTo(px, py);
          }
          ctx.closePath();
        });
        break;
      }
      case 'shape':
        this.drawShape(rest);
        break;
      case 'image':
        this.drawImage(rest);
        break;
      case 'note':
        if (audible) {
          this.playTone(rest);
        }
        break;
      case 'sound':
        if (audible) {
          this.playSample(rest);
        }
        break;
      default:
        break;
    }
  }

  private fillThenStroke(path: () => void): void {
    const ctx = this.#ctx;
    ctx.beginPath();
    path();
    if (this.#fill) {
      ctx.fillStyle = this.#fill;
      ctx.fill();
    }
    if (this.#stroke) {
      ctx.strokeStyle = this.#stroke;
      ctx.lineWidth = this.#strokeWidth;
      ctx.stroke();
    }
  }

  private drawText(text: string, x: number, y: number, rotation: number): void {
    const ctx = this.#ctx;
    ctx.save();
    if (rotation !== 0) {
      ctx.translate(x, y);
      ctx.rotate((rotation * Math.PI) / 180);
      ctx.translate(-x, -y);
    }
    ctx.font = `${this.#fontStyle} ${String(this.#textHeight)}px ${this.#fontFamily}`.trim();
    ctx.fillStyle = this.#textColor;
    ctx.textBaseline = 'alphabetic';
    ctx.textAlign = 'left';
    ctx.fillText(text, x, y);
    ctx.restore();
  }

  private drawShape(rest: string[]): void {
    const close = rest[rest.length - 1] === 'true';
    const points = rest.slice(0, -1).map(Number);
    const ctx = this.#ctx;
    if (close) {
      this.fillThenStroke(() => {
        for (let i = 0; i + 1 < points.length; i += 2) {
          const px = points[i] ?? 0;
          const py = points[i + 1] ?? 0;
          if (i === 0) ctx.moveTo(px, py);
          else ctx.lineTo(px, py);
        }
        ctx.closePath();
      });
    } else if (this.#stroke) {
      ctx.strokeStyle = this.#stroke;
      ctx.lineWidth = this.#strokeWidth;
      ctx.beginPath();
      for (let i = 0; i + 1 < points.length; i += 2) {
        const px = points[i] ?? 0;
        const py = points[i + 1] ?? 0;
        if (i === 0) ctx.moveTo(px, py);
        else ctx.lineTo(px, py);
      }
      ctx.stroke();
    }
  }

  /**
   * `image <name|WxH> x y size [rot]` or `image <name|WxH> x y w h rot`. A named
   * starter asset draws for real; an in-engine `Image` object (logged as its
   * dimensions, with no pixels) still draws the placeholder box.
   */
  private drawImage(rest: string[]): void {
    const key = rest[0] ?? '';
    const bitmap = this.#images.get(key);
    if (!bitmap) {
      this.drawImagePlaceholder(rest);
      return;
    }
    const nums = rest.slice(1).map(Number);
    const [x, y] = [nums[0] ?? 0, nums[1] ?? 0];
    let width: number;
    let height: number;
    let rotation: number;
    if (nums.length >= 5) {
      // x y w h rot
      width = nums[2] ?? bitmap.width;
      height = nums[3] ?? bitmap.height;
      rotation = nums[4] ?? 0;
    } else {
      // x y size [rot] — `size` is the width; the height keeps the aspect ratio.
      width = nums[2] ?? bitmap.width;
      height = bitmap.width === 0 ? width : (width * bitmap.height) / bitmap.width;
      rotation = nums[3] ?? 0;
    }
    const ctx = this.#ctx;
    ctx.save();
    if (rotation !== 0) {
      ctx.translate(x, y);
      ctx.rotate((rotation * Math.PI) / 180);
      ctx.translate(-x, -y);
    }
    ctx.drawImage(bitmap, x, y, width, height);
    ctx.restore();
  }

  /** No asset for this name (or an in-engine Image): a labelled placeholder box. */
  private drawImagePlaceholder(rest: string[]): void {
    const nums = rest.slice(1).map(Number);
    const [x, y, size] = [nums[0] ?? 0, nums[1] ?? 0, nums[2] ?? 40];
    const ctx = this.#ctx;
    ctx.fillStyle = '#dcdce4';
    ctx.strokeStyle = '#9a9aa8';
    ctx.lineWidth = 1;
    ctx.fillRect(x, y, size, size);
    ctx.strokeRect(x, y, size, size);
  }

  /** The shared AudioContext, created on first use; null where Web Audio is absent. */
  private audioContext(): AudioContext | null {
    if (this.#audio) {
      return this.#audio;
    }
    if (typeof AudioContext === 'undefined') {
      return null;
    }
    this.#audio = new AudioContext();
    return this.#audio;
  }

  /**
   * Fetch and decode named assets (`name` → URL), returning their mono samples
   * by name so the caller can preload them into the engine's VFS for
   * `SoundLoader.read`. Decoded buffers are cached and reused to play
   * `sound file <name>`. Whatever the stored format, decodeAudioData resamples
   * to the AudioContext's rate, so the program still sees 44.1 kHz samples.
   */
  async loadAssets(assets: Map<string, string>): Promise<Map<string, Float32Array>> {
    const ctx = this.audioContext();
    const samples = new Map<string, Float32Array>();
    if (!ctx) {
      return samples;
    }
    for (const [name, url] of assets) {
      let buffer = this.#assetBuffers.get(name);
      if (!buffer) {
        try {
          const response = await fetch(url);
          buffer = await ctx.decodeAudioData(await response.arrayBuffer());
          this.#assetBuffers.set(name, buffer);
        } catch {
          continue; // asset unavailable — SoundLoader.read falls back to silence
        }
      }
      samples.set(name, buffer.getChannelData(0));
    }
    return samples;
  }

  /** Fetch and decode named images (`name` → URL) for `drawImage(name, …)`. */
  async loadImages(assets: Map<string, string>): Promise<void> {
    for (const [name, url] of assets) {
      if (this.#images.has(name)) {
        continue;
      }
      try {
        const response = await fetch(url);
        this.#images.set(name, await createImageBitmap(await response.blob()));
      } catch {
        // Unavailable — drawImage falls back to the placeholder box.
      }
    }
  }

  /** Play a `sound file <name>` (bundled asset) or `sound pcm <id> <len>` command. */
  private playSample(rest: string[]): void {
    const ctx = this.audioContext();
    if (!ctx) {
      return;
    }
    if (ctx.state === 'suspended') {
      void ctx.resume();
    }
    let buffer: AudioBuffer | undefined;
    if (rest[0] === 'file') {
      buffer = this.#assetBuffers.get(rest[1] ?? '');
    } else if (rest[0] === 'pcm') {
      const samples = this.#pcm.get(Number(rest[1]));
      if (samples && samples.length > 0) {
        buffer = ctx.createBuffer(1, samples.length, SAMPLE_RATE);
        // Copy into a fresh ArrayBuffer-backed view (createBuffer's channel type).
        buffer.copyToChannel(new Float32Array(samples), 0);
      }
    }
    if (!buffer) {
      return;
    }
    const source = ctx.createBufferSource();
    source.buffer = buffer;
    source.connect(this.masterGain(ctx));
    source.start(ctx.currentTime);
  }

  /** The current run's master gain, created lazily and rebuilt each run. */
  private masterGain(ctx: AudioContext): GainNode {
    if (!this.#master) {
      const gain = ctx.createGain();
      gain.gain.value = 1;
      gain.connect(ctx.destination);
      this.#master = gain;
    }
    return this.#master;
  }

  /** Fade out notes still ringing from the previous run; next run gets a fresh master. */
  private silenceAudio(): void {
    if (this.#master && this.#audio) {
      const now = this.#audio.currentTime;
      this.#master.gain.cancelScheduledValues(now);
      this.#master.gain.setTargetAtTime(0, now, 0.02);
    }
    this.#master = null;
  }

  /**
   * Synthesize a `note <INSTRUMENT> <midi> <seconds>` command with a single
   * oscillator and a percussive envelope. MIDI note 69 (A4) is 440 Hz;
   * PIANO is a triangle wave, BASS a sawtooth. The audible length is clamped
   * because the visual timeline is compressed — a long nominal note shouldn't
   * ring over the rest of the scene.
   */
  private playTone(rest: string[]): void {
    const ctx = this.audioContext();
    if (!ctx) {
      return;
    }
    if (ctx.state === 'suspended') {
      void ctx.resume();
    }
    const midi = Number(rest[1] ?? 60);
    const seconds = Number(rest[2] ?? 0.5);
    if (!Number.isFinite(midi)) {
      return;
    }
    const bass = rest[0] === 'BASS';
    const frequency = 440 * 2 ** ((midi - 69) / 12);
    const duration = Math.min(Math.max(Number.isFinite(seconds) ? seconds : 0.5, 0.12), 1.5);

    const osc = ctx.createOscillator();
    osc.type = bass ? 'sawtooth' : 'triangle';
    osc.frequency.value = frequency;

    const gain = ctx.createGain();
    const t0 = ctx.currentTime;
    const peak = bass ? 0.3 : 0.22;
    gain.gain.setValueAtTime(0.0001, t0);
    gain.gain.exponentialRampToValueAtTime(peak, t0 + 0.01);
    gain.gain.exponentialRampToValueAtTime(0.0001, t0 + duration);

    osc.connect(gain);
    gain.connect(this.masterGain(ctx));
    osc.start(t0);
    osc.stop(t0 + duration + 0.02);
  }
}
