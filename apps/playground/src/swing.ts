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
    | 'textarea'
    | 'checkbox'
    | 'radio'
    | 'combobox'
    | 'list'
    | 'slider'
    | 'scrollpane'
    | 'table'
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
  /** This child's BorderLayout region ("North".."Center"), when its parent
   * uses a BorderLayout. */
  region?: string;
  columns?: number;
  rows?: number;
  editable?: boolean;
  wrap?: boolean;
  selected?: boolean;
  /** JList: the selected indices and whether multiple selection is allowed. */
  selectedIndices?: number[];
  multiple?: boolean;
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
  /** Panel has a MouseListener (wire click) / MouseMotionListener (wire drag). */
  mouse?: boolean;
  drag?: boolean;
  /** A frame's menu bar (setJMenuBar). */
  menubar?: SwingMenuBar;
  /** A JScrollPane's wrapped component and scrollbar policies. */
  view?: SwingNode | null;
  hpolicy?: number;
  vpolicy?: number;
  /** JTable column names, row-major cell text, and the selected row (-1 none). */
  headers?: string[];
  cells?: string[][];
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
interface SwingMenuEntry {
  type: 'menuitem' | 'separator' | 'menu';
  text?: string;
  id?: string;
  listens?: boolean;
  items?: SwingMenuEntry[];
}

type FieldElement = HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement;

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
    const focusedId =
      this.mount.contains(document.activeElement) && document.activeElement instanceof HTMLElement
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
    for (const [i, child] of nodes.entries()) {
      const el = this.#reconcile(child, prev);
      if (border) {
        el.style.gridArea = (child.region ?? 'Center').toLowerCase();
      }
      const current = host.children[i];
      if (current !== el) {
        host.insertBefore(el, current ?? null);
      }
    }
    while (host.children.length > nodes.length) {
      host.lastElementChild?.remove();
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
      case 'label':
        el.textContent = node.text ?? '';
        break;
      case 'button':
        el.textContent = node.text ?? '';
        (el as HTMLButtonElement).disabled = node.enabled === false;
        break;
      case 'textfield':
      case 'textarea': {
        const input = el as HTMLInputElement | HTMLTextAreaElement;
        // Don't clobber the caret/selection when the value is unchanged.
        if (input.value !== (node.text ?? '')) {
          input.value = node.text ?? '';
        }
        input.disabled = node.enabled === false;
        if (input instanceof HTMLTextAreaElement) {
          input.readOnly = node.editable === false;
        }
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
      case 'table':
        return this.table(node);
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
    const border = node.layout === 'border';
    for (const child of node.children ?? []) {
      const el = this.build(child);
      if (border) {
        // Place the child in its BorderLayout region; unconstrained children
        // default to the center, as real BorderLayout does.
        el.style.gridArea = (child.region ?? 'Center').toLowerCase();
      }
      container.appendChild(el);
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
          popup.hidden = false;
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
    popup.hidden = true;

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
            subPopup.hidden = false;
            subPopup.style.top = `${String(subButton.offsetTop)}px`;
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
      item.setAttribute('role', 'menuitem');
      item.tabIndex = -1; // reached via arrow keys, not Tab
      item.textContent = entry.text ?? '';
      // Moving onto a plain item closes any sibling submenu that was open.
      item.addEventListener('mouseenter', () => {
        closeChild();
      });
      if (entry.listens === true && entry.id !== undefined) {
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
      const items = [...popup.querySelectorAll<HTMLElement>(':scope > [role="menuitem"]')];
      const index = items.indexOf(document.activeElement as HTMLElement);
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
    for (const nested of popup.querySelectorAll<HTMLElement>('[role="menu"]')) {
      nested.hidden = true;
    }
    for (const expanded of popup.querySelectorAll<HTMLElement>('[aria-expanded="true"]')) {
      expanded.setAttribute('aria-expanded', 'false');
    }
    popup.hidden = true;
    button.setAttribute('aria-expanded', 'false');
  }

  /** Focus a popup's first menu item (its direct children only). */
  #focusFirst(popup: HTMLElement): void {
    popup.querySelector<HTMLElement>(':scope > [role="menuitem"]')?.focus();
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
    if (node.listens === true) {
      const id = node.id;
      button.addEventListener('click', () => {
        this.#dispatch(this.#payload(id));
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

  private textArea(node: SwingNode): HTMLElement {
    const textarea = document.createElement('textarea');
    textarea.className = 'swing-textarea';
    textarea.id = node.id;
    textarea.value = node.text ?? '';
    textarea.disabled = node.enabled === false;
    textarea.readOnly = node.editable === false;
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
    if (node.tooltip !== undefined) {
      select.setAttribute('aria-label', node.tooltip);
    }
    this.#field(select, node, 'change');
    this.common(select, node);
    return select;
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
    if (selectable) {
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
      for (const value of row) {
        const td = document.createElement('td');
        td.textContent = value;
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
