/**
 * Wikilink autocomplete for CodeMirror 6.
 * Triggers on [[ and suggests note names from the provided file list.
 */

import { autocompletion } from '@codemirror/autocomplete';

/** Derive wikilink target from file path: remove .md, normalize slashes. */
function pathToTarget(path: string): string {
  return path.replace(/\\/g, '/').replace(/\.md$/i, '').replace(/^\//, '');
}

/** Derive short label from path (filename without .md). */
function pathToLabel(path: string): string {
  const parts = path.replace(/\\/g, '/').split('/');
  const last = parts[parts.length - 1] ?? path;
  return last.replace(/\.md$/i, '');
}

/**
 * Create wikilink autocomplete extension.
 * @param filePaths - Full paths to markdown files (e.g. from vault or server).
 */
export function wikilinkAutocomplete(filePaths: string[]): ReturnType<typeof autocompletion> {
  const targets = filePaths.map(pathToTarget);
  const labels = filePaths.map(pathToLabel);

  return autocompletion({
    override: [
      (context) => {
        const match = context.matchBefore(/\[\[([^\]]*)$/);
        if (!match) return null;
        const query = match.text.slice(2).toLowerCase();
        const options = targets
          .map((target, i) => ({ target, label: labels[i] }))
          .filter(
            (o) =>
              o.target.toLowerCase().includes(query) || o.label.toLowerCase().includes(query)
          )
          .slice(0, 50)
          .map((o) => ({
            label: o.label,
            detail: o.target !== o.label ? o.target : undefined,
            apply: o.target + ']]',
          }));
        if (options.length === 0) return null;
        return {
          from: match.from + 2,
          to: match.to,
          options,
        };
      },
    ],
    activateOnTyping: true,
    maxRenderedOptions: 12,
  });
}
