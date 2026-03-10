/**
 * Custom live preview extension for CodeMirror 6.
 * Cursor-position-aware: raw markdown when cursor is "near" an element, rendered otherwise.
 * Phase 1: ATX headings. Phase 2: List markers + task checkboxes.
 */

import {
  Decoration,
  DecorationSet,
  EditorView,
  ViewPlugin,
  ViewUpdate,
  WidgetType,
} from '@codemirror/view';
import { syntaxTree } from '@codemirror/language';

/** Empty widget used to hide syntax (replaces ### or - with nothing) */
class HideWidget extends WidgetType {
  eq() {
    return true;
  }
  toDOM() {
    return document.createElement('span');
  }
}

/** Bullet widget: shows • when marker is hidden */
class BulletWidget extends WidgetType {
  eq() {
    return true;
  }
  toDOM() {
    const span = document.createElement('span');
    span.className = 'cm-live-preview-bullet';
    span.textContent = '• ';
    return span;
  }
}

/** Interactive checkbox widget: replaces [ ] / [x] when not being edited */
class CheckboxWidget extends WidgetType {
  constructor(readonly checked: boolean, readonly from: number, readonly to: number) {
    super();
  }

  eq(other: CheckboxWidget) {
    return other.checked === this.checked && other.from === this.from;
  }

  toDOM() {
    const wrap = document.createElement('span');
    wrap.className = 'cm-live-preview-checkbox';
    wrap.setAttribute('aria-hidden', 'true');
    const input = document.createElement('input');
    input.type = 'checkbox';
    input.checked = this.checked;
    wrap.appendChild(input);
    return wrap;
  }

  ignoreEvent() {
    return false;
  }
}

const hideWidget = new HideWidget();
const bulletWidget = new BulletWidget();

function isAtxHeading(name: string): number | null {
  const m = /^ATXHeading(\d)$/.exec(name);
  return m ? +m[1] : null;
}

function isListItem(name: string): boolean {
  return name === 'ListItem';
}

function cursorInRange(pos: number, from: number, to: number): boolean {
  return pos >= from && pos <= to;
}

function computeDecorations(view: EditorView): DecorationSet {
  const decorations: Decoration.Range[] = [];
  const cursorPos = view.state.selection.main.head;
  const tree = syntaxTree(view.state);

  tree.iterate({
    enter(node) {
      // --- Headings ---
      const level = isAtxHeading(node.name);
      if (level != null) {
        const line = view.state.doc.lineAt(node.from);
        const lineNum = line.number;
        const cursorLine = view.state.doc.lineAt(cursorPos).number;

        if (lineNum === cursorLine) return;

        const text = view.state.doc.sliceString(node.from, node.to);
        const match = text.match(/^(#+)\s*/);
        if (!match) return;

        const prefixLen = match[0].length;
        const hideFrom = node.from;
        const hideTo = node.from + prefixLen;

        if (hideTo > hideFrom) {
          decorations.push(
            Decoration.replace({ widget: hideWidget, inclusive: false }).range(hideFrom, hideTo)
          );
          const contentTo = Math.min(node.to, line.to);
          if (contentTo > hideTo) {
            decorations.push(
              Decoration.mark({
                class: `cm-live-preview-heading cm-live-preview-h${level}`,
              }).range(hideTo, contentTo)
            );
          }
        }
        return;
      }

      // --- List items ---
      if (!isListItem(node.name)) return;

      const line = view.state.doc.lineAt(node.from);
      const lineText = view.state.doc.sliceString(node.from, line.to);
      const lineFrom = node.from;

      // Bullet list: - + *
      const bulletMatch = lineText.match(/^(\s*)([-+*])(\s{1,4}\[[ xX]\])?(\s*)/);
      if (bulletMatch) {
        const [fullMatch, indent, marker, taskPart, spaceAfter] = bulletMatch;
        const markerFrom = lineFrom;
        const markerTo = lineFrom + fullMatch.length;

        const cursorNearMarker = cursorInRange(cursorPos, markerFrom, markerTo);

        if (!cursorNearMarker && fullMatch.length > 0) {
          // Hide bullet marker, show bullet
          const bulletEnd = lineFrom + indent.length + marker.length;
          decorations.push(
            Decoration.replace({ widget: bulletWidget, inclusive: false }).range(
              lineFrom + indent.length,
              bulletEnd
            )
          );

          // Task checkbox: [ ] or [x]
          if (taskPart) {
            const taskFrom = bulletEnd;
            const taskTo = bulletEnd + taskPart.length;
            const cursorNearTask = cursorInRange(cursorPos, taskFrom, taskTo);

            if (!cursorNearTask) {
              const checked = /\[[xX]\]/.test(taskPart);
              decorations.push(
                Decoration.replace({
                  widget: new CheckboxWidget(checked, taskFrom, taskTo),
                  inclusive: false,
                }).range(taskFrom, taskTo)
              );
            }
          }
        }
        return;
      }

      // Ordered list: 1. 2) etc.
      const orderedMatch = lineText.match(/^(\s*)(\d+)([.)])(\s+)/);
      if (orderedMatch) {
        const [fullMatch, indent, num, delim, space] = orderedMatch;
        const markerFrom = lineFrom;
        const markerTo = lineFrom + fullMatch.length;

        const cursorNearMarker = cursorInRange(cursorPos, markerFrom, markerTo);

        if (!cursorNearMarker && fullMatch.length > 0) {
          const markerEnd = lineFrom + indent.length + num.length + delim.length + space.length;
          decorations.push(
            Decoration.replace({ widget: hideWidget, inclusive: false }).range(
              lineFrom + indent.length,
              markerEnd
            )
          );
        }
      }
    },
  });

  return Decoration.set(decorations);
}

function toggleCheckbox(view: EditorView, pos: number): boolean {
  const doc = view.state.doc;
  const line = doc.lineAt(pos);
  const text = line.text;
  const lineStart = line.from;

  const emptyMatch = text.match(/\[ \]/);
  const checkedMatch = text.match(/\[[xX]\]/);

  if (emptyMatch) {
    const idx = text.indexOf('[ ]');
    const from = lineStart + idx;
    const to = from + 3;
    view.dispatch({ changes: { from, to, insert: '[x]' } });
    return true;
  }
  if (checkedMatch) {
    const idx = text.indexOf(checkedMatch[0]);
    const from = lineStart + idx;
    const to = from + checkedMatch[0].length;
    view.dispatch({ changes: { from, to, insert: '[ ]' } });
    return true;
  }
  return false;
}

const livePreviewTheme = EditorView.baseTheme({
  '.cm-live-preview-heading': { fontWeight: 600 },
  '.cm-live-preview-h1': { fontSize: '1.5em' },
  '.cm-live-preview-h2': { fontSize: '1.25em' },
  '.cm-live-preview-h3': { fontSize: '1.1em' },
  '.cm-live-preview-h4': { fontSize: '1em' },
  '.cm-live-preview-h5': { fontSize: '0.9em' },
  '.cm-live-preview-h6': { fontSize: '0.85em', color: '#a1a1aa' },
  '.cm-live-preview-bullet': { color: '#a1a1aa' },
  '.cm-live-preview-checkbox': {
    display: 'inline-flex',
    alignItems: 'center',
    marginRight: '4px',
  },
  '.cm-live-preview-checkbox input': {
    margin: 0,
    cursor: 'pointer',
  },
});

const livePreviewPlugin = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;

    constructor(view: EditorView) {
      this.decorations = computeDecorations(view);
    }

    update(update: ViewUpdate) {
      if (
        update.docChanged ||
        update.selectionSet ||
        update.viewportChanged ||
        syntaxTree(update.startState) !== syntaxTree(update.state)
      ) {
        this.decorations = computeDecorations(update.view);
      }
    }
  },
  {
    decorations: (v) => v.decorations,
    eventHandlers: {
      mousedown: (e: MouseEvent, view: EditorView) => {
        const target = e.target as HTMLElement;
        if (
          target.nodeName === 'INPUT' &&
          target.parentElement?.classList.contains('cm-live-preview-checkbox')
        ) {
          e.preventDefault();
          const pos = view.posAtDOM(target);
          return toggleCheckbox(view, pos);
        }
      },
    },
  }
);

/** Live preview extension: use with markdown(). Only in Edit (inline) mode. */
export const livePreview = [livePreviewTheme, livePreviewPlugin];
