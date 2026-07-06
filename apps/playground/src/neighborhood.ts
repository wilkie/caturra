/**
 * Neighborhood visualizer: replays the `org.code.neighborhood` animation
 * stream caturra emits (the same `{type,value,detail}` ClientMessages the real
 * javabuilder marshals to its frontend) onto a canvas.
 *
 * The bundled Painter writes one JSON message per line to `neighborhood.jsonl`
 * and the level grid to `grid.txt`; this module parses both and animates the
 * painter over the grid.
 */

interface Tile {
  passable: boolean;
  paintCount: number;
  color: string | null;
}

interface Painter {
  x: number;
  y: number;
  dir: string;
  visible: boolean;
}

interface NeighborhoodMessage {
  value: string;
  detail?: Record<string, string>;
}

/** Unit step per compass direction (grid y grows downward). */
const DIRECTIONS: Record<string, [number, number]> = {
  east: [1, 0],
  west: [-1, 0],
  north: [0, -1],
  south: [0, 1],
};

/** Grid text is rows of space-separated `tileType,paintCount` cells. */
export function parseGrid(text: string): Tile[][] {
  return text
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
    .map((row) =>
      row.split(' ').map((cell) => {
        const [tileType, paint] = cell.split(',');
        const type = Number.parseInt(tileType ?? '1', 10);
        return {
          // 1 OPEN, 2 START, 3 FINISH, 5 START+FINISH are passable.
          passable: type === 1 || type === 2 || type === 3 || type === 5,
          paintCount: paint ? Number.parseInt(paint, 10) : 0,
          color: null,
        };
      }),
    );
}

export function parseMessages(text: string): NeighborhoodMessage[] {
  return text
    .split('\n')
    .filter((line) => line.trim().length > 0)
    .map((line) => JSON.parse(line) as NeighborhoodMessage);
}

/** A snapshot for tests: painted cells and painter positions. */
export interface NeighborhoodState {
  colors: (string | null)[][];
  painters: { id: string; x: number; y: number; dir: string }[];
}

export class NeighborhoodViz {
  #ctx: CanvasRenderingContext2D;
  #grid: Tile[][] = [];
  #painters = new Map<string, Painter>();
  #bucketsVisible = true;
  #timer: ReturnType<typeof setTimeout> | undefined;
  #cell = 40;

  constructor(private readonly canvas: HTMLCanvasElement) {
    const ctx = canvas.getContext('2d');
    if (!ctx) {
      throw new Error('2d canvas context unavailable');
    }
    this.#ctx = ctx;
  }

  /** Load a grid + message stream and reset to the pre-run state. */
  load(gridText: string, messagesText: string): NeighborhoodMessage[] {
    this.stop();
    this.#grid = parseGrid(gridText);
    this.#painters.clear();
    this.#bucketsVisible = true;
    const height = this.#grid.length;
    const width = this.#grid[0]?.length ?? 0;
    this.#cell = Math.max(12, Math.min(48, Math.floor(480 / Math.max(width, height, 1))));
    this.canvas.width = width * this.#cell;
    this.canvas.height = height * this.#cell;
    this.draw();
    return parseMessages(messagesText);
  }

  /** Animate the stream, one message per tick, finishing in ~4s. */
  play(messages: NeighborhoodMessage[]): void {
    this.stop();
    const stepMs = Math.max(20, Math.min(220, Math.floor(4000 / Math.max(messages.length, 1))));
    let i = 0;
    const tick = (): void => {
      const message = messages[i];
      if (message === undefined) {
        this.stop();
        return;
      }
      this.apply(message);
      i += 1;
      this.draw();
      this.#timer = setTimeout(tick, stepMs);
    };
    tick();
  }

  /** Apply every message instantly (for tests / a static final frame). */
  applyAll(messages: NeighborhoodMessage[]): void {
    for (const message of messages) {
      this.apply(message);
    }
    this.draw();
  }

  stop(): void {
    if (this.#timer !== undefined) {
      clearTimeout(this.#timer);
      this.#timer = undefined;
    }
  }

  state(): NeighborhoodState {
    return {
      colors: this.#grid.map((row) => row.map((tile) => tile.color)),
      painters: [...this.#painters].map(([id, p]) => ({ id, x: p.x, y: p.y, dir: p.dir })),
    };
  }

  private apply(message: NeighborhoodMessage): void {
    const detail = message.detail ?? {};
    const id = detail.id ?? '';
    if (message.value === 'INITIALIZE_PAINTER') {
      this.#painters.set(id, {
        x: Number.parseInt(detail.x ?? '0', 10),
        y: Number.parseInt(detail.y ?? '0', 10),
        dir: detail.direction ?? 'east',
        visible: true,
      });
      return;
    }
    if (message.value === 'HIDE_BUCKETS') {
      this.#bucketsVisible = false;
      return;
    }
    if (message.value === 'SHOW_BUCKETS') {
      this.#bucketsVisible = true;
      return;
    }
    const painter = this.#painters.get(id);
    if (!painter) {
      return;
    }
    switch (message.value) {
      case 'MOVE': {
        const [dx, dy] = DIRECTIONS[detail.direction ?? painter.dir] ?? [0, 0];
        painter.x += dx;
        painter.y += dy;
        painter.dir = detail.direction ?? painter.dir;
        break;
      }
      case 'TURN_LEFT':
        painter.dir = detail.direction ?? painter.dir;
        break;
      case 'PAINT': {
        const tile = this.tile(painter);
        if (tile) tile.color = detail.color ?? null;
        break;
      }
      case 'REMOVE_PAINT': {
        const tile = this.tile(painter);
        if (tile) tile.color = null;
        break;
      }
      case 'TAKE_PAINT': {
        const tile = this.tile(painter);
        if (tile) tile.paintCount = Math.max(0, tile.paintCount - 1);
        break;
      }
      case 'HIDE_PAINTER':
        painter.visible = false;
        break;
      case 'SHOW_PAINTER':
        painter.visible = true;
        break;
      default:
        break;
    }
  }

  private tile(painter: Painter): Tile | undefined {
    return this.#grid[painter.y]?.[painter.x];
  }

  private draw(): void {
    const ctx = this.#ctx;
    const cell = this.#cell;
    this.#grid.forEach((row, y) => {
      row.forEach((tile, x) => {
        ctx.fillStyle = !tile.passable ? '#4a4a55' : (tile.color ?? '#f6f6ef');
        ctx.fillRect(x * cell, y * cell, cell, cell);
        ctx.strokeStyle = '#c9c9bd';
        ctx.strokeRect(x * cell, y * cell, cell, cell);
        if (this.#bucketsVisible && tile.paintCount > 0) {
          ctx.fillStyle = '#2b7de0';
          ctx.beginPath();
          ctx.arc(x * cell + cell / 2, y * cell + cell / 2, cell * 0.28, 0, Math.PI * 2);
          ctx.fill();
          ctx.fillStyle = '#fff';
          ctx.font = `${String(Math.floor(cell * 0.4))}px sans-serif`;
          ctx.textAlign = 'center';
          ctx.textBaseline = 'middle';
          ctx.fillText(String(tile.paintCount), x * cell + cell / 2, y * cell + cell / 2);
        }
      });
    });
    for (const painter of this.#painters.values()) {
      if (painter.visible) {
        this.drawPainter(painter);
      }
    }
  }

  private drawPainter(painter: Painter): void {
    const ctx = this.#ctx;
    const cell = this.#cell;
    const cx = painter.x * cell + cell / 2;
    const cy = painter.y * cell + cell / 2;
    const angle: Record<string, number> = { east: 0, south: 90, west: 180, north: 270 };
    ctx.save();
    ctx.translate(cx, cy);
    ctx.rotate(((angle[painter.dir] ?? 0) * Math.PI) / 180);
    ctx.fillStyle = '#e5533d';
    ctx.strokeStyle = '#7a2618';
    ctx.beginPath();
    ctx.moveTo(cell * 0.34, 0);
    ctx.lineTo(-cell * 0.26, -cell * 0.26);
    ctx.lineTo(-cell * 0.26, cell * 0.26);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
    ctx.restore();
  }
}
