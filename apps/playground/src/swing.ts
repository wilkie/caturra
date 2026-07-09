/**
 * Swing visualizer: renders the `javax.swing` component tree caturra emits
 * into real, accessible HTML — the browser-side equivalent of a live Swing
 * window, but built from semantic elements so keyboard navigation and screen
 * readers work without any extra ARIA scaffolding.
 *
 * The bundled JFrame serializes its tree to `swing.json` on setVisible(true);
 * this module parses that tree and builds native controls: JButton → <button>,
 * JTextField → <input>, JLabel(setLabelFor) → <label for>, JCheckBox → a
 * labelled checkbox, JFrame → a named region landmark. DOM order is tab order,
 * so focus follows the order components were added. Interactivity (listeners)
 * is Phase 2; Phase 1 renders the declarative UI.
 */

/** A component border (BorderFactory), applied as CSS in common(). */
interface BorderSpec {
  type: 'line' | 'empty' | 'titled' | 'etched' | 'matte' | 'bevel' | 'compound';
  thickness: number;
  color?: string; // "r,g,b"
  title?: string;
  insets: string; // "top,left,bottom,right"
  raised?: boolean; // bevel: raised vs lowered
  outer?: BorderSpec; // compound outside
  inner?: BorderSpec; // compound inside, or a titled border's frame
}

/** One tab of a JTabbedPane: a caption and the component it shows. */
interface TabSpec {
  title: string;
  component: SwingNode;
}

/** A child's GridBagConstraints under a GridBagLayout parent. */
interface GbcSpec {
  gridx: number;
  gridy: number;
  gridwidth: number;
  gridheight: number;
  weightx: number;
  weighty: number;
  anchor: number;
  fill: number;
  insets: string; // "top,left,bottom,right"
}

interface SwingNode {
  type:
    | 'frame'
    | 'panel'
    | 'label'
    | 'button'
    | 'toggle'
    | 'textfield'
    | 'textarea'
    | 'checkbox'
    | 'radio'
    | 'combobox'
    | 'list'
    | 'slider'
    | 'scrollpane'
    | 'tabbedpane'
    | 'splitpane'
    | 'toolbar'
    | 'toolbarsep'
    | 'table'
    | 'progressbar'
    | 'spinner'
    | 'strut'
    | 'component';
  id: string;
  enabled?: boolean;
  /** Component.setVisible(false): the component is hidden (display:none). */
  hidden?: boolean;
  /** AccessibleContext name/description → aria-label / aria-description. */
  accName?: string;
  accDesc?: string;
  tooltip?: string;
  bg?: string;
  fg?: string;
  /** Component.setFont: "<style> <size> <family>". */
  font?: string;
  /** Component.setBorder (BorderFactory). */
  border?: BorderSpec;
  text?: string;
  title?: string;
  width?: number;
  height?: number;
  layout?: string;
  /** This child's BorderLayout region ("North".."Center"), when its parent
   * uses a BorderLayout. */
  region?: string;
  /** This child's GridBagConstraints, when its parent uses a GridBagLayout. */
  gbc?: GbcSpec;
  /** setBounds "x,y,w,h", honored when the parent uses a null (absolute) layout. */
  bounds?: string;
  /** setPreferredSize / setMinimumSize / setMaximumSize "w,h" hints. */
  psize?: string;
  minsize?: string;
  maxsize?: string;
  /** setHorizontalAlignment (SwingConstants): 0 center, 2/10 left, 4/11 right. */
  halign?: number;
  columns?: number;
  rows?: number;
  editable?: boolean;
  /** A one-shot `"start,end"` caret move requested by setCaretPosition/select. */
  caretReq?: string;
  /** JPasswordField: render as a masked <input type=password>. */
  password?: boolean;
  wrap?: boolean;
  selected?: boolean;
  /** JButton mnemonic letter (Alt-shortcut), applied as the accessKey. */
  mnemonic?: string;
  /** JList: the selected indices and whether multiple selection is allowed. */
  selectedIndices?: number[];
  multiple?: boolean;
  for?: string;
  /** True when the widget has a listener, so its native event dispatches. */
  listens?: boolean;
  /** A text component has a DocumentListener (fire on every keystroke). */
  docListen?: boolean;
  /** JComboBox option labels. */
  items?: string[];
  /** Per-row colours from a JList's ListCellRenderer (parallel to `items`). */
  itemStyles?: SwingItemStyle[];
  /** JComboBox selected option index. */
  selectedIndex?: number;
  /** JSlider/JSpinner/JProgressBar range and value; JSpinner step. */
  min?: number;
  max?: number;
  value?: number;
  step?: number;
  /** JProgressBar: indeterminate mode and its painted string. */
  indeterminate?: boolean;
  stringPainted?: boolean;
  string?: string;
  /** Box strut/glue: fixed spacer size, or glue that stretches. */
  w?: number;
  h?: number;
  glue?: boolean;
  /** JRadioButton ButtonGroup id (shared DOM `name` for exclusivity). */
  group?: string;
  /** A custom JPanel's recorded Graphics commands (newline-separated). */
  paint?: string;
  /** Canvas dimensions for a painted panel. */
  pw?: number;
  ph?: number;
  /** Panel has a MouseListener (wire click) / MouseMotionListener (wire drag). */
  mouse?: boolean;
  drag?: boolean;
  /** Panel has a KeyListener (make it focusable and wire keydown/keyup). */
  key?: boolean;
  /** A frame's menu bar (setJMenuBar). */
  menubar?: SwingMenuBar;
  /** A pending requestFocus target, "<cid>:<seq>". */
  focus?: string;
  /** A JScrollPane's wrapped component and scrollbar policies. */
  view?: SwingNode | null;
  hpolicy?: number;
  vpolicy?: number;
  /** A JTabbedPane's tabs (title + component) and tab placement (TOP=1..RIGHT=4). */
  tabs?: TabSpec[];
  placement?: number;
  /** A JSplitPane's two components, orientation (HORIZONTAL_SPLIT=1), and
   * divider location in px (<=0 = even split). */
  left?: SwingNode | null;
  right?: SwingNode | null;
  orientation?: number;
  divider?: number;
  /** A JToolBar's accessible name (setName), used as its aria-label. */
  name?: string | null;
  /** JTable column names, row-major cell text, and the selected row (-1 none). */
  headers?: string[];
  cells?: string[][];
  /** Per-cell styling from a column's TableCellRenderer (parallel to `cells`). */
  cellStyles?: SwingCellStyle[][];
  selectedRow?: number;
  children?: SwingNode[];
}

interface SwingMenuBar {
  menus: SwingMenu[];
}
interface SwingMenu {
  type?: 'menu';
  text?: string;
  items?: SwingMenuEntry[];
}
// A menu entry is a leaf item, a separator, or a nested submenu (which is
// itself a SwingMenu with its own items).
/** Per-row colours produced by a JList's ListCellRenderer. */
interface SwingItemStyle {
  fg?: string;
  bg?: string;
}

/** Per-cell styling produced by a JTable column's TableCellRenderer. */
interface SwingCellStyle {
  fg?: string;
  bg?: string;
  halign?: number;
}

interface SwingMenuEntry {
  type: 'menuitem' | 'checkmenuitem' | 'radiomenuitem' | 'separator' | 'menu';
  text?: string;
  id?: string;
  enabled?: boolean;
  listens?: boolean;
  selected?: boolean;
  /** setAccelerator hint/match string, e.g. "Ctrl+S". */
  accel?: string;
  items?: SwingMenuEntry[];
}

type FieldElement = HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement;

/** Collect every menu item's accelerator (and its id) from a menu tree. */
function collectAccelerators(menus: SwingMenu[]): { accel: string; id: string }[] {
  const out: { accel: string; id: string }[] = [];
  const walk = (items: SwingMenuEntry[] | undefined): void => {
    for (const entry of items ?? []) {
      if (entry.type === 'menu') {
        walk(entry.items);
      } else if (entry.accel !== undefined && entry.id !== undefined) {
        out.push({ accel: entry.accel, id: entry.id });
      }
    }
  };
  for (const menu of menus) {
    walk(menu.items);
  }
  return out;
}

/** A keydown as an accelerator string ("Ctrl+Shift+S"), matching KeyStroke's. */
function keyComboString(event: KeyboardEvent): string {
  let s = '';
  if (event.ctrlKey || event.metaKey) {
    s += 'Ctrl+'; // treat Cmd as Ctrl so Mac accelerators match
  }
  if (event.shiftKey) {
    s += 'Shift+';
  }
  if (event.altKey) {
    s += 'Alt+';
  }
  const named: Record<string, string> = {
    Enter: 'Enter',
    ' ': 'Space',
    Escape: 'Esc',
    ArrowLeft: 'Left',
    ArrowUp: 'Up',
    ArrowRight: 'Right',
    ArrowDown: 'Down',
    Backspace: 'Backspace',
    Tab: 'Tab',
    Delete: 'Delete',
  };
  return s + (named[event.key] ?? (event.key.length === 1 ? event.key.toUpperCase() : event.key));
}

/** A text control's `"start,end"` selection, or null for anything else. The
 * editable combo's input is registered under the JComboBox's id, but the engine
 * ignores a caret it can't use, so reporting one is harmless. */
function caretOf(el: FieldElement): string | null {
  const isText =
    (el instanceof HTMLInputElement && (el.type === 'text' || el.type === 'password')) ||
    el instanceof HTMLTextAreaElement;
  if (!isText || el.selectionStart === null || el.selectionEnd === null) {
    return null;
  }
  return `${String(el.selectionStart)},${String(el.selectionEnd)}`;
}

/** Apply a one-shot caret/selection move the program asked for (setCaretPosition
 * / select / selectAll). Absent on renders the program didn't request. */
function applyCaretRequest(el: HTMLInputElement | HTMLTextAreaElement, node: SwingNode): void {
  if (node.caretReq === undefined) {
    return;
  }
  const [start, end] = node.caretReq.split(',').map(Number);
  if (start !== undefined && end !== undefined) {
    el.setSelectionRange(start, end);
  }
}

/** Turn a `"r,g,b"` triple from the engine into a CSS color. */
function cssColor(triple: string | undefined): string | null {
  if (triple === undefined) {
    return null;
  }
  const parts = triple.split(',').map(Number);
  if (parts.length !== 3 || parts.some((n) => Number.isNaN(n))) {
    return null;
  }
  return `rgb(${String(parts[0])}, ${String(parts[1])}, ${String(parts[2])})`;
}

/** The value string for a field element, matching the engine's __setFromHost.
 * Percent-escapes `%` and newlines so a multi-line JTextArea value survives
 * the newline-delimited event payload (the engine decodes it). */
function fieldValue(el: FieldElement): string {
  let raw: string;
  if (el instanceof HTMLSelectElement) {
    // A multi-select JList reports every chosen index; a single select (or
    // JComboBox) reports the one selected index (-1 when none).
    raw = el.multiple
      ? [...el.selectedOptions].map((option) => String(option.index)).join(',')
      : String(el.selectedIndex);
  } else if (el instanceof HTMLInputElement && (el.type === 'checkbox' || el.type === 'radio')) {
    raw = String(el.checked);
  } else {
    raw = el.value; // text field / text area / slider
  }
  return raw.replace(/%/g, '%25').replace(/\n/g, '%0A');
}

/** Keys that scroll the page; suppressed while a keyboard surface is focused. */
const SCROLL_KEYS = new Set([
  'ArrowLeft',
  'ArrowUp',
  'ArrowRight',
  'ArrowDown',
  ' ',
  'PageUp',
  'PageDown',
  'Home',
  'End',
]);

/** DOM key names → AWT virtual-key codes (java.awt.event.KeyEvent). */
const AWT_KEY_CODES: Record<string, number> = {
  ArrowLeft: 37,
  ArrowUp: 38,
  ArrowRight: 39,
  ArrowDown: 40,
  Enter: 10,
  ' ': 32,
  Escape: 27,
  Tab: 9,
  Backspace: 8,
  Delete: 127,
  Shift: 16,
  Control: 17,
  Alt: 18,
};

/** The AWT virtual-key code for a DOM keyboard event (VK_A == 'A' == 65). */
function awtKeyCode(event: KeyboardEvent): number {
  const mapped = AWT_KEY_CODES[event.key];
  if (mapped !== undefined) {
    return mapped;
  }
  if (event.key.length === 1) {
    return event.key.toUpperCase().charCodeAt(0);
  }
  return 0; // VK_UNDEFINED
}

/** The AWT key character for a DOM keyboard event (CHAR_UNDEFINED for action keys). */
function awtKeyChar(event: KeyboardEvent): number {
  if (event.key.length === 1) {
    return event.key.charCodeAt(0);
  }
  const named: Record<string, number> = { Enter: 10, Tab: 9, Backspace: 8, Escape: 27 };
  return named[event.key] ?? 65535; // CHAR_UNDEFINED
}

/** Replay a custom panel's recorded java.awt.Graphics commands onto a canvas. */
function paintCanvas(canvas: HTMLCanvasElement, paint: string): void {
  const ctx = canvas.getContext('2d');
  if (!ctx) {
    return;
  }
  // Wipe the whole canvas, then replay — a full redraw, like paintComponent
  // (the panel is cleared before it paints). The canvas element is retained
  // across ticks, so without this the previous frame would show through.
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  let color = 'rgb(0,0,0)'; // Graphics current color (fills and strokes)
  let font = '14px system-ui, sans-serif'; // Graphics current font (drawString)
  // Stroke state persists on the retained context, so reset it each frame to
  // the Graphics default (a 1px pen); Graphics2D.setStroke overrides it.
  ctx.lineWidth = 1;
  ctx.lineCap = 'butt';
  ctx.lineJoin = 'miter';
  const oval = (x: number, y: number, w: number, h: number): void => {
    ctx.beginPath();
    ctx.ellipse(x + w / 2, y + h / 2, w / 2, h / 2, 0, 0, Math.PI * 2);
  };
  // Trace an AWT arc (degrees, 0 at 3 o'clock, CCW positive) as a canvas
  // ellipse arc. Canvas angles are clockwise (y-down), so negate; the sweep
  // sign follows arcAngle.
  const arcPath = (
    x: number,
    y: number,
    w: number,
    h: number,
    start: number,
    sweep: number,
  ): void => {
    const cx = x + w / 2;
    const cy = y + h / 2;
    const a0 = (-start * Math.PI) / 180;
    const a1 = (-(start + sweep) * Math.PI) / 180;
    ctx.ellipse(cx, cy, w / 2, h / 2, 0, a0, a1, sweep > 0);
  };
  for (const command of paint.split('\n')) {
    if (command === '') {
      continue;
    }
    // `drawString "..." x y` keeps the quoted body intact.
    const text = /^drawString "(.*)" (-?\d+) (-?\d+)$/.exec(command);
    if (text) {
      ctx.fillStyle = color;
      ctx.font = font;
      ctx.textBaseline = 'alphabetic';
      ctx.fillText(text[1] ?? '', Number(text[2]), Number(text[3]));
      continue;
    }
    const [op, ...rest] = command.split(' ');
    const n = (i: number): number => Number(rest[i] ?? 0);
    switch (op) {
      case 'setColor':
        color = `rgb(${String(n(0))},${String(n(1))},${String(n(2))})`;
        break;
      case 'setFont':
        // `setFont <style> <size> <family...>`; family is the rest of the line.
        font = cssFont(n(0), n(1), rest.slice(2).join(' '));
        break;
      case 'setStroke':
        // `setStroke <width> <cap> <join>` (java.awt.BasicStroke). Applies to
        // subsequent stroke operations until changed.
        ctx.lineWidth = Math.max(1, n(0));
        ctx.lineCap = (['butt', 'round', 'square'] as const)[n(1)] ?? 'butt';
        ctx.lineJoin = (['miter', 'round', 'bevel'] as const)[n(2)] ?? 'miter';
        break;
      case 'fillRect':
        ctx.fillStyle = color;
        ctx.fillRect(n(0), n(1), n(2), n(3));
        break;
      case 'drawRect':
        ctx.strokeStyle = color;
        ctx.strokeRect(n(0), n(1), n(2), n(3));
        break;
      case 'fillOval':
        oval(n(0), n(1), n(2), n(3));
        ctx.fillStyle = color;
        ctx.fill();
        break;
      case 'drawOval':
        oval(n(0), n(1), n(2), n(3));
        ctx.strokeStyle = color;
        ctx.stroke();
        break;
      case 'drawLine':
        ctx.strokeStyle = color;
        ctx.beginPath();
        ctx.moveTo(n(0), n(1));
        ctx.lineTo(n(2), n(3));
        ctx.stroke();
        break;
      case 'fillPolygon':
      case 'drawPolygon': {
        if (rest.length < 6) {
          break; // need at least 3 (x,y) points
        }
        ctx.beginPath();
        ctx.moveTo(n(0), n(1));
        for (let i = 2; i + 1 < rest.length; i += 2) {
          ctx.lineTo(n(i), n(i + 1));
        }
        ctx.closePath();
        if (op === 'fillPolygon') {
          ctx.fillStyle = color;
          ctx.fill();
        } else {
          ctx.strokeStyle = color;
          ctx.stroke();
        }
        break;
      }
      case 'fillArc':
        ctx.beginPath();
        ctx.moveTo(n(0) + n(2) / 2, n(1) + n(3) / 2); // center: a pie slice
        arcPath(n(0), n(1), n(2), n(3), n(4), n(5));
        ctx.closePath();
        ctx.fillStyle = color;
        ctx.fill();
        break;
      case 'drawArc':
        ctx.beginPath();
        arcPath(n(0), n(1), n(2), n(3), n(4), n(5));
        ctx.strokeStyle = color;
        ctx.stroke();
        break;
      default:
        break;
    }
  }
}

/** Build a CSS font from an AWT Font (style bit flags, size px, logical family). */
function cssFont(style: number, size: number, family: string): string {
  const families: Record<string, string> = {
    SansSerif: 'system-ui, sans-serif',
    Dialog: 'system-ui, sans-serif',
    DialogInput: 'ui-monospace, monospace',
    Serif: 'Georgia, serif',
    Monospaced: 'ui-monospace, monospace',
  };
  const css = families[family] ?? `${family}, system-ui, sans-serif`;
  const weight = (style & 1) !== 0 ? 'bold ' : ''; // Font.BOLD
  const slant = (style & 2) !== 0 ? 'italic ' : ''; // Font.ITALIC
  return `${slant}${weight}${String(size)}px ${css}`;
}

/** Map a JScrollPane scrollbar policy to a CSS overflow value (NEVER=21/31,
 * ALWAYS=22/32, else AS_NEEDED). */
function scrollbarOverflow(policy: number | undefined): string {
  if (policy === 21 || policy === 31) {
    return 'hidden';
  }
  if (policy === 22 || policy === 32) {
    return 'scroll';
  }
  return 'auto';
}

/** CSS for a container, mapping the Swing LayoutManager to flex/grid. */
/** One dimension ("w,h"[index]) of a size hint as a px string, or '' if absent. */
function sizeDim(size: string | undefined, index: 0 | 1): string {
  if (size === undefined) {
    return '';
  }
  const n = Number(size.split(',')[index] ?? 0);
  return `${String(n)}px`;
}

/** Set a control's label text, underlining the mnemonic letter and exposing
 * its Alt-shortcut (setMnemonic). The full text remains the accessible name. */
function setMnemonicLabel(el: HTMLElement, text: string, mnemonic: string | undefined): void {
  const at = mnemonic === undefined || mnemonic === '' ? -1 : text.toUpperCase().indexOf(mnemonic.toUpperCase());
  if (at < 0) {
    el.textContent = text;
    el.removeAttribute('aria-keyshortcuts');
    return;
  }
  const underline = document.createElement('u');
  underline.textContent = text.charAt(at);
  el.replaceChildren(
    document.createTextNode(text.slice(0, at)),
    underline,
    document.createTextNode(text.slice(at + 1)),
  );
  el.setAttribute('aria-keyshortcuts', `Alt+${mnemonic?.toUpperCase() ?? ''}`);
}

/** A SwingConstants horizontal alignment as a CSS text-align. */
function horizAlign(align: number): string {
  if (align === 0) {
    return 'center'; // CENTER
  }
  if (align === 4 || align === 11) {
    return 'right'; // RIGHT / TRAILING
  }
  return 'left'; // LEFT / LEADING (2 / 10)
}

/** Place a GridBagLayout's children into CSS grid cells from their constraints:
 * resolve grid positions (RELATIVE flows to the next cell, REMAINDER spans to
 * the row end), size weighted rows/columns as fr tracks, and apply each child's
 * fill/anchor (as justify/align-self) and insets (as margin). */
function layoutGridBag(container: HTMLElement, pairs: { el: HTMLElement; node: SwingNode }[]): void {
  interface Cell {
    el: HTMLElement;
    gbc: GbcSpec;
    x: number;
    y: number;
    w: number;
    h: number;
    remainder: boolean;
  }
  const cells: Cell[] = [];
  let curX = 0;
  let curY = 0;
  for (const { el, node } of pairs) {
    const gbc = node.gbc ?? DEFAULT_GBC;
    const x = gbc.gridx < 0 ? curX : gbc.gridx; // RELATIVE = -1 → next in row
    const y = gbc.gridy < 0 ? curY : gbc.gridy;
    const remainder = gbc.gridwidth === 0; // REMAINDER: span to the row's end
    const w = remainder || gbc.gridwidth < 0 ? 1 : gbc.gridwidth;
    const h = gbc.gridheight <= 0 ? 1 : gbc.gridheight;
    cells.push({ el, gbc, x, y, w, h, remainder });
    if (remainder) {
      curX = 0;
      curY = y + 1; // REMAINDER ends the row
    } else {
      curX = x + w;
      curY = y;
    }
  }
  // Grid extent: REMAINDER cells then span from their x to the last column.
  let maxCol = 0;
  let maxRow = 0;
  for (const c of cells) {
    maxCol = Math.max(maxCol, c.x + (c.remainder ? 1 : c.w));
    maxRow = Math.max(maxRow, c.y + c.h);
  }
  for (const c of cells) {
    if (c.remainder) {
      c.w = Math.max(1, maxCol - c.x);
    }
  }
  // A column/row is an fr track when a child in it carries weight, else auto.
  const colWeighted = new Array<boolean>(maxCol).fill(false);
  const rowWeighted = new Array<boolean>(maxRow).fill(false);
  for (const c of cells) {
    if (c.gbc.weightx > 0 && c.x < maxCol) {
      colWeighted[c.x] = true;
    }
    if (c.gbc.weighty > 0 && c.y < maxRow) {
      rowWeighted[c.y] = true;
    }
  }
  container.style.gridTemplateColumns = colWeighted.map((wt) => (wt ? '1fr' : 'auto')).join(' ') || 'auto';
  container.style.gridTemplateRows = rowWeighted.map((wt) => (wt ? '1fr' : 'auto')).join(' ') || 'auto';
  for (const c of cells) {
    c.el.style.gridColumn = `${String(c.x + 1)} / span ${String(c.w)}`;
    c.el.style.gridRow = `${String(c.y + 1)} / span ${String(c.h)}`;
    const fill = c.gbc.fill;
    c.el.style.justifySelf = fill === 1 || fill === 2 ? 'stretch' : gbJustify(c.gbc.anchor);
    c.el.style.alignSelf = fill === 1 || fill === 3 ? 'stretch' : gbAlign(c.gbc.anchor);
    const [t, l, b, r] = c.gbc.insets.split(',').map(Number);
    c.el.style.margin = `${String(t)}px ${String(r)}px ${String(b)}px ${String(l)}px`;
  }
}

const DEFAULT_GBC: GbcSpec = {
  gridx: -1,
  gridy: -1,
  gridwidth: 1,
  gridheight: 1,
  weightx: 0,
  weighty: 0,
  anchor: 10,
  fill: 0,
  insets: '0,0,0,0',
};

/** A GridBag anchor's horizontal justify-self (EAST-ish → end, WEST-ish → start). */
function gbJustify(anchor: number): string {
  if (anchor === 12 || anchor === 13 || anchor === 14) {
    return 'end'; // NORTHEAST / EAST / SOUTHEAST
  }
  if (anchor === 16 || anchor === 17 || anchor === 18) {
    return 'start'; // SOUTHWEST / WEST / NORTHWEST
  }
  return 'center';
}

/** A GridBag anchor's vertical align-self (NORTH-ish → start, SOUTH-ish → end). */
function gbAlign(anchor: number): string {
  if (anchor === 11 || anchor === 12 || anchor === 18) {
    return 'start'; // NORTH / NORTHEAST / NORTHWEST
  }
  if (anchor === 14 || anchor === 15 || anchor === 16) {
    return 'end'; // SOUTHEAST / SOUTH / SOUTHWEST
  }
  return 'center';
}

/** Absolutely position a child from its setBounds "x,y,w,h" under a null layout,
 * returning the running max extent so the parent can be sized to fit. */
function placeAbsolute(
  el: HTMLElement,
  child: SwingNode,
  extent: { w: number; h: number },
): { w: number; h: number } {
  el.style.position = 'absolute';
  if (child.bounds === undefined) {
    return extent; // no bounds: left at 0,0 (real Swing would too)
  }
  const parts = child.bounds.split(',').map(Number);
  const x = parts[0] ?? 0;
  const y = parts[1] ?? 0;
  const w = parts[2] ?? 0;
  const h = parts[3] ?? 0;
  el.style.left = `${String(x)}px`;
  el.style.top = `${String(y)}px`;
  el.style.width = `${String(w)}px`;
  el.style.height = `${String(h)}px`;
  el.style.margin = '0';
  return { w: Math.max(extent.w, x + w), h: Math.max(extent.h, y + h) };
}

function applyLayout(el: HTMLElement, layout: string | undefined): void {
  if (layout?.startsWith('grid ')) {
    const cols = Number(layout.split(' ')[2] ?? 1);
    el.style.display = 'grid';
    el.style.gridTemplateColumns = `repeat(${String(Math.max(cols, 1))}, auto)`;
    el.style.gap = '8px';
    el.style.alignItems = 'center';
    return;
  }
  if (layout === 'border') {
    // Five regions: NORTH/SOUTH span the full width, WEST/EAST flank a CENTER
    // that takes the remaining space. Children are placed by grid-area (set in
    // children()); an empty region collapses to nothing.
    el.style.display = 'grid';
    el.style.gridTemplateAreas = '"north north north" "west center east" "south south south"';
    el.style.gridTemplateRows = 'auto 1fr auto';
    el.style.gridTemplateColumns = 'auto 1fr auto';
    el.style.gap = '8px';
    return;
  }
  if (layout?.startsWith('box ')) {
    // BoxLayout: stack along one axis (Y_AXIS/PAGE_AXIS = column, else row).
    // No default gap — spacing comes from Box struts/glue.
    const axis = Number(layout.split(' ')[1] ?? 1);
    el.style.display = 'flex';
    el.style.flexDirection = axis === 1 || axis === 3 ? 'column' : 'row';
    el.style.alignItems = 'flex-start';
    el.style.gap = '0';
    return;
  }
  if (layout === 'none') {
    // A null layout: children are positioned absolutely by their setBounds
    // (applied in children()/#reconcileChildren); the container is the origin.
    el.style.display = 'block';
    el.style.position = 'relative';
    el.style.flexWrap = '';
    el.style.gap = '';
    return;
  }
  if (layout === 'gridbag') {
    // The grid template + per-child placement come from each child's
    // GridBagConstraints (applied in layoutGridBag); just establish the grid.
    el.style.display = 'grid';
    el.style.gap = '0';
    el.style.flexWrap = '';
    return;
  }
  // FlowLayout (the default) lays out inline.
  el.style.display = 'flex';
  el.style.flexWrap = 'wrap';
  el.style.gap = '8px';
  el.style.alignItems = 'center';
}

export class SwingViz {
  // When set (interactive runs), activating a control calls this with the
  // event payload; when null (Phase 1 batch render) controls are inert.
  #onEvent: ((payload: string) => void) | null = null;
  // Live field controls by id, so any event can report every field's
  // current value back to the program (what the user typed / toggled / chose).
  #fields = new Map<string, FieldElement>();
  // Cleanups to run before the next render (e.g. document-level listeners
  // added by the menu bar for click-away dismissal).
  #teardown: (() => void)[] = [];
  // Rendered elements by component id, so re-renders REUSE them (reconcile)
  // instead of tearing down and rebuilding the whole tree each tick — the
  // window persists, the canvas keeps its pixels, and focus/scroll survive.
  #byId = new Map<string, HTMLElement>();
  // The last requestFocus token honored, so a repeat render doesn't refocus.
  #lastFocusRequest = '';

  constructor(private readonly mount: HTMLElement) {}

  /** Dispatch an event payload to the program (no-op in batch mode). Listeners
   * call this rather than capturing `#onEvent`, so a retained element always
   * reaches the current callback. */
  #dispatch(payload: string): void {
    this.#onEvent?.(payload);
  }

  /** Remove any previously rendered UI. */
  clear(): void {
    this.#onEvent = null;
    this.#fields.clear();
    this.#byId.clear();
    this.mount.replaceChildren();
  }

  /**
   * Parse a `swing.json` tree and render it as accessible DOM. Pass
   * `onEvent` for interactive runs: activating a control (a button) invokes
   * it with the event payload, which the engine's event loop consumes.
   *
   * Re-renders RECONCILE against the previous DOM: an element is reused for a
   * component id whenever its kind matches, and only its mutable state is
   * updated. So the window isn't recreated each tick (like a native OS window),
   * a canvas keeps its pixels, and — because elements persist — focus and
   * scroll position survive without any save/restore dance.
   */
  render(json: string, onEvent?: (payload: string) => void): void {
    // Focus can still be lost when an element's CONTENT is rebuilt (table
    // rows); note it so we can put it back afterward.
    const hadFocusInMount = this.mount.contains(document.activeElement);
    const focusedId =
      hadFocusInMount && document.activeElement instanceof HTMLElement
        ? document.activeElement.id
        : '';
    for (const cleanup of this.#teardown) {
      cleanup();
    }
    this.#teardown = [];
    this.#onEvent = onEvent ?? null;
    this.#fields.clear();
    const prev = this.#byId;
    this.#byId = new Map();
    const root = JSON.parse(json) as SwingNode;
    const rootEl = this.#reconcile(root, prev);
    if (this.mount.firstElementChild !== rootEl) {
      this.mount.replaceChildren(rootEl);
    }
    if (focusedId !== '') {
      const again = this.mount.querySelector<HTMLElement>(`#${CSS.escape(focusedId)}`);
      if (again && document.activeElement !== again) {
        again.focus();
      }
    } else if (!hadFocusInMount) {
      // Nothing was focused: put focus on the keyboard surface (if any) so a
      // game is immediately playable. It persists across ticks (the canvas is
      // reused, not rebuilt), so this fires only on the first frame — and it
      // won't yank focus back if the user has since moved to another control.
      this.mount.querySelector<HTMLElement>('[data-key-surface]')?.focus();
    }
    // Component.requestFocus(): the engine sends "<cid>:<seq>" while a request
    // is pending; honor a NEW one (by moving focus), then remember it so a
    // repeat render doesn't keep yanking focus back. Deferred to a task so the
    // focus lands after the host has made the Swing view visible — focus() on a
    // still-hidden (display:none) element is a no-op.
    if (root.focus !== undefined && root.focus !== this.#lastFocusRequest) {
      this.#lastFocusRequest = root.focus;
      this.#applyFocusRequest(root.focus.split(':')[0] ?? '', 20);
    }
  }

  /** Move focus to the requested component, retrying across frames until the
   * host has made the Swing view visible — focus() on an element inside a
   * hidden (display:none) subtree is a no-op, and the view un-hides a tick or
   * two after render() (React flushes its state update asynchronously). */
  #applyFocusRequest(cid: string, attempts: number): void {
    const target = this.mount.querySelector<HTMLElement>(`#${CSS.escape(cid)}`);
    if (target !== null) {
      if (target.closest('[hidden]') === null) {
        target.focus(); // visible now — apply it
        return;
      }
    }
    if (attempts > 0) {
      requestAnimationFrame(() => {
        this.#applyFocusRequest(cid, attempts - 1);
      });
    }
  }

  /** Reuse the element previously rendered for this component id (if its kind
   * matches) and patch its mutable state; otherwise build it fresh. */
  #reconcile(node: SwingNode, prev: Map<string, HTMLElement>): HTMLElement {
    const old = prev.get(node.id);
    if (old?.dataset.kind === node.type) {
      this.#patch(old, node, prev);
      this.#byId.set(node.id, old);
      return old;
    }
    const el = this.build(node);
    this.#byId.set(node.id, el);
    return el;
  }

  /** Reconcile a container's children in order: reuse/patch each by id, insert
   * new ones, drop the leftovers, and re-apply BorderLayout regions. */
  #reconcileChildren(host: HTMLElement, node: SwingNode, prev: Map<string, HTMLElement>): void {
    const nodes = node.children ?? [];
    const border = node.layout === 'border';
    const absolute = node.layout === 'none';
    const gridbag = node.layout === 'gridbag';
    const gbPairs: { el: HTMLElement; node: SwingNode }[] = [];
    let extent = { w: 0, h: 0 };
    for (const [i, child] of nodes.entries()) {
      const el = this.#reconcile(child, prev);
      if (border) {
        el.style.gridArea = (child.region ?? 'Center').toLowerCase();
      } else if (absolute) {
        extent = placeAbsolute(el, child, extent);
      } else if (gridbag) {
        gbPairs.push({ el, node: child });
      }
      const current = host.children[i];
      if (current !== el) {
        host.insertBefore(el, current ?? null);
      }
    }
    while (host.children.length > nodes.length) {
      host.lastElementChild?.remove();
    }
    if (gridbag) {
      layoutGridBag(host, gbPairs);
    }
    if (absolute) {
      // Floor to the container's own size (e.g. a frame's setSize) so the null
      // layout shows the intended area, not just a box around the children.
      host.style.minWidth = `${String(Math.max(extent.w, node.width ?? 0))}px`;
      host.style.minHeight = `${String(Math.max(extent.h, node.height ?? 0))}px`;
    }
  }

  /** Update the mutable state of a reused element (and reconcile its children). */
  #patch(el: HTMLElement, node: SwingNode, prev: Map<string, HTMLElement>): void {
    this.common(el, node);
    switch (node.type) {
      case 'frame':
        this.#patchFrame(el, node, prev);
        break;
      case 'panel':
        this.#patchPanel(el, node, prev);
        break;
      case 'scrollpane':
        this.#patchScroll(el, node, prev);
        break;
      case 'tabbedpane':
        this.#buildTabs(el, node, prev);
        break;
      case 'splitpane':
        this.#buildSplit(el, node, prev);
        break;
      case 'toolbar':
        this.#reconcileChildren(el, node, prev);
        this.#toolbarRoving(el);
        break;
      case 'label':
        // A <label for> may carry a mnemonic (underline + accessKey); a plain
        // status label just updates its text (setMnemonicLabel handles both).
        setMnemonicLabel(el, node.text ?? '', node.mnemonic);
        if (el instanceof HTMLLabelElement) {
          el.accessKey = node.mnemonic ?? '';
        }
        break;
      case 'button':
        setMnemonicLabel(el, node.text ?? '', node.mnemonic);
        (el as HTMLButtonElement).disabled = node.enabled === false;
        (el as HTMLButtonElement).accessKey = node.mnemonic ?? '';
        break;
      case 'toggle':
        el.textContent = node.text ?? '';
        (el as HTMLButtonElement).disabled = node.enabled === false;
        el.setAttribute('aria-pressed', String(node.selected === true));
        break;
      case 'textfield':
      case 'textarea': {
        const input = el as HTMLInputElement | HTMLTextAreaElement;
        // Don't clobber the caret/selection when the value is unchanged.
        if (input.value !== (node.text ?? '')) {
          input.value = node.text ?? '';
        }
        input.disabled = node.enabled === false;
        input.readOnly = node.editable === false; // JTextField / JTextArea setEditable
        applyCaretRequest(input, node); // after the value: setting it resets the caret
        this.#fields.set(node.id, input);
        break;
      }
      case 'checkbox':
      case 'radio': {
        const input = el.querySelector('input');
        const span = el.querySelector('span');
        if (input) {
          input.checked = node.selected === true;
          input.disabled = node.enabled === false;
          this.#fields.set(node.id, input);
        }
        if (span) {
          span.textContent = node.text ?? '';
        }
        break;
      }
      case 'combobox':
        // An editable combo is a wrapper (input + datalist), not a <select>.
        if (el instanceof HTMLSelectElement) {
          this.#patchSelect(el, node);
        } else {
          this.#patchEditableCombo(el, node);
        }
        break;
      case 'list':
        this.#patchSelect(el as HTMLSelectElement, node);
        break;
      case 'slider': {
        const input = el as HTMLInputElement;
        input.min = String(node.min ?? 0);
        input.max = String(node.max ?? 100);
        input.value = String(node.value ?? 0);
        input.disabled = node.enabled === false;
        this.#fields.set(node.id, input);
        break;
      }
      case 'table':
        this.#patchTable(el as HTMLTableElement, node);
        break;
      case 'progressbar':
        this.#progressState(el, node);
        break;
      case 'spinner': {
        const input = el as HTMLInputElement;
        input.min = String(node.min ?? 0);
        input.max = String(node.max ?? 100);
        input.step = String(node.step ?? 1);
        if (input.value !== String(node.value ?? 0)) {
          input.value = String(node.value ?? 0);
        }
        input.disabled = node.enabled === false;
        this.#fields.set(node.id, input);
        break;
      }
      default:
        break;
    }
  }

  #patchFrame(el: HTMLElement, node: SwingNode, prev: Map<string, HTMLElement>): void {
    el.setAttribute(
      'aria-label',
      node.title === undefined || node.title === '' ? 'Window' : node.title,
    );
    const titleText = el.querySelector('.swing-titlebar span');
    if (titleText) {
      titleText.textContent = node.title ?? '';
    }
    // The menu bar is cheap and its click-away listeners were just torn down —
    // rebuild it rather than reconcile its nested structure.
    el.querySelector(':scope > .swing-menubar')?.remove();
    const content = el.querySelector<HTMLElement>(':scope > .swing-content');
    if (content && node.menubar) {
      el.insertBefore(this.menuBar(node.menubar), content);
    }
    if (content) {
      applyLayout(content, node.layout);
      this.#reconcileChildren(content, node, prev);
    }
  }

  #patchPanel(el: HTMLElement, node: SwingNode, prev: Map<string, HTMLElement>): void {
    if (node.paint !== undefined) {
      const canvas = el.querySelector('canvas');
      if (canvas) {
        paintCanvas(canvas, node.paint); // clears then replays — a full redraw
      }
      return;
    }
    applyLayout(el, node.layout);
    this.#reconcileChildren(el, node, prev);
  }

  #patchScroll(el: HTMLElement, node: SwingNode, prev: Map<string, HTMLElement>): void {
    el.style.width = `${String(node.pw ?? 200)}px`;
    el.style.height = `${String(node.ph ?? 150)}px`;
    el.style.overflowX = scrollbarOverflow(node.hpolicy);
    el.style.overflowY = scrollbarOverflow(node.vpolicy);
    if (node.view) {
      const child = this.#reconcile(node.view, prev);
      if (el.firstElementChild !== child) {
        el.replaceChildren(child);
      }
    }
  }

  #patchSelect(select: HTMLSelectElement, node: SwingNode): void {
    const items = node.items ?? [];
    const same =
      select.options.length === items.length &&
      items.every((item, i) => select.options[i]?.textContent === item);
    if (!same) {
      select.replaceChildren(
        ...items.map((item, i) => {
          const option = document.createElement('option');
          option.value = String(i);
          option.textContent = item;
          return option;
        }),
      );
    }
    if (node.type === 'list') {
      this.#applyItemStyles(select, node);
      const selected = new Set(node.selectedIndices ?? []);
      for (let i = 0; i < select.options.length; i++) {
        const option = select.options[i];
        if (option) {
          option.selected = selected.has(i);
        }
      }
    } else {
      select.selectedIndex = node.selectedIndex ?? -1;
    }
    select.disabled = node.enabled === false;
    this.#fields.set(node.id, select);
  }

  #patchTable(table: HTMLTableElement, node: SwingNode): void {
    // If a cell is mid-edit, leave the table alone — an unrelated re-render
    // (e.g. another widget updating) must not destroy the active editor.
    if (table.querySelector('.swing-cell-editor')) {
      return;
    }
    // Rebuild the head/body (rows change with the model) but keep the <table>
    // element itself, so its place in a scroll pane and identity are stable.
    const fresh = this.table(node);
    const role = fresh.getAttribute('role');
    if (role !== null) {
      table.setAttribute('role', role);
    } else {
      table.removeAttribute('role');
    }
    table.replaceChildren(...Array.from(fresh.childNodes));
  }

  /**
   * The event payload for activating `clickedId`: its id, then one
   * `id=value` line per live field so the program reads the user's current
   * input (text) and checkbox states when the listener runs.
   */
  #payload(clickedId: string): string {
    let payload = clickedId;
    for (const [id, el] of this.#fields) {
      payload += `\n${id}=${fieldValue(el)}`;
      // A text component also reports its live cursor, so getCaretPosition()
      // and getSelectedText() see what the user actually has selected.
      const caret = caretOf(el);
      if (caret !== null) {
        payload += `\n__caret:${id}=${caret}`;
      }
    }
    return payload;
  }

  /**
   * Register a live field and, when it has a listener, dispatch its value to
   * the program on the given native event. Fields are always registered (so
   * their value rides along with any event); the event is wired only when the
   * widget listens, so an input read at submit-time doesn't round-trip.
   */
  #field(el: FieldElement, node: SwingNode, event: 'change'): void {
    this.#fields.set(node.id, el);
    if (node.listens === true) {
      const id = node.id;
      el.addEventListener(event, () => {
        this.#dispatch(this.#payload(id));
      });
    }
  }

  /** A text component with a DocumentListener dispatches on every input, so the
   * listener fires per keystroke (the engine picks insert vs remove by the
   * text-length delta). Marked with a "__doc=" line so it routes to __onDoc. */
  #documentInput(el: FieldElement, node: SwingNode): void {
    if (node.docListen !== true) {
      return;
    }
    const id = node.id;
    el.addEventListener('input', () => {
      this.#dispatch(`${this.#payload(id)}\n__doc=${id}`);
    });
  }

  /** Component-relative coordinates of a pointer event (mapped to the
   * canvas's own pixel grid when it is CSS-scaled). */
  #pointer(el: HTMLElement, event: MouseEvent): { x: number; y: number } {
    const rect = el.getBoundingClientRect();
    const scaleX = el instanceof HTMLCanvasElement && rect.width ? el.width / rect.width : 1;
    const scaleY = el instanceof HTMLCanvasElement && rect.height ? el.height / rect.height : 1;
    return {
      x: Math.round((event.clientX - rect.left) * scaleX),
      y: Math.round((event.clientY - rect.top) * scaleY),
    };
  }

  /** The event payload for a pointer event: `<cid>\n<kind>=x,y` + field state. */
  #pointerPayload(id: string, kind: '__mouse' | '__drag', x: number, y: number): string {
    let payload = `${id}\n${kind}=${String(x)},${String(y)}`;
    for (const [fieldId, field] of this.#fields) {
      payload += `\n${fieldId}=${fieldValue(field)}`;
    }
    return payload;
  }

  /**
   * Wire a panel with a MouseListener: a click reports component-relative
   * coordinates. Pointer-only — a bitmap click target has no keyboard
   * equivalent.
   */
  #mouse(el: HTMLElement, node: SwingNode): void {
    if (node.mouse !== true) {
      return;
    }
    const id = node.id;
    if (el instanceof HTMLCanvasElement) {
      el.style.cursor = 'crosshair';
    }
    el.addEventListener('click', (event) => {
      const { x, y } = this.#pointer(el, event);
      this.#dispatch(this.#pointerPayload(id, '__mouse', x, y));
    });
  }

  /**
   * Wire a panel with a MouseMotionListener: while a button is held, each
   * move reports a drag position. These fire rapidly — the app layer
   * coalesces them so the loop processes only the latest per render.
   */
  #motion(el: HTMLElement, node: SwingNode): void {
    if (node.drag !== true) {
      return;
    }
    const id = node.id;
    if (el instanceof HTMLCanvasElement) {
      el.style.cursor = 'crosshair';
    }
    el.addEventListener('mousemove', (event) => {
      if ((event.buttons & 1) === 0) {
        return; // only a drag (primary button held), not a passive move
      }
      const { x, y } = this.#pointer(el, event);
      this.#dispatch(this.#pointerPayload(id, '__drag', x, y));
    });
  }

  /** The event payload for a keyboard event: `<cid>\n__key=type,code,char` + field state. */
  #keyPayload(id: string, type: number, code: number, ch: number): string {
    let payload = `${id}\n__key=${String(type)},${String(code)},${String(ch)}`;
    for (const [fieldId, field] of this.#fields) {
      payload += `\n${fieldId}=${fieldValue(field)}`;
    }
    return payload;
  }

  /**
   * Wire a panel with a KeyListener: make it focusable and report each
   * keydown (type 0) / keyup (type 1) with the AWT key code and character.
   * Arrow/space/paging keys are prevented from scrolling the page while the
   * surface is focused, so a game reads them cleanly.
   */
  #keys(el: HTMLElement, node: SwingNode): void {
    if (node.key !== true) {
      return;
    }
    const id = node.id;
    el.tabIndex = 0;
    // Marks the keyboard surface so render() can focus it on the first frame.
    el.dataset.keySurface = 'true';
    if (el instanceof HTMLCanvasElement) {
      el.style.outline = 'none';
    }
    const send = (type: number) => (event: KeyboardEvent) => {
      if (SCROLL_KEYS.has(event.key)) {
        event.preventDefault(); // don't let a game's controls scroll the page
      }
      this.#dispatch(this.#keyPayload(id, type, awtKeyCode(event), awtKeyChar(event)));
    };
    el.addEventListener('keydown', send(0));
    el.addEventListener('keyup', send(1));
  }

  private build(node: SwingNode): HTMLElement {
    switch (node.type) {
      case 'frame':
        return this.frame(node);
      case 'panel':
        return this.panel(node);
      case 'label':
        return this.label(node);
      case 'button':
        return this.button(node);
      case 'toggle':
        return this.toggleButton(node);
      case 'textfield':
        return this.textField(node);
      case 'textarea':
        return this.textArea(node);
      case 'checkbox':
        return this.checkBox(node);
      case 'radio':
        return this.radioButton(node);
      case 'combobox':
        return this.comboBox(node);
      case 'list':
        return this.listBox(node);
      case 'slider':
        return this.slider(node);
      case 'scrollpane':
        return this.scrollPane(node);
      case 'tabbedpane':
        return this.tabbedPane(node);
      case 'splitpane':
        return this.splitPaneSplit(node);
      case 'toolbar':
        return this.toolBar(node);
      case 'toolbarsep':
        return this.toolBarSeparator(node);
      case 'table':
        return this.table(node);
      case 'progressbar':
        return this.progressBar(node);
      case 'spinner':
        return this.spinner(node);
      case 'strut':
        return this.strut(node);
      default: {
        const span = document.createElement('span');
        this.common(span, node);
        return span;
      }
    }
  }

  /** Shared props: id (label-for target), enabled, tooltip, colors. */
  private common(el: HTMLElement, node: SwingNode): void {
    el.dataset.cid = node.id;
    el.dataset.kind = node.type; // reconcile reuses an element only if kind matches
    el.hidden = node.hidden === true; // Component.setVisible(false)
    if (node.tooltip !== undefined) {
      el.title = node.tooltip;
    }
    // AccessibleContext.setAccessibleName / setAccessibleDescription. An
    // explicit accessible name is authoritative, so it overrides a widget's
    // default (text-content or tooltip) name; common() runs last, after the
    // builder's own aria-label, so this wins.
    if (node.accName !== undefined) {
      el.setAttribute('aria-label', node.accName);
    }
    if (node.accDesc !== undefined) {
      el.setAttribute('aria-description', node.accDesc);
    }
    const bg = cssColor(node.bg);
    if (bg !== null) {
      el.style.backgroundColor = bg;
    }
    const fg = cssColor(node.fg);
    if (fg !== null) {
      el.style.color = fg;
    }
    // Component.setFont: "<style> <size> <family>". Cleared on reconcile when a
    // later render drops the font, so a reused element doesn't keep a stale one.
    if (node.font !== undefined) {
      const parts = node.font.split(' ');
      el.style.font = cssFont(Number(parts[0]), Number(parts[1]), parts.slice(2).join(' '));
    } else {
      el.style.font = '';
    }
    // Size hints (setPreferredSize / setMinimumSize / setMaximumSize). Applied
    // only when set — a container's builder (scroll/split panes) sizes itself,
    // so clearing here would clobber it.
    if (node.psize !== undefined) {
      el.style.width = sizeDim(node.psize, 0);
      el.style.height = sizeDim(node.psize, 1);
    }
    if (node.minsize !== undefined) {
      el.style.minWidth = sizeDim(node.minsize, 0);
      el.style.minHeight = sizeDim(node.minsize, 1);
    }
    if (node.maxsize !== undefined) {
      el.style.maxWidth = sizeDim(node.maxsize, 0);
      el.style.maxHeight = sizeDim(node.maxsize, 1);
    }
    // setHorizontalAlignment → text-align (right-aligns a field's text, etc.).
    el.style.textAlign = node.halign === undefined ? '' : horizAlign(node.halign);
    this.applyBorder(el, node.border);
    // A named plain JPanel becomes an accessible group so assistive tech
    // actually exposes its name — an aria-label on a role-less div is ignored.
    // (A titled border already provides a group role via applyBorder; a painted
    // panel's canvas carries its own name.)
    if (
      node.type === 'panel' &&
      node.paint === undefined &&
      node.accName !== undefined &&
      !el.hasAttribute('role')
    ) {
      el.setAttribute('role', 'group');
    }
  }

  /** Apply a BorderFactory border as CSS. A compound border layers its inner
   * border inside the outer frame. Resets prior border styling first so a
   * reconciled element doesn't keep a stale border. */
  private applyBorder(el: HTMLElement, border: BorderSpec | undefined): void {
    // Undo any titled-group state from a previous render before reapplying.
    if (el.dataset.borderTitle !== undefined && border?.type !== 'titled') {
      el.classList.remove('swing-titled');
      delete el.dataset.borderTitle;
      el.removeAttribute('role');
      el.removeAttribute('aria-label');
    }
    el.style.border = '';
    el.style.padding = '';
    el.style.boxShadow = '';
    if (border === undefined) {
      return;
    }
    if (border.type === 'compound') {
      if (border.outer) {
        this.frameBorder(el, border.outer);
      }
      if (border.inner) {
        this.innerBorder(el, border.inner);
      }
      return;
    }
    this.frameBorder(el, border);
  }

  /** The visible frame of a single (non-compound) border on the element. */
  private frameBorder(el: HTMLElement, b: BorderSpec): void {
    const [t, l, bottom, r] = b.insets.split(',').map(Number);
    switch (b.type) {
      case 'line':
        el.style.border = `${String(b.thickness)}px solid ${cssColor(b.color) ?? 'currentColor'}`;
        break;
      case 'matte':
        // Per-side thickness in one colour.
        el.style.borderStyle = 'solid';
        el.style.borderColor = cssColor(b.color) ?? 'currentColor';
        el.style.borderWidth = `${String(t)}px ${String(r)}px ${String(bottom)}px ${String(l)}px`;
        break;
      case 'etched':
        el.style.border = `${String(b.thickness)}px groove light-dark(#c8c8c0, #5a5a5a)`;
        break;
      case 'bevel':
        el.style.border = `2px ${b.raised === true ? 'outset' : 'inset'} light-dark(#d8d8d0, #565656)`;
        break;
      case 'empty':
        el.style.padding = `${String(t)}px ${String(r)}px ${String(bottom)}px ${String(l)}px`;
        break;
      case 'titled':
        // A group box: an accessible region named by its caption, with the
        // caption drawn over the top border via the ::before rule.
        el.classList.add('swing-titled');
        el.dataset.borderTitle = b.title ?? '';
        el.setAttribute('role', 'group');
        el.setAttribute('aria-label', b.title ?? '');
        // TitledBorder(border, title) frames with that border (inline, so it
        // overrides the default etched groove from the CSS class).
        if (b.inner) {
          this.frameBorder(el, b.inner);
        }
        break;
      default:
        break;
    }
  }

  /** A compound border's inner contribution, layered inside the outer frame:
   * padding for an empty border, an inset ring for a line/matte border. */
  private innerBorder(el: HTMLElement, b: BorderSpec): void {
    if (b.type === 'empty') {
      const [t, l, bottom, r] = b.insets.split(',').map(Number);
      el.style.padding = `${String(t)}px ${String(r)}px ${String(bottom)}px ${String(l)}px`;
    } else if (b.type === 'line' || b.type === 'matte') {
      el.style.boxShadow = `inset 0 0 0 ${String(b.thickness)}px ${cssColor(b.color) ?? 'currentColor'}`;
    }
  }

  private children(container: HTMLElement, node: SwingNode): void {
    const border = node.layout === 'border';
    const absolute = node.layout === 'none';
    const gridbag = node.layout === 'gridbag';
    const gbPairs: { el: HTMLElement; node: SwingNode }[] = [];
    let extent = { w: 0, h: 0 };
    for (const child of node.children ?? []) {
      const el = this.build(child);
      if (border) {
        // Place the child in its BorderLayout region; unconstrained children
        // default to the center, as real BorderLayout does.
        el.style.gridArea = (child.region ?? 'Center').toLowerCase();
      } else if (absolute) {
        extent = placeAbsolute(el, child, extent);
      } else if (gridbag) {
        gbPairs.push({ el, node: child });
      }
      container.appendChild(el);
    }
    if (gridbag) {
      layoutGridBag(container, gbPairs);
    }
    if (absolute) {
      // Absolute children don't stretch the container, so floor its size to the
      // furthest child edge (and the container's own size, e.g. a frame's
      // setSize) — otherwise it would collapse to nothing.
      container.style.minWidth = `${String(Math.max(extent.w, node.width ?? 0))}px`;
      container.style.minHeight = `${String(Math.max(extent.h, node.height ?? 0))}px`;
    }
  }

  private frame(node: SwingNode): HTMLElement {
    // A window is a named region landmark: screen readers can jump to it and
    // announce its title, and the title is not repeated on every child.
    const section = document.createElement('section');
    section.className = 'swing-window';
    section.setAttribute(
      'aria-label',
      node.title === undefined || node.title === '' ? 'Window' : node.title,
    );
    this.common(section, node);

    const titlebar = document.createElement('div');
    titlebar.className = 'swing-titlebar';
    // The title text is hidden from assistive tech (the region name already
    // conveys it), but the close button stays in the accessibility tree.
    const titleText = document.createElement('span');
    titleText.setAttribute('aria-hidden', 'true');
    titleText.textContent = node.title ?? '';
    titlebar.appendChild(titleText);
    if (this.#onEvent) {
      const close = document.createElement('button');
      close.type = 'button';
      close.className = 'swing-close';
      close.setAttribute('aria-label', 'Close window');
      close.textContent = '×'; // ×
      close.addEventListener('click', () => {
        this.#dispatch(this.#payload('__close'));
      });
      titlebar.appendChild(close);
    }
    section.appendChild(titlebar);

    // The menu bar sits below the title bar, above the content.
    if (node.menubar) {
      section.appendChild(this.menuBar(node.menubar));
    }

    const content = document.createElement('div');
    content.className = 'swing-content';
    applyLayout(content, node.layout);
    if (node.width !== undefined && node.width > 0) {
      content.style.minWidth = `${String(node.width)}px`;
    }
    this.children(content, node);
    section.appendChild(content);
    return section;
  }

  /**
   * An accessible menu bar (WAI-ARIA menu-button pattern): each menu is a
   * button that opens a `role="menu"` popup of `role="menuitem"` buttons, and
   * a menu item can itself be a submenu that cascades a nested popup. Keyboard:
   * open with Enter/Space/ArrowDown, Arrow/Home/End move within a menu,
   * ArrowRight opens a submenu, ArrowLeft/Escape close back to the parent;
   * activating a leaf item dispatches its ActionListener. A click (or focus)
   * anywhere outside the bar collapses every open menu.
   */
  private menuBar(bar: SwingMenuBar): HTMLElement {
    const menubar = document.createElement('div');
    menubar.className = 'swing-menubar';
    menubar.setAttribute('role', 'menubar');

    let openTop: { button: HTMLButtonElement; popup: HTMLElement } | null = null;
    const closeTop = (): void => {
      if (openTop) {
        this.#collapseMenu(openTop.popup, openTop.button);
        openTop = null;
      }
    };

    for (const menu of bar.menus) {
      const wrap = document.createElement('div');
      wrap.className = 'swing-menu';

      const button = document.createElement('button');
      button.type = 'button';
      button.className = 'swing-menu-button';
      button.textContent = menu.text ?? '';
      button.setAttribute('aria-haspopup', 'true');
      button.setAttribute('aria-expanded', 'false');

      const popup = this.#menuPopup(menu, closeTop, () => {
        closeTop();
        button.focus();
      });

      const openThis = (): void => {
        if (openTop?.button !== button) {
          closeTop();
          this.#showPopup(popup, button, false);
          button.setAttribute('aria-expanded', 'true');
          openTop = { button, popup };
        }
      };

      button.addEventListener('click', () => {
        if (openTop?.button === button) {
          closeTop();
        } else {
          openThis();
          this.#focusFirst(popup);
        }
      });
      // Once a menu is open, hovering a sibling menu switches to it.
      button.addEventListener('mouseenter', () => {
        if (openTop && openTop.button !== button) {
          openThis();
        }
      });
      button.addEventListener('keydown', (event) => {
        if (event.key === 'ArrowDown' || event.key === 'Enter' || event.key === ' ') {
          event.preventDefault();
          openThis();
          this.#focusFirst(popup);
        }
      });

      wrap.append(button, popup);
      menubar.appendChild(wrap);
    }

    // Click or focus outside the bar collapses whatever is open.
    const onOutside = (event: Event): void => {
      if (!menubar.contains(event.target as Node)) {
        closeTop();
      }
    };
    document.addEventListener('click', onOutside);
    document.addEventListener('focusin', onOutside);
    this.#teardown.push(() => {
      document.removeEventListener('click', onOutside);
      document.removeEventListener('focusin', onOutside);
    });

    // Menu accelerators (setAccelerator): a document key handler fires the
    // matching item so e.g. Ctrl+S works without opening the menu. Only combos
    // with a modifier are handled globally (a bare-letter accelerator would
    // hijack typing).
    const accelerators = collectAccelerators(bar.menus);
    if (accelerators.length > 0) {
      const onKey = (event: KeyboardEvent): void => {
        const combo = keyComboString(event);
        if (!combo.includes('+')) {
          return;
        }
        const match = accelerators.find((a) => a.accel.toUpperCase() === combo.toUpperCase());
        if (match) {
          event.preventDefault();
          closeTop();
          this.#dispatch(this.#payload(match.id));
        }
      };
      document.addEventListener('keydown', onKey);
      this.#teardown.push(() => {
        document.removeEventListener('keydown', onKey);
      });
    }

    return menubar;
  }

  /** Build one `role="menu"` popup (recursively for submenus). `closeRoot`
   * collapses the whole tree (used when a leaf item fires); `closeSelf` closes
   * just this popup and refocuses its opener (Escape / ArrowLeft). */
  #menuPopup(
    menu: SwingMenu | SwingMenuEntry,
    closeRoot: () => void,
    closeSelf: () => void,
  ): HTMLElement {
    const popup = document.createElement('div');
    popup.className = 'swing-menu-popup';
    popup.setAttribute('role', 'menu');
    popup.setAttribute('aria-label', menu.text ?? 'Menu');
    // A manual popover renders in the browser's top layer, so it is never
    // clipped by the window's `overflow: hidden` (positioned via #showPopup).
    popup.popover = 'manual';

    let openChild: { button: HTMLButtonElement; popup: HTMLElement } | null = null;
    const closeChild = (): void => {
      if (openChild) {
        this.#collapseMenu(openChild.popup, openChild.button);
        openChild = null;
      }
    };

    for (const entry of menu.items ?? []) {
      if (entry.type === 'separator') {
        const separator = document.createElement('div');
        separator.className = 'swing-menu-separator';
        separator.setAttribute('role', 'separator');
        popup.appendChild(separator);
        continue;
      }

      if (entry.type === 'menu') {
        const subButton = document.createElement('button');
        subButton.type = 'button';
        subButton.className = 'swing-menu-item swing-submenu';
        subButton.setAttribute('role', 'menuitem');
        subButton.setAttribute('aria-haspopup', 'true');
        subButton.setAttribute('aria-expanded', 'false');
        subButton.tabIndex = -1;
        subButton.textContent = entry.text ?? '';

        const subPopup = this.#menuPopup(entry, closeRoot, () => {
          closeChild();
          subButton.focus();
        });

        const openSub = (): void => {
          if (openChild?.button !== subButton) {
            closeChild();
            this.#showPopup(subPopup, subButton, true);
            subButton.setAttribute('aria-expanded', 'true');
            openChild = { button: subButton, popup: subPopup };
          }
        };

        subButton.addEventListener('click', (event) => {
          event.stopPropagation();
          if (openChild?.button === subButton) {
            closeChild();
          } else {
            openSub();
          }
        });
        subButton.addEventListener('mouseenter', () => {
          openSub();
        });
        subButton.addEventListener('keydown', (event) => {
          if (event.key === 'ArrowRight' || event.key === 'Enter' || event.key === ' ') {
            event.preventDefault();
            event.stopPropagation();
            openSub();
            this.#focusFirst(subPopup);
          }
        });

        popup.append(subButton, subPopup);
        continue;
      }

      const item = document.createElement('button');
      item.type = 'button';
      item.className = 'swing-menu-item';
      // A check/radio item announces its state (menuitemcheckbox / menuitemradio
      // with aria-checked); a plain item is a menuitem.
      const check = entry.type === 'checkmenuitem';
      const radio = entry.type === 'radiomenuitem';
      item.setAttribute('role', check ? 'menuitemcheckbox' : radio ? 'menuitemradio' : 'menuitem');
      item.tabIndex = -1; // reached via arrow keys, not Tab
      if (check || radio) {
        item.setAttribute('aria-checked', String(entry.selected === true));
        item.classList.add('swing-menu-checkable');
        // A hidden indicator carries the check/bullet (via CSS), kept OUT of the
        // accessible name — aria-checked already conveys state to a reader.
        const indicator = document.createElement('span');
        indicator.className = 'swing-menu-indicator';
        indicator.setAttribute('aria-hidden', 'true');
        item.append(indicator, document.createTextNode(entry.text ?? ''));
      } else {
        item.textContent = entry.text ?? '';
      }
      // setAccelerator: a right-aligned shortcut hint (aria-hidden — the actual
      // shortcut works via the document key handler, and aria-checked/name stay
      // clean).
      if (entry.accel !== undefined) {
        const accel = document.createElement('span');
        accel.className = 'swing-menu-accel';
        accel.setAttribute('aria-hidden', 'true');
        accel.textContent = entry.accel;
        item.appendChild(accel);
      }
      // A disabled item (e.g. from a disabled Action) can't be activated.
      const enabled = entry.enabled !== false;
      item.disabled = !enabled;
      item.setAttribute('aria-disabled', String(!enabled));
      // Moving onto a plain item closes any sibling submenu that was open.
      item.addEventListener('mouseenter', () => {
        closeChild();
      });
      if (enabled && entry.listens === true && entry.id !== undefined) {
        const id = entry.id;
        item.addEventListener('click', (event) => {
          event.stopPropagation();
          closeRoot();
          this.#dispatch(this.#payload(id));
        });
      }
      popup.appendChild(item);
    }

    popup.addEventListener('keydown', (event) => {
      // Skip disabled items — a disabled <button> can't take focus, so it would
      // otherwise trap arrow navigation.
      const items = [...popup.querySelectorAll<HTMLButtonElement>(':scope > [role="menuitem"]')].filter(
        (item) => !item.disabled,
      );
      const index = items.indexOf(document.activeElement as HTMLButtonElement);
      if (event.key === 'ArrowDown') {
        event.preventDefault();
        event.stopPropagation();
        items[(index + 1) % items.length]?.focus();
      } else if (event.key === 'ArrowUp') {
        event.preventDefault();
        event.stopPropagation();
        items[(index - 1 + items.length) % items.length]?.focus();
      } else if (event.key === 'Home') {
        event.preventDefault();
        event.stopPropagation();
        items[0]?.focus();
      } else if (event.key === 'End') {
        event.preventDefault();
        event.stopPropagation();
        items[items.length - 1]?.focus();
      } else if (event.key === 'Escape' || event.key === 'ArrowLeft') {
        event.preventDefault();
        event.stopPropagation();
        closeSelf();
      }
    });

    return popup;
  }

  /** Hide a popup and everything nested inside it, resetting aria-expanded. */
  #collapseMenu(popup: HTMLElement, button: HTMLElement): void {
    // Hide nested popovers first, then this one (a manual popover doesn't
    // auto-close its descendants).
    for (const nested of popup.querySelectorAll<HTMLElement>('[role="menu"]')) {
      if (nested.matches(':popover-open')) {
        nested.hidePopover();
      }
    }
    for (const expanded of popup.querySelectorAll<HTMLElement>('[aria-expanded="true"]')) {
      expanded.setAttribute('aria-expanded', 'false');
    }
    if (popup.matches(':popover-open')) {
      popup.hidePopover();
    }
    button.setAttribute('aria-expanded', 'false');
  }

  /** Show a menu popover in the top layer, positioned relative to its trigger:
   * a top-level menu drops below its button, a submenu flies out to the right.
   * Flips to stay within the viewport. */
  #showPopup(popup: HTMLElement, trigger: HTMLElement, submenu: boolean): void {
    if (!popup.matches(':popover-open')) {
      popup.showPopover();
    }
    const rect = trigger.getBoundingClientRect();
    let left = submenu ? rect.right : rect.left;
    let top = submenu ? rect.top : rect.bottom;
    // Now that it's laid out in the top layer, flip if it would overflow.
    const pw = popup.offsetWidth;
    const ph = popup.offsetHeight;
    if (left + pw > window.innerWidth) {
      left = submenu ? rect.left - pw : window.innerWidth - pw - 4;
    }
    if (top + ph > window.innerHeight) {
      top = (submenu ? rect.bottom : rect.top) - ph;
    }
    popup.style.left = `${String(Math.max(4, left))}px`;
    popup.style.top = `${String(Math.max(4, top))}px`;
  }

  /** Focus a popup's first menu item (its direct children only). */
  #focusFirst(popup: HTMLElement): void {
    // The first item that can actually take focus (skip disabled ones).
    const items = [...popup.querySelectorAll<HTMLButtonElement>(':scope > [role="menuitem"]')];
    items.find((item) => !item.disabled)?.focus();
  }

  private panel(node: SwingNode): HTMLElement {
    if (node.paint !== undefined) {
      return this.canvasPanel(node);
    }
    const panel = document.createElement('div');
    panel.className = 'swing-panel';
    applyLayout(panel, node.layout);
    this.#mouse(panel, node);
    this.#motion(panel, node);
    this.#keys(panel, node);
    this.common(panel, node);
    this.children(panel, node);
    return panel;
  }

  /** A JScrollPane: a fixed-size viewport that scrolls its wrapped view. */
  private scrollPane(node: SwingNode): HTMLElement {
    const pane = document.createElement('div');
    pane.className = 'swing-scrollpane';
    pane.style.width = `${String(node.pw ?? 200)}px`;
    pane.style.height = `${String(node.ph ?? 150)}px`;
    pane.style.overflowX = scrollbarOverflow(node.hpolicy);
    pane.style.overflowY = scrollbarOverflow(node.vpolicy);
    if (node.view) {
      pane.appendChild(this.build(node.view));
      // A textarea scrolls itself; other content needs a focusable, named
      // scroll region so keyboard users can scroll it (arrow keys).
      if (node.view.type !== 'textarea') {
        pane.tabIndex = 0;
        pane.setAttribute('role', 'region');
        pane.setAttribute('aria-label', node.tooltip ?? 'Scroll area');
      }
    }
    this.common(pane, node);
    return pane;
  }

  /** A JTabbedPane: an ARIA tablist of tabs, each controlling a tabpanel; only
   * the selected panel shows. Built fresh; #buildTabs reconciles on patch. */
  private tabbedPane(node: SwingNode): HTMLElement {
    const el = document.createElement('div');
    el.className = 'swing-tabbedpane';
    this.common(el, node);
    this.#buildTabs(el, node, new Map());
    return el;
  }

  /** (Re)build a tabbed pane's tablist + panels, reconciling each tab's
   * component against `prev` so its state survives a tab switch / re-render. */
  #buildTabs(el: HTMLElement, node: SwingNode, prev: Map<string, HTMLElement>): void {
    const tabs = node.tabs ?? [];
    const selected = node.selectedIndex ?? 0;
    const placement = node.placement ?? 1; // TOP
    const vertical = placement === 2 || placement === 4; // LEFT / RIGHT
    el.classList.toggle('swing-tabs-vertical', vertical);

    const tablist = document.createElement('div');
    tablist.className = 'swing-tablist';
    tablist.setAttribute('role', 'tablist');
    if (vertical) {
      tablist.setAttribute('aria-orientation', 'vertical');
    }
    const panels = document.createElement('div');
    panels.className = 'swing-tabpanels';

    tabs.forEach((tab, i) => {
      const tabId = `${node.id}-tab${String(i)}`;
      const panelId = `${node.id}-panel${String(i)}`;
      const isSelected = i === selected;

      const button = document.createElement('button');
      button.type = 'button';
      button.className = 'swing-tab';
      button.id = tabId;
      button.setAttribute('role', 'tab');
      button.setAttribute('aria-selected', String(isSelected));
      button.setAttribute('aria-controls', panelId);
      button.classList.toggle('swing-tab-selected', isSelected);
      button.tabIndex = isSelected ? 0 : -1; // roving: only the active tab tabs in
      button.textContent = tab.title;
      button.addEventListener('click', () => {
        this.#dispatch(`${this.#payload(node.id)}\n${node.id}=${String(i)}`);
      });
      tablist.appendChild(button);

      const panel = document.createElement('div');
      panel.className = 'swing-tabpanel';
      panel.id = panelId;
      panel.setAttribute('role', 'tabpanel');
      panel.setAttribute('aria-labelledby', tabId);
      panel.hidden = !isSelected;
      panel.appendChild(this.#reconcile(tab.component, prev));
      panels.appendChild(panel);
    });

    // Manual-activation ARIA tabs: arrows move focus between tabs; Enter/Space
    // (or click) activates. Manual activation avoids a VM round-trip per arrow.
    tablist.addEventListener('keydown', (event) => {
      const buttons = [...tablist.querySelectorAll<HTMLElement>('[role="tab"]')];
      const current = buttons.indexOf(document.activeElement as HTMLElement);
      if (current < 0) {
        return;
      }
      const nextKey = vertical ? 'ArrowDown' : 'ArrowRight';
      const prevKey = vertical ? 'ArrowUp' : 'ArrowLeft';
      let next: number;
      if (event.key === nextKey) {
        next = (current + 1) % buttons.length;
      } else if (event.key === prevKey) {
        next = (current - 1 + buttons.length) % buttons.length;
      } else if (event.key === 'Home') {
        next = 0;
      } else if (event.key === 'End') {
        next = buttons.length - 1;
      } else {
        return;
      }
      event.preventDefault();
      buttons[current]?.setAttribute('tabindex', '-1');
      const target = buttons[next];
      if (target) {
        target.tabIndex = 0;
        target.focus();
      }
    });

    // Placement: TOP/LEFT put the tablist first; BOTTOM/RIGHT put it last.
    const tabsLast = placement === 3 || placement === 4; // BOTTOM / RIGHT
    el.replaceChildren(...(tabsLast ? [panels, tablist] : [tablist, panels]));
  }

  /** A JSplitPane: two panes separated by a draggable divider. Built fresh;
   * #buildSplit reconciles on patch and preserves a user-dragged divider. */
  private splitPaneSplit(node: SwingNode): HTMLElement {
    const el = document.createElement('div');
    el.className = 'swing-splitpane';
    this.common(el, node);
    this.#buildSplit(el, node, new Map());
    return el;
  }

  /** (Re)build a split pane's two panes + divider, reconciling each side's
   * component against `prev`. A divider the user has dragged (recorded in a data
   * attribute) is preserved across re-renders; otherwise the set location seeds
   * it, and <=0 means an even split. */
  #buildSplit(el: HTMLElement, node: SwingNode, prev: Map<string, HTMLElement>): void {
    const vertical = node.orientation === 0; // VERTICAL_SPLIT
    el.classList.toggle('swing-split-vertical', vertical);

    const pane1 = document.createElement('div');
    pane1.className = 'swing-split-pane1';
    const pane2 = document.createElement('div');
    pane2.className = 'swing-split-pane2';
    const divider = document.createElement('div');
    divider.className = 'swing-split-divider';
    divider.setAttribute('role', 'separator');
    divider.setAttribute('aria-orientation', vertical ? 'horizontal' : 'vertical');
    divider.setAttribute('aria-label', 'Resize panes');
    divider.setAttribute('aria-valuemin', '0');
    divider.tabIndex = 0;

    if (node.left) {
      pane1.appendChild(this.#reconcile(node.left, prev));
    }
    if (node.right) {
      pane2.appendChild(this.#reconcile(node.right, prev));
    }

    // Size pane1: a dragged basis wins; else the set location; else an even split.
    const dragged = el.dataset.splitBasis;
    const loc = node.divider ?? -1;
    if (dragged !== undefined) {
      pane1.style.flex = `0 0 ${dragged}px`;
      pane2.style.flex = '1 1 0';
      divider.setAttribute('aria-valuenow', dragged);
    } else if (loc > 0) {
      pane1.style.flex = `0 0 ${String(loc)}px`;
      pane2.style.flex = '1 1 0';
      divider.setAttribute('aria-valuenow', String(loc));
    } else {
      pane1.style.flex = '1 1 0';
      pane2.style.flex = '1 1 0';
    }

    this.#splitDrag(el, pane1, pane2, divider, vertical);
    el.replaceChildren(pane1, divider, pane2);
  }

  /** Wire the divider: drag or arrow-key resize of the first pane, clamped to
   * the container and recorded so it survives re-renders. */
  #splitDrag(
    el: HTMLElement,
    pane1: HTMLElement,
    pane2: HTMLElement,
    divider: HTMLElement,
    vertical: boolean,
  ): void {
    const setSize = (size: number): void => {
      const rect = el.getBoundingClientRect();
      const max = (vertical ? rect.height : rect.width) - 8;
      const clamped = Math.round(Math.max(20, Math.min(size, Math.max(20, max))));
      pane1.style.flex = `0 0 ${String(clamped)}px`;
      pane2.style.flex = '1 1 0';
      el.dataset.splitBasis = String(clamped);
      divider.setAttribute('aria-valuenow', String(clamped));
    };
    divider.addEventListener('pointerdown', (event) => {
      event.preventDefault();
      divider.setPointerCapture(event.pointerId);
      const rect = el.getBoundingClientRect();
      const onMove = (ev: PointerEvent): void => {
        setSize(vertical ? ev.clientY - rect.top : ev.clientX - rect.left);
      };
      const onUp = (): void => {
        divider.removeEventListener('pointermove', onMove);
        divider.removeEventListener('pointerup', onUp);
      };
      divider.addEventListener('pointermove', onMove);
      divider.addEventListener('pointerup', onUp);
    });
    divider.addEventListener('keydown', (event) => {
      const dec = vertical ? 'ArrowUp' : 'ArrowLeft';
      const inc = vertical ? 'ArrowDown' : 'ArrowRight';
      if (event.key !== dec && event.key !== inc) {
        return;
      }
      event.preventDefault();
      const cur = pane1.getBoundingClientRect();
      const size = (vertical ? cur.height : cur.width) + (event.key === inc ? 10 : -10);
      setSize(size);
    });
  }

  /** A JToolBar: an ARIA toolbar strip of controls with roving-focus arrow
   * navigation (one Tab stop; arrows move between items). */
  private toolBar(node: SwingNode): HTMLElement {
    const el = document.createElement('div');
    el.className = 'swing-toolbar';
    el.setAttribute('role', 'toolbar');
    const vertical = node.orientation === 1; // VERTICAL
    el.classList.toggle('swing-toolbar-vertical', vertical);
    el.setAttribute('aria-orientation', vertical ? 'vertical' : 'horizontal');
    if (node.name !== undefined && node.name !== null) {
      el.setAttribute('aria-label', node.name);
    }
    this.common(el, node);
    this.children(el, node);
    // Arrow keys move focus among the toolbar's controls (roving tabindex).
    el.addEventListener('keydown', (event) => {
      const items = this.#toolbarItems(el);
      const current = items.indexOf(document.activeElement as HTMLElement);
      if (current < 0) {
        return;
      }
      const next = vertical ? 'ArrowDown' : 'ArrowRight';
      const prevKey = vertical ? 'ArrowUp' : 'ArrowLeft';
      let target: number;
      if (event.key === next) {
        target = (current + 1) % items.length;
      } else if (event.key === prevKey) {
        target = (current - 1 + items.length) % items.length;
      } else if (event.key === 'Home') {
        target = 0;
      } else if (event.key === 'End') {
        target = items.length - 1;
      } else {
        return;
      }
      event.preventDefault();
      items[current]?.setAttribute('tabindex', '-1');
      const el2 = items[target];
      if (el2) {
        el2.tabIndex = 0;
        el2.focus();
      }
    });
    this.#toolbarRoving(el);
    return el;
  }

  /** The focusable controls of a toolbar, in order. */
  #toolbarItems(el: HTMLElement): HTMLElement[] {
    return [...el.querySelectorAll<HTMLElement>('button, input, select, a[href]')].filter(
      (item) => !(item as HTMLButtonElement).disabled,
    );
  }

  /** Roving tabindex: exactly one toolbar item is in the Tab order (the focused
   * one, else the first). Re-applied on build and after each reconcile. */
  #toolbarRoving(el: HTMLElement): void {
    const items = this.#toolbarItems(el);
    const active = items.find((item) => item === document.activeElement) ?? items[0];
    for (const item of items) {
      item.tabIndex = item === active ? 0 : -1;
    }
  }

  private toolBarSeparator(node: SwingNode): HTMLElement {
    const sep = document.createElement('div');
    sep.className = 'swing-toolbar-sep';
    sep.setAttribute('aria-hidden', 'true');
    this.common(sep, node);
    return sep;
  }

  /** A custom-painted JPanel: a canvas replaying its Graphics commands. */
  private canvasPanel(node: SwingNode): HTMLElement {
    const wrap = document.createElement('div');
    wrap.className = 'swing-panel';
    const canvas = document.createElement('canvas');
    canvas.className = 'swing-canvas';
    canvas.width = node.pw ?? 200;
    canvas.height = node.ph ?? 150;
    // A bitmap drawing is opaque to assistive tech; expose it as a named
    // image so a screen reader announces it (setToolTipText names it).
    canvas.setAttribute('role', 'img');
    canvas.setAttribute('aria-label', node.tooltip ?? 'Drawing');
    paintCanvas(canvas, node.paint ?? '');
    this.#mouse(canvas, node);
    this.#motion(canvas, node);
    this.#keys(canvas, node);
    wrap.appendChild(canvas);
    // A painted panel rarely also holds child widgets, but honor them.
    this.children(wrap, node);
    this.common(wrap, node);
    return wrap;
  }

  private label(node: SwingNode): HTMLElement {
    // With setLabelFor this is a real <label for=…>, giving the target field
    // an accessible name and making the label text a click target for it.
    if (node.for !== undefined) {
      const label = document.createElement('label');
      label.className = 'swing-label';
      label.htmlFor = node.for;
      // setDisplayedMnemonic: underline the letter; an accessKey on a <label>
      // moves focus to its associated control on Alt+letter (native).
      setMnemonicLabel(label, node.text ?? '', node.mnemonic);
      label.accessKey = node.mnemonic ?? '';
      this.common(label, node);
      return label;
    }
    // A standalone label is the classic status/output widget (captions use
    // setLabelFor → the <label> above). Make it a polite live region so a
    // screen reader announces changes to its text — the reused element is
    // patched in place, so only labels that actually change ever speak, and
    // "polite" coalesces rapid updates rather than reading every intermediate.
    const span = document.createElement('span');
    span.className = 'swing-label';
    span.setAttribute('aria-live', 'polite');
    span.setAttribute('aria-atomic', 'true');
    span.textContent = node.text ?? '';
    this.common(span, node);
    return span;
  }

  private button(node: SwingNode): HTMLElement {
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'swing-button';
    button.id = node.id;
    button.disabled = node.enabled === false;
    // setMnemonic → the underlined letter + an accessKey (the platform's
    // Alt-shortcut) + aria-keyshortcuts so it's announced.
    setMnemonicLabel(button, node.text ?? '', node.mnemonic);
    button.accessKey = node.mnemonic ?? '';
    // A click reports the button's id plus every field's current value to
    // the program's ActionListener. (Native <button> also fires on
    // Enter/Space, so keyboard works.) Only wired when it has a listener.
    if (node.listens === true) {
      const id = node.id;
      button.addEventListener('click', () => {
        this.#dispatch(this.#payload(id));
      });
    }
    this.common(button, node);
    return button;
  }

  /** A JToggleButton: a two-state button (aria-pressed). Clicking flips its
   * pressed state and reports it, firing the Action/Item listeners. */
  private toggleButton(node: SwingNode): HTMLElement {
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'swing-button swing-toggle';
    button.id = node.id;
    button.textContent = node.text ?? '';
    button.disabled = node.enabled === false;
    button.setAttribute('aria-pressed', String(node.selected === true));
    if (node.listens === true) {
      const id = node.id;
      button.addEventListener('click', () => {
        const pressed = button.getAttribute('aria-pressed') !== 'true';
        button.setAttribute('aria-pressed', String(pressed));
        this.#dispatch(`${this.#payload(id)}\n${id}=${String(pressed)}`);
      });
    }
    this.common(button, node);
    return button;
  }

  private textField(node: SwingNode): HTMLElement {
    const input = document.createElement('input');
    input.type = node.password === true ? 'password' : 'text';
    input.className = 'swing-textfield';
    input.id = node.id;
    input.value = node.text ?? '';
    input.disabled = node.enabled === false;
    input.readOnly = node.editable === false;
    applyCaretRequest(input, node);
    if (node.columns !== undefined && node.columns > 0) {
      input.size = node.columns;
    }
    // Fall back to the tooltip for an accessible name when no label targets it.
    if (node.tooltip !== undefined) {
      input.setAttribute('aria-label', node.tooltip);
    }
    this.#fields.set(node.id, input);
    // A JTextField with an ActionListener fires on Enter (Swing-accurate, not
    // on blur); its typed value rides along in the payload.
    if (node.listens === true) {
      const id = node.id;
      input.addEventListener('keydown', (event) => {
        if (event.key === 'Enter') {
          event.preventDefault();
          this.#dispatch(this.#payload(id));
        }
      });
    }
    this.#documentInput(input, node);
    this.common(input, node);
    return input;
  }

  private textArea(node: SwingNode): HTMLElement {
    const textarea = document.createElement('textarea');
    textarea.className = 'swing-textarea';
    textarea.id = node.id;
    textarea.value = node.text ?? '';
    textarea.disabled = node.enabled === false;
    textarea.readOnly = node.editable === false;
    applyCaretRequest(textarea, node);
    if (node.rows !== undefined && node.rows > 0) {
      textarea.rows = node.rows;
    }
    if (node.columns !== undefined && node.columns > 0) {
      textarea.cols = node.columns;
    }
    // Soft-wrap follows setLineWrap; off means a horizontally scrolling area.
    textarea.wrap = node.wrap === true ? 'soft' : 'off';
    if (node.tooltip !== undefined) {
      textarea.setAttribute('aria-label', node.tooltip);
    }
    this.#fields.set(node.id, textarea);
    this.#documentInput(textarea, node);
    this.common(textarea, node);
    return textarea;
  }

  private checkBox(node: SwingNode): HTMLElement {
    // A wrapping <label> gives the checkbox its accessible name and makes the
    // whole row (box + text) a single click/tap target.
    const label = document.createElement('label');
    label.className = 'swing-checkbox';
    const input = document.createElement('input');
    input.type = 'checkbox';
    input.id = node.id;
    input.checked = node.selected === true;
    input.disabled = node.enabled === false;
    const text = document.createElement('span');
    text.textContent = node.text ?? '';
    label.append(input, text);
    // Toggling fires the ItemListener/ActionListener (when present).
    this.#field(input, node, 'change');
    this.common(label, node);
    return label;
  }

  private radioButton(node: SwingNode): HTMLElement {
    const label = document.createElement('label');
    label.className = 'swing-radio';
    const input = document.createElement('input');
    input.type = 'radio';
    input.id = node.id;
    // A shared ButtonGroup name lets the browser enforce single-selection.
    if (node.group !== undefined) {
      input.name = node.group;
    }
    input.checked = node.selected === true;
    input.disabled = node.enabled === false;
    const text = document.createElement('span');
    text.textContent = node.text ?? '';
    label.append(input, text);
    this.#field(input, node, 'change');
    this.common(label, node);
    return label;
  }

  private comboBox(node: SwingNode): HTMLElement {
    if (node.editable === true) {
      return this.editableComboBox(node);
    }
    const select = document.createElement('select');
    select.className = 'swing-combobox';
    select.id = node.id;
    select.disabled = node.enabled === false;
    for (const [index, item] of (node.items ?? []).entries()) {
      const option = document.createElement('option');
      option.value = String(index);
      option.textContent = item;
      option.selected = index === node.selectedIndex;
      select.appendChild(option);
    }
    // A <select> has no implicit label; fall back to the tooltip when no
    // JLabel targets it (setLabelFor still gives the strongest name).
    if (node.tooltip !== undefined) {
      select.setAttribute('aria-label', node.tooltip);
    }
    this.#field(select, node, 'change');
    this.common(select, node);
    return select;
  }

  /** An editable JComboBox: a text input + a drop arrow that toggles a listbox
   * of all the items (a real JComboBox-style dropdown, not native autocomplete).
   * Type a custom value or pick one; the value is the text. The listbox is a
   * top-layer popover so it isn't clipped by the window. */
  private editableComboBox(node: SwingNode): HTMLElement {
    const wrap = document.createElement('span');
    wrap.className = 'swing-combobox-wrap';

    const input = document.createElement('input');
    input.type = 'text';
    input.className = 'swing-combobox-input';
    input.setAttribute('role', 'combobox');
    input.setAttribute('aria-autocomplete', 'list');
    input.setAttribute('aria-expanded', 'false');
    input.value = node.text ?? '';
    input.disabled = node.enabled === false;
    const listId = `${node.id}-listbox`;
    input.setAttribute('aria-controls', listId);
    if (node.tooltip !== undefined) {
      input.setAttribute('aria-label', node.tooltip);
    }

    const arrow = document.createElement('button');
    arrow.type = 'button';
    arrow.className = 'swing-combobox-arrow';
    arrow.tabIndex = -1;
    arrow.setAttribute('aria-label', 'Show options');
    arrow.textContent = '▾';

    const list = document.createElement('ul');
    list.className = 'swing-combobox-list';
    list.id = listId;
    list.setAttribute('role', 'listbox');
    list.popover = 'manual';

    this.#buildComboOptions(list, input, node);

    // The arrow keeps the input focused and toggles the list.
    arrow.addEventListener('mousedown', (event) => {
      event.preventDefault();
    });
    arrow.addEventListener('click', () => {
      if (list.matches(':popover-open')) {
        this.#closeComboList(list, input);
      } else {
        this.#openComboList(list, input, wrap);
        input.focus();
      }
    });

    if (node.listens === true) {
      const id = node.id;
      input.addEventListener('change', () => {
        this.#dispatch(this.#payload(id));
      });
    }
    input.addEventListener('keydown', (event) => {
      const options = [...list.querySelectorAll<HTMLElement>('[role="option"]')];
      const current = options.findIndex((o) => o.classList.contains('swing-combobox-active'));
      if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
        event.preventDefault();
        if (!list.matches(':popover-open')) {
          this.#openComboList(list, input, wrap);
        }
        const next =
          event.key === 'ArrowDown'
            ? Math.min(current + 1, options.length - 1)
            : Math.max(current - 1, 0);
        this.#highlightComboOption(input, options, next < 0 ? 0 : next);
      } else if (event.key === 'Enter') {
        if (list.matches(':popover-open') && current >= 0) {
          event.preventDefault();
          input.value = options[current]?.textContent ?? input.value;
          this.#closeComboList(list, input);
        }
        // Enter also commits (the native change fires); dispatch happens there.
      } else if (event.key === 'Escape' && list.matches(':popover-open')) {
        event.preventDefault();
        this.#closeComboList(list, input);
      }
    });

    // Click outside the combo closes the list.
    const onOutside = (event: Event): void => {
      if (!wrap.contains(event.target as Node)) {
        this.#closeComboList(list, input);
      }
    };
    document.addEventListener('click', onOutside);
    this.#teardown.push(() => {
      document.removeEventListener('click', onOutside);
    });

    this.#fields.set(node.id, input); // the input carries the value
    wrap.append(input, arrow, list);
    this.common(wrap, node);
    return wrap;
  }

  #patchEditableCombo(wrap: HTMLElement, node: SwingNode): void {
    const input = wrap.querySelector<HTMLInputElement>('.swing-combobox-input');
    const list = wrap.querySelector<HTMLElement>('.swing-combobox-list');
    if (input) {
      if (input.value !== (node.text ?? '')) {
        input.value = node.text ?? '';
      }
      input.disabled = node.enabled === false;
      this.#fields.set(node.id, input);
      if (list) {
        this.#buildComboOptions(list, input, node);
      }
    }
  }

  /** (Re)build the listbox's options and wire each to fill the input on click. */
  #buildComboOptions(list: HTMLElement, input: HTMLInputElement, node: SwingNode): void {
    list.replaceChildren(
      ...(node.items ?? []).map((item, i) => {
        const option = document.createElement('li');
        option.className = 'swing-combobox-option';
        option.id = `${node.id}-opt${String(i)}`;
        option.setAttribute('role', 'option');
        option.setAttribute('aria-selected', String(item === input.value));
        option.textContent = item;
        option.addEventListener('mousedown', (event) => {
          event.preventDefault(); // don't blur the input before the click lands
        });
        option.addEventListener('click', () => {
          input.value = item;
          this.#closeComboList(list, input);
          input.focus();
          if (node.listens === true) {
            this.#dispatch(this.#payload(node.id));
          }
        });
        return option;
      }),
    );
  }

  #openComboList(list: HTMLElement, input: HTMLElement, wrap: HTMLElement): void {
    if (!list.matches(':popover-open')) {
      list.showPopover();
    }
    input.setAttribute('aria-expanded', 'true');
    const rect = wrap.getBoundingClientRect();
    list.style.left = `${String(rect.left)}px`;
    list.style.minWidth = `${String(rect.width)}px`;
    // Drop below, or flip above if it would overflow the viewport.
    const below = rect.bottom;
    list.style.top = `${String(below)}px`;
    if (below + list.offsetHeight > window.innerHeight) {
      list.style.top = `${String(Math.max(4, rect.top - list.offsetHeight))}px`;
    }
  }

  #closeComboList(list: HTMLElement, input: HTMLElement): void {
    if (list.matches(':popover-open')) {
      list.hidePopover();
    }
    input.setAttribute('aria-expanded', 'false');
    input.removeAttribute('aria-activedescendant');
  }

  #highlightComboOption(input: HTMLElement, options: HTMLElement[], index: number): void {
    options.forEach((option, i) => {
      option.classList.toggle('swing-combobox-active', i === index);
    });
    const active = options[index];
    if (active) {
      input.setAttribute('aria-activedescendant', active.id);
      active.scrollIntoView({ block: 'nearest' });
    }
  }

  /** A JList: a sized <select> renders as a native, accessible list box
   * (role listbox; arrow keys, Home/End, and typeahead come for free).
   * `multiple` makes it a multi-select list box. */
  private listBox(node: SwingNode): HTMLElement {
    const select = document.createElement('select');
    select.className = 'swing-list';
    select.id = node.id;
    select.size = Math.max(node.rows ?? 8, 2); // size > 1 => list box, not dropdown
    select.multiple = node.multiple === true;
    select.disabled = node.enabled === false;
    const selected = new Set(node.selectedIndices ?? []);
    for (const [index, item] of (node.items ?? []).entries()) {
      const option = document.createElement('option');
      option.value = String(index);
      option.textContent = item;
      option.selected = selected.has(index);
      select.appendChild(option);
    }
    this.#applyItemStyles(select, node);
    if (node.tooltip !== undefined) {
      select.setAttribute('aria-label', node.tooltip);
    }
    this.#field(select, node, 'change');
    this.common(select, node);
    return select;
  }

  /** Paint each option with the colours its ListCellRenderer chose. Re-applied
   * on every render: a renderer can recolour a row without its text changing. */
  #applyItemStyles(select: HTMLSelectElement, node: SwingNode): void {
    const styles = node.itemStyles;
    for (let i = 0; i < select.options.length; i++) {
      const option = select.options[i];
      if (!option) continue;
      const style = styles?.[i];
      option.style.color = cssColor(style?.fg) ?? '';
      option.style.backgroundColor = cssColor(style?.bg) ?? '';
    }
  }

  /**
   * A JTable as a semantic <table> (headers announced with their cells). When
   * it has a selection listener it becomes an ARIA grid: rows are focusable,
   * the arrow keys move focus, and Enter/Space or a click selects the row and
   * dispatches (the highlight follows the model's selection on re-render).
   */
  private table(node: SwingNode): HTMLElement {
    const table = document.createElement('table');
    table.className = 'swing-table';
    const headers = node.headers ?? [];
    const cells = node.cells ?? [];
    const selectedRow = node.selectedRow ?? -1;
    const selectable = this.#onEvent !== null && node.listens === true;
    const editable = this.#onEvent !== null && node.editable !== false;
    if (selectable || editable) {
      table.setAttribute('role', 'grid');
    }
    if (node.tooltip !== undefined) {
      table.setAttribute('aria-label', node.tooltip);
    }

    const thead = document.createElement('thead');
    const headRow = document.createElement('tr');
    for (const name of headers) {
      const th = document.createElement('th');
      th.scope = 'col';
      th.textContent = name;
      headRow.appendChild(th);
    }
    thead.appendChild(headRow);
    table.appendChild(thead);

    const tbody = document.createElement('tbody');
    for (const [r, row] of cells.entries()) {
      const tr = document.createElement('tr');
      for (const [c, value] of row.entries()) {
        const td = document.createElement('td');
        td.textContent = value;
        // Colours and alignment chosen by this column's TableCellRenderer.
        const style = node.cellStyles?.[r]?.[c];
        if (style !== undefined) {
          td.style.color = cssColor(style.fg) ?? '';
          td.style.backgroundColor = cssColor(style.bg) ?? '';
          td.style.textAlign = style.halign === undefined ? '' : horizAlign(style.halign);
        }
        if (selectable || editable) {
          td.setAttribute('role', 'gridcell');
        }
        if (editable) {
          td.classList.add('swing-cell-editable');
          td.title = 'Double-click to edit';
          td.addEventListener('dblclick', () => {
            this.#beginCellEdit(td, node.id, r, c);
          });
        }
        tr.appendChild(td);
      }
      if (selectable) {
        tr.id = `${node.id}-r${String(r)}`;
        const isSelected = r === selectedRow;
        tr.setAttribute('aria-selected', String(isSelected));
        tr.classList.toggle('swing-row-selected', isSelected);
        // Roving tabindex: only the selected row (or the first) is tabbable.
        tr.tabIndex = isSelected || (selectedRow < 0 && r === 0) ? 0 : -1;
        const select = (): void => {
          // Selecting reports the row index alongside the usual field state.
          this.#dispatch(`${this.#payload(node.id)}\n${node.id}=${String(r)}`);
        };
        tr.addEventListener('click', select);
        tr.addEventListener('keydown', (event) => {
          if (event.key === 'Enter' || event.key === ' ') {
            event.preventDefault();
            select();
          }
        });
      }
      tbody.appendChild(tr);
    }
    table.appendChild(tbody);

    if (selectable) {
      // Arrow keys move focus between rows (client-side); Enter/Space selects.
      tbody.addEventListener('keydown', (event) => {
        const rows = [...tbody.querySelectorAll<HTMLElement>('tr')];
        const current = rows.indexOf(document.activeElement as HTMLElement);
        if (current < 0) {
          return;
        }
        let next: number;
        if (event.key === 'ArrowDown') {
          next = Math.min(current + 1, rows.length - 1);
        } else if (event.key === 'ArrowUp') {
          next = Math.max(current - 1, 0);
        } else if (event.key === 'Home') {
          next = 0;
        } else if (event.key === 'End') {
          next = rows.length - 1;
        } else {
          return;
        }
        event.preventDefault();
        rows[current]?.setAttribute('tabindex', '-1');
        const target = rows[next];
        if (target) {
          target.tabIndex = 0;
          target.focus();
        }
      });
    }

    this.common(table, node);
    return table;
  }

  /** Inline cell editor: replace a cell with a text input; Enter/blur commits
   * the value to the model (via an `edit:` payload), Escape cancels. */
  #beginCellEdit(td: HTMLTableCellElement, tableId: string, row: number, col: number): void {
    const original = td.textContent;
    const input = document.createElement('input');
    input.type = 'text';
    input.className = 'swing-cell-editor';
    input.value = original;
    input.setAttribute('aria-label', 'Cell value');
    td.replaceChildren(input);
    input.focus();
    input.select();
    let settled = false;
    const commit = (): void => {
      if (settled) {
        return;
      }
      settled = true;
      const value = input.value;
      td.textContent = value; // optimistic; the re-render confirms it
      const encoded = value.replace(/%/g, '%25').replace(/\n/g, '%0A');
      // The "__edit" sentinel means no selection listener fires — only setValueAt.
      this.#dispatch(
        `${this.#payload('__edit')}\n${tableId}=edit:${String(row)},${String(col)},${encoded}`,
      );
    };
    input.addEventListener('keydown', (event) => {
      if (event.key === 'Enter') {
        event.preventDefault();
        commit();
      } else if (event.key === 'Escape') {
        event.preventDefault();
        settled = true;
        td.textContent = original;
      }
    });
    input.addEventListener('blur', () => {
      commit();
    });
  }

  private slider(node: SwingNode): HTMLElement {
    const input = document.createElement('input');
    input.type = 'range';
    input.className = 'swing-slider';
    input.id = node.id;
    input.min = String(node.min ?? 0);
    input.max = String(node.max ?? 100);
    input.value = String(node.value ?? 0);
    input.disabled = node.enabled === false;
    // Native range inputs expose value/min/max to assistive tech; add a name
    // from the tooltip when no JLabel targets it.
    if (node.tooltip !== undefined) {
      input.setAttribute('aria-label', node.tooltip);
    }
    this.#field(input, node, 'change');
    this.common(input, node);
    return input;
  }

  /** A JProgressBar: an accessible progress bar (role progressbar with aria
   * value min/max/now), or indeterminate when there's no known value. */
  private progressBar(node: SwingNode): HTMLElement {
    const bar = document.createElement('div');
    bar.className = 'swing-progress';
    bar.setAttribute('role', 'progressbar');
    const fill = document.createElement('div');
    fill.className = 'swing-progress-fill';
    const text = document.createElement('span');
    text.className = 'swing-progress-text';
    bar.append(fill, text);
    if (node.tooltip !== undefined) {
      bar.setAttribute('aria-label', node.tooltip);
    }
    this.#progressState(bar, node);
    this.common(bar, node);
    return bar;
  }

  /** Apply a progress bar's value/indeterminate state (shared build + patch). */
  #progressState(bar: HTMLElement, node: SwingNode): void {
    const min = node.min ?? 0;
    const max = node.max ?? 100;
    const value = node.value ?? 0;
    const fill = bar.querySelector<HTMLElement>('.swing-progress-fill');
    const text = bar.querySelector<HTMLElement>('.swing-progress-text');
    bar.setAttribute('aria-valuemin', String(min));
    bar.setAttribute('aria-valuemax', String(max));
    if (node.indeterminate === true) {
      bar.classList.add('swing-progress-indeterminate');
      bar.removeAttribute('aria-valuenow');
      bar.removeAttribute('aria-valuetext');
      if (fill) fill.style.width = '100%';
      if (text) text.textContent = '';
      return;
    }
    bar.classList.remove('swing-progress-indeterminate');
    bar.setAttribute('aria-valuenow', String(value));
    const span = max - min;
    const pct = span <= 0 ? 0 : ((value - min) * 100) / span;
    if (fill) fill.style.width = `${String(pct)}%`;
    const showText = node.stringPainted === true;
    if (text) text.textContent = showText ? (node.string ?? '') : '';
    if (showText && node.string !== undefined) {
      bar.setAttribute('aria-valuetext', node.string);
    } else {
      bar.removeAttribute('aria-valuetext');
    }
  }

  /** A JSpinner: a native number input (role spinbutton; arrow keys step). */
  private spinner(node: SwingNode): HTMLElement {
    const input = document.createElement('input');
    input.type = 'number';
    input.className = 'swing-spinner';
    input.id = node.id;
    input.min = String(node.min ?? 0);
    input.max = String(node.max ?? 100);
    input.step = String(node.step ?? 1);
    input.value = String(node.value ?? 0);
    input.disabled = node.enabled === false;
    if (node.tooltip !== undefined) {
      input.setAttribute('aria-label', node.tooltip);
    }
    this.#field(input, node, 'change');
    this.common(input, node);
    return input;
  }

  /** A Box strut or glue: an invisible spacer in a BoxLayout. A strut holds a
   * fixed size; glue stretches to push its neighbors apart. */
  private strut(node: SwingNode): HTMLElement {
    const div = document.createElement('div');
    div.className = 'swing-strut';
    div.setAttribute('aria-hidden', 'true');
    if (node.glue === true) {
      div.style.flex = '1 1 auto';
    } else {
      div.style.flexShrink = '0';
      if (node.w !== undefined && node.w > 0) {
        div.style.width = `${String(node.w)}px`;
      }
      if (node.h !== undefined && node.h > 0) {
        div.style.height = `${String(node.h)}px`;
      }
    }
    this.common(div, node);
    return div;
  }
}
