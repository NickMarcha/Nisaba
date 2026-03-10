/**
 * Shared types for Nisaba layout. Data layer implementations (desktop vs web) conform to these.
 */

export interface IndexedBlock {
  id: string;
  file_path: string;
  block_index: number;
  block_type: string | null;
  content: string;
}

export interface IndexedLink {
  source_file: string;
  target: string;
  /** Present for frontmatter relations (e.g. "assignee"); null for body wikilinks. */
  relation_key: string | null;
}

export type ViewMode = 'notes' | 'blocks' | 'links';
export type EditorMode = 'raw' | 'edit' | 'split' | 'read';
