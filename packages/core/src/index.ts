/**
 * Shared types and utilities for Nisaba.
 * Relationships, blocks, and data model.
 */

export interface Block {
  id: string;
  filePath: string;
  blockIndex: number;
  type: string;
  content: string;
  properties?: Record<string, unknown>;
}

export interface Relation {
  sourceId: string;
  targetId: string;
  type?: string;
}

export interface VaultFile {
  path: string;
  content: string;
  frontmatter: Record<string, unknown>;
  blockIds: string[];
}
