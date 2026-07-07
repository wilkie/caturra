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

interface SwingNode {
  type:
    | 'frame'
    | 'panel'
    | 'label'
    | 'button'
    | 'textfield'
    | 'checkbox'
    | 'radio'
    | 'combobox'
    | 'slider'
    | 'component';
  id: string;
  enabled?: boolean;
  tooltip?: string;
  bg?: string;
  fg?: string;
  text?: string;
  title?: string;
  width?: number;
  height?: number;
  layout?: string;
  columns?: number;
  selected?: boolean;
  for?: string;
  /** True when the widget has a listener, so its native event dispatches. */
  listens?: boolean;
  /** JComboBox option labels. */
  items?: string[];
  /** JComboBox selected option index. */
  selectedIndex?: number;
  /** JSlider range/value. */
  min?: number;
  max?: number;
  value?: number;
  /** JRadioButton ButtonGroup id (shared DOM `name` for exclusivity). */
  group?: string;
  /** A custom JPanel's recorded Graphics commands (newline-separated). */
  paint?: string;
  /** Canvas dimensions for a painted panel. */
  pw?: number;
  ph?: number;
  children?: SwingNode[];
}

type FieldElement = HTMLInputElement | HTMLSelectElement;

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

/** The value string for a field element, matching the engine's __setFromHost. */
function fieldValue(el: FieldElement): string {
  if (el instanceof HTMLSelectElement) {
    return String(el.selectedIndex); // JComboBox: selected index
  }
  if (el.type === 'checkbox' || el.type === 'radio') {
    return String(el.checked);
  }
  return el.value; // text field / slider
}

/** Replay a custom panel's recorded java.awt.Graphics commands onto a canvas. */
function paintCanvas(canvas: HTMLCanvasElement, paint: string): void {
  const ctx = canvas.getContext('2d');
  if (!ctx) {
    return;
  }
  let color = 'rgb(0,0,0)'; // Graphics current color (fills and strokes)
  const oval = (x: number, y: number, w: number, h: number): void => {
    ctx.beginPath();
    ctx.ellipse(x + w / 2, y + h / 2, w / 2, h / 2, 0, 0, Math.PI * 2);
  };
  for (const command of paint.split('\n')) {
    if (command === '') {
      continue;
    }
    // `drawString "..." x y` keeps the quoted body intact.
    const text = /^drawString "(.*)" (-?\d+) (-?\d+)$/.exec(command);
    if (text) {
      ctx.fillStyle = color;
      ctx.font = '14px system-ui, sans-serif';
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
      default:
        break;
    }
  }
}

/** CSS for a container, mapping the Swing LayoutManager to flex/grid. */
function applyLayout(el: HTMLElement, layout: string | undefined): void {
  if (layout?.startsWith('grid ')) {
    const cols = Number(layout.split(' ')[2] ?? 1);
    el.style.display = 'grid';
    el.style.gridTemplateColumns = `repeat(${String(Math.max(cols, 1))}, auto)`;
    el.style.gap = '8px';
    el.style.alignItems = 'center';
    return;
  }
  // FlowLayout (the default) and BorderLayout both lay out inline in Phase 1.
  el.style.display = 'flex';
  el.style.flexWrap = 'wrap';
  el.style.gap = '8px';
  el.style.alignItems = 'center';
  if (layout === 'border') {
    el.style.flexDirection = 'column';
    el.style.alignItems = 'stretch';
  }
}

export class SwingViz {
  // When set (interactive runs), activating a control calls this with the
  // event payload; when null (Phase 1 batch render) controls are inert.
  #onEvent: ((payload: string) => void) | null = null;
  // Live field controls by id, so any event can report every field's
  // current value back to the program (what the user typed / toggled / chose).
  #fields = new Map<string, FieldElement>();

  constructor(private readonly mount: HTMLElement) {}

  /** Remove any previously rendered UI. */
  clear(): void {
    this.#onEvent = null;
    this.#fields.clear();
    this.mount.replaceChildren();
  }

  /**
   * Parse a `swing.json` tree and render it as accessible DOM. Pass
   * `onEvent` for interactive runs: activating a control (a button) invokes
   * it with the event payload, which the engine's event loop consumes.
   * Focus is preserved across re-renders so keyboard users keep their place.
   */
  render(json: string, onEvent?: (payload: string) => void): void {
    const focusedId = this.mount.contains(document.activeElement)
      ? (document.activeElement as HTMLElement).id
      : '';
    this.#onEvent = onEvent ?? null;
    this.#fields.clear();
    this.mount.replaceChildren();
    const root = JSON.parse(json) as SwingNode;
    this.mount.appendChild(this.build(root));
    if (focusedId !== '') {
      this.mount.querySelector<HTMLElement>(`#${CSS.escape(focusedId)}`)?.focus();
    }
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
    if (this.#onEvent && node.listens === true) {
      const onEvent = this.#onEvent;
      const id = node.id;
      el.addEventListener(event, () => {
        onEvent(this.#payload(id));
      });
    }
  }

  /**
   * Wire a panel with a MouseListener: a click reports component-relative
   * coordinates (mapped to the canvas's own pixel grid when it is scaled).
   * Pointer-only — a bitmap click target has no keyboard equivalent.
   */
  #mouse(el: HTMLElement, node: SwingNode): void {
    if (!this.#onEvent || node.listens !== true) {
      return;
    }
    const onEvent = this.#onEvent;
    const id = node.id;
    if (el instanceof HTMLCanvasElement) {
      el.style.cursor = 'crosshair';
    }
    el.addEventListener('click', (event) => {
      const rect = el.getBoundingClientRect();
      const scaleX = el instanceof HTMLCanvasElement && rect.width ? el.width / rect.width : 1;
      const scaleY = el instanceof HTMLCanvasElement && rect.height ? el.height / rect.height : 1;
      const x = Math.round((event.clientX - rect.left) * scaleX);
      const y = Math.round((event.clientY - rect.top) * scaleY);
      let payload = `${id}\n__mouse=${String(x)},${String(y)}`;
      for (const [fieldId, field] of this.#fields) {
        payload += `\n${fieldId}=${fieldValue(field)}`;
      }
      onEvent(payload);
    });
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
      case 'textfield':
        return this.textField(node);
      case 'checkbox':
        return this.checkBox(node);
      case 'radio':
        return this.radioButton(node);
      case 'combobox':
        return this.comboBox(node);
      case 'slider':
        return this.slider(node);
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
    if (node.tooltip !== undefined) {
      el.title = node.tooltip;
    }
    const bg = cssColor(node.bg);
    if (bg !== null) {
      el.style.backgroundColor = bg;
    }
    const fg = cssColor(node.fg);
    if (fg !== null) {
      el.style.color = fg;
    }
  }

  private children(container: HTMLElement, node: SwingNode): void {
    for (const child of node.children ?? []) {
      container.appendChild(this.build(child));
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
      const onEvent = this.#onEvent;
      const close = document.createElement('button');
      close.type = 'button';
      close.className = 'swing-close';
      close.setAttribute('aria-label', 'Close window');
      close.textContent = '×'; // ×
      close.addEventListener('click', () => {
        onEvent(this.#payload('__close'));
      });
      titlebar.appendChild(close);
    }
    section.appendChild(titlebar);

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

  private panel(node: SwingNode): HTMLElement {
    if (node.paint !== undefined) {
      return this.canvasPanel(node);
    }
    const panel = document.createElement('div');
    panel.className = 'swing-panel';
    applyLayout(panel, node.layout);
    this.#mouse(panel, node);
    this.common(panel, node);
    this.children(panel, node);
    return panel;
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
      label.textContent = node.text ?? '';
      this.common(label, node);
      return label;
    }
    const span = document.createElement('span');
    span.className = 'swing-label';
    span.textContent = node.text ?? '';
    this.common(span, node);
    return span;
  }

  private button(node: SwingNode): HTMLElement {
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'swing-button';
    button.id = node.id;
    button.textContent = node.text ?? '';
    button.disabled = node.enabled === false;
    // A click reports the button's id plus every field's current value to
    // the program's ActionListener. (Native <button> also fires on
    // Enter/Space, so keyboard works.) Only wired when it has a listener.
    if (this.#onEvent && node.listens === true) {
      const onEvent = this.#onEvent;
      const id = node.id;
      button.addEventListener('click', () => {
        onEvent(this.#payload(id));
      });
    }
    this.common(button, node);
    return button;
  }

  private textField(node: SwingNode): HTMLElement {
    const input = document.createElement('input');
    input.type = 'text';
    input.className = 'swing-textfield';
    input.id = node.id;
    input.value = node.text ?? '';
    input.disabled = node.enabled === false;
    if (node.columns !== undefined && node.columns > 0) {
      input.size = node.columns;
    }
    // Fall back to the tooltip for an accessible name when no label targets it.
    if (node.tooltip !== undefined) {
      input.setAttribute('aria-label', node.tooltip);
    }
    this.#fields.set(node.id, input);
    this.common(input, node);
    return input;
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
}
