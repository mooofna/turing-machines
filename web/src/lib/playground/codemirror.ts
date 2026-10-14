// The text editor of the playground: CodeMirror 6 with a small highlighter for the description
// language, the site's colours and fonts, Rust diagnostics as lint marks, and the line of the
// transition about to be applied marked. Loaded with a dynamic import when the Text tab opens.

import { history, defaultKeymap, historyKeymap } from '@codemirror/commands';
import { HighlightStyle, StreamLanguage, syntaxHighlighting } from '@codemirror/language';
import { lintGutter, setDiagnostics, type Diagnostic } from '@codemirror/lint';
import { Annotation, EditorState, StateEffect, StateField } from '@codemirror/state';
import {
  Decoration,
  drawSelection,
  EditorView,
  highlightActiveLineGutter,
  keymap,
  lineNumbers,
  type DecorationSet
} from '@codemirror/view';
import { tags } from '@lezer/highlight';
import type { DiagnosticDto } from '#lib/engine/pkg/turing_wasm.js';

const DECLARATION =
  /^(machine|kind|input|output|tapes|blank|start|Q|F|Σ|Sigma|Γ|Gamma)(?=[\s=]|$)/u;

const language = StreamLanguage.define<{ line: boolean }>({
  name: 'turing',
  startState: () => ({ line: true }),
  token(stream) {
    if (stream.eatSpace()) return null;
    if (stream.match('//')) {
      stream.skipToEnd();
      return 'comment';
    }
    if (stream.match(/^"[^"]*"?/u)) return 'string';
    if (stream.match(/^(δ|delta)(?=\s*\()/u)) return 'keyword';
    if (stream.column() === stream.indentation() && stream.match(DECLARATION)) return 'keyword';
    if (stream.match(/^[(){}=,]/u)) return 'punctuation';
    if (stream.match(/^[^\s(){}=,"]+/u)) return 'variableName';
    stream.next();
    return null;
  },
  languageData: { commentTokens: { line: '//' } }
});

const highlight = HighlightStyle.define([
  { tag: tags.keyword, fontWeight: '500' },
  { tag: tags.comment, color: 'var(--muted)' },
  { tag: tags.punctuation, color: 'var(--muted)' },
  { tag: tags.string, color: 'var(--text)' },
  { tag: tags.variableName, color: 'var(--text)' }
]);

const theme = EditorView.theme({
  '&': {
    color: 'var(--text)',
    backgroundColor: 'var(--field)',
    border: '1px solid var(--rule-strong)',
    fontSize: '0.9375rem'
  },
  '&.cm-focused': { outline: '2px solid var(--accent)', outlineOffset: '-1px' },
  '.cm-scroller': {
    fontFamily: 'var(--font-mono)',
    lineHeight: '1.65',
    minHeight: '22rem',
    maxHeight: '36rem'
  },
  '.cm-content': { caretColor: 'var(--text)', padding: '0.6rem 0' },
  '.cm-line': { padding: '0 0.75rem' },
  '.cm-cursor, .cm-dropCursor': { borderLeft: '2px solid var(--text)' },
  '&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground, .cm-selectionBackground':
    { backgroundColor: 'color-mix(in srgb, var(--accent) 22%, transparent)' },
  '.cm-gutters': {
    backgroundColor: 'var(--field)',
    color: 'var(--muted)',
    border: 'none',
    borderRight: '1px solid var(--rule)'
  },
  '.cm-lineNumbers .cm-gutterElement': { padding: '0 0.5rem 0 0.75rem', minWidth: '2.5rem' },
  '.cm-activeLineGutter': { backgroundColor: 'transparent', color: 'var(--text)' },
  '.cm-next': { backgroundColor: 'color-mix(in srgb, var(--accent) 12%, transparent)' },
  '.cm-lintRange-error, .cm-lintRange-warning': {
    backgroundImage: 'none',
    textDecorationLine: 'underline',
    textDecorationStyle: 'wavy',
    textDecorationThickness: '1px',
    textUnderlineOffset: '3px'
  },
  '.cm-lintRange-error': { textDecorationColor: 'var(--text)' },
  '.cm-lintRange-warning': { textDecorationColor: 'var(--muted)' },
  '.cm-gutter-lint': { width: '1rem' },
  '.cm-gutter-lint .cm-gutterElement': { padding: '0', display: 'grid', placeItems: 'center' },
  '.cm-lint-marker': { width: '0.5rem', height: '0.5rem', content: 'none' },
  '.cm-lint-marker-error': { content: 'none', backgroundColor: 'var(--text)' },
  '.cm-lint-marker-warning': {
    content: 'none',
    backgroundColor: 'transparent',
    border: '1px solid var(--text)'
  },
  '.cm-tooltip': {
    border: '1px solid var(--rule-strong)',
    backgroundColor: 'var(--bg)',
    color: 'var(--text)',
    borderRadius: '0'
  },
  '.cm-tooltip-lint': { padding: '0' },
  '.cm-diagnostic': {
    padding: '0.4rem 0.6rem',
    borderLeft: '2px solid var(--text)',
    fontFamily: 'var(--font-sans)',
    fontSize: '0.875rem'
  },
  '.cm-diagnostic-warning': { borderLeftColor: 'var(--muted)' },
  '.cm-panels': { backgroundColor: 'var(--bg)', color: 'var(--text)' }
});

/** Marks text changes made from outside, which must not be reported back as edits. */
const external = Annotation.define<boolean>();

const setNext = StateEffect.define<number | null>();

const nextLine = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(value, tr) {
    let next = value.map(tr.changes);
    for (const effect of tr.effects) {
      if (effect.is(setNext)) {
        if (effect.value === null || effect.value > tr.state.doc.length) next = Decoration.none;
        else {
          const line = tr.state.doc.lineAt(effect.value);
          next = Decoration.set([Decoration.line({ class: 'cm-next' }).range(line.from)]);
        }
      }
    }
    return next;
  },
  provide: (field) => EditorView.decorations.from(field)
});

export interface SourceView {
  readonly text: string;
  /** Replaces the text, changing only the part that differs. */
  setText(text: string): void;
  setDiagnostics(diagnostics: DiagnosticDto[]): void;
  /** Marks the line that contains UTF-16 offset `at`, or none. */
  setNext(at: number | null): void;
  focus(): void;
  destroy(): void;
}

export function createSourceView(
  parent: HTMLElement,
  text: string,
  onchange: (text: string) => void,
  label: string
): SourceView {
  const view = new EditorView({
    parent,
    state: EditorState.create({
      doc: text,
      extensions: [
        lineNumbers(),
        highlightActiveLineGutter(),
        lintGutter(),
        history(),
        drawSelection(),
        keymap.of([...defaultKeymap, ...historyKeymap]),
        language,
        syntaxHighlighting(highlight),
        theme,
        nextLine,
        EditorView.lineWrapping,
        EditorView.contentAttributes.of({
          'aria-label': label,
          spellcheck: 'false',
          autocapitalize: 'off',
          autocorrect: 'off'
        }),
        EditorView.updateListener.of((update) => {
          if (!update.docChanged) return;
          if (update.transactions.some((tr) => tr.annotation(external))) return;
          onchange(update.state.doc.toString());
        })
      ]
    })
  });

  return {
    get text() {
      return view.state.doc.toString();
    },
    setText(next) {
      const current = view.state.doc.toString();
      if (current === next) return;
      let start = 0;
      while (start < current.length && start < next.length && current[start] === next[start]) {
        start++;
      }
      let end = 0;
      while (
        end < current.length - start &&
        end < next.length - start &&
        current[current.length - 1 - end] === next[next.length - 1 - end]
      ) {
        end++;
      }
      view.dispatch({
        changes: {
          from: start,
          to: current.length - end,
          insert: next.slice(start, next.length - end)
        },
        annotations: external.of(true)
      });
    },
    setDiagnostics(diagnostics) {
      const length = view.state.doc.length;
      const marks: Diagnostic[] = diagnostics.map((d) => {
        const from = Math.min(Math.max(0, d.from ?? 0), length);
        const to = Math.min(Math.max(from, d.to ?? from), length);
        return {
          from,
          to,
          severity: d.severity,
          message: d.help ? `${d.message}. ${d.help}` : d.message,
          source: d.code
        };
      });
      view.dispatch(setDiagnostics(view.state, marks));
    },
    setNext(at) {
      view.dispatch({ effects: setNext.of(at) });
    },
    focus() {
      view.focus();
    },
    destroy() {
      view.destroy();
    }
  };
}
