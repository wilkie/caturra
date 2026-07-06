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
  type: 'frame' | 'panel' | 'label' | 'button' | 'textfield' | 'checkbox' | 'component';
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
  children?: SwingNode[];
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
  constructor(private readonly mount: HTMLElement) {}

  /** Remove any previously rendered UI. */
  clear(): void {
    this.mount.replaceChildren();
  }

  /** Parse a `swing.json` tree and render it as accessible DOM. */
  render(json: string): void {
    this.clear();
    const root = JSON.parse(json) as SwingNode;
    this.mount.appendChild(this.build(root));
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
    titlebar.setAttribute('aria-hidden', 'true'); // the region name already conveys it
    titlebar.textContent = node.title ?? '';
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
    const panel = document.createElement('div');
    panel.className = 'swing-panel';
    applyLayout(panel, node.layout);
    this.common(panel, node);
    this.children(panel, node);
    return panel;
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
    this.common(label, node);
    return label;
  }
}
