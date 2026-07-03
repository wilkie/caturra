/**
 * The source editor: CodeMirror 6 with Java syntax, a breakpoint
 * gutter, and a paused-line highlight for the debugger.
 *
 * Breakpoints are toggled the traditional way — clicking a line
 * number — and rendered as red dots in a dedicated gutter to the left
 * of the numbers. The playground reads them as 1-based line numbers,
 * matching the engine's `(file, line)` breakpoint keys.
 */
import { java } from '@codemirror/lang-java';
import { RangeSet, StateEffect, StateField } from '@codemirror/state';
import { Decoration, GutterMarker, gutter, lineNumbers } from '@codemirror/view';
import type { DecorationSet } from '@codemirror/view';
import { EditorView, minimalSetup } from 'codemirror';

// ----- Breakpoints -----

const toggleBreakpointEffect = StateEffect.define<{ pos: number; on: boolean }>({
  map: (value, mapping) => ({ pos: mapping.mapPos(value.pos), on: value.on }),
});

const breakpointMarker = new (class extends GutterMarker {
  override toDOM(): Node {
    const dot = document.createElement('span');
    dot.className = 'cm-breakpoint-dot';
    dot.textContent = '●';
    return dot;
  }
})();

/** Reserves gutter width without looking like a breakpoint. */
const spacerMarker = new (class extends GutterMarker {
  override toDOM(): Node {
    const spacer = document.createElement('span');
    spacer.className = 'cm-breakpoint-spacer';
    spacer.textContent = '●';
    return spacer;
  }
})();

const breakpointField = StateField.define<RangeSet<GutterMarker>>({
  create: () => RangeSet.empty,
  update(set, transaction) {
    let next = set.map(transaction.changes);
    for (const effect of transaction.effects) {
      if (effect.is(toggleBreakpointEffect)) {
        if (effect.value.on) {
          next = next.update({ add: [breakpointMarker.range(effect.value.pos)] });
        } else {
          next = next.update({ filter: (from) => from !== effect.value.pos });
        }
      }
    }
    return next;
  },
});

function toggleBreakpoint(view: EditorView, pos: number): void {
  const breakpoints = view.state.field(breakpointField);
  let on = true;
  breakpoints.between(pos, pos, () => {
    on = false;
  });
  view.dispatch({ effects: toggleBreakpointEffect.of({ pos, on }) });
}

/** 1-based line numbers of all current breakpoints. */
export function breakpointLines(view: EditorView): number[] {
  const lines: number[] = [];
  const iter = view.state.field(breakpointField).iter();
  while (iter.value !== null) {
    lines.push(view.state.doc.lineAt(iter.from).number);
    iter.next();
  }
  return lines;
}

/** Toggle the breakpoint on a 1-based line (the e2e test hook). */
export function toggleBreakpointAtLine(view: EditorView, line: number): void {
  if (line >= 1 && line <= view.state.doc.lines) {
    toggleBreakpoint(view, view.state.doc.line(line).from);
  }
}

// ----- Paused-line highlight -----

const setPausedLineEffect = StateEffect.define<number | null>();

const pausedLineDecoration = Decoration.line({ class: 'cm-paused-line' });

const pausedLineField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(decorations, transaction) {
    let next = decorations.map(transaction.changes);
    for (const effect of transaction.effects) {
      if (effect.is(setPausedLineEffect)) {
        if (effect.value === null || effect.value > transaction.state.doc.lines) {
          next = Decoration.none;
        } else {
          const line = transaction.state.doc.line(effect.value);
          next = Decoration.set([pausedLineDecoration.range(line.from)]);
        }
      }
    }
    return next;
  },
  provide: (field) => EditorView.decorations.from(field),
});

/** Highlight (and scroll to) the paused line; `null` clears it. */
export function setPausedLine(view: EditorView, line: number | null): void {
  const effects: StateEffect<unknown>[] = [setPausedLineEffect.of(line)];
  if (line !== null && line >= 1 && line <= view.state.doc.lines) {
    effects.push(EditorView.scrollIntoView(view.state.doc.line(line).from, { y: 'center' }));
  }
  view.dispatch({ effects });
}

// ----- Construction -----

/** Create the editor inside `parent` with the given initial source. */
export function createEditor(parent: HTMLElement, doc: string): EditorView {
  return new EditorView({
    parent,
    doc,
    extensions: [
      breakpointField,
      gutter({
        class: 'cm-breakpoint-gutter',
        markers: (view) => view.state.field(breakpointField),
        initialSpacer: () => spacerMarker,
        domEventHandlers: {
          mousedown(view, line) {
            toggleBreakpoint(view, line.from);
            return true;
          },
        },
      }),
      lineNumbers({
        domEventHandlers: {
          mousedown(view, line) {
            toggleBreakpoint(view, line.from);
            return true;
          },
        },
      }),
      pausedLineField,
      // minimalSetup instead of basicSetup: the latter brings its own
      // line-number gutter, which would duplicate ours.
      minimalSetup,
      java(),
    ],
  });
}

/** The full source text. */
export function getSource(view: EditorView): string {
  return view.state.doc.toString();
}

/** Replace the full source text (clears breakpoints on old positions). */
export function setSource(view: EditorView, text: string): void {
  view.dispatch({
    changes: { from: 0, to: view.state.doc.length, insert: text },
  });
}
