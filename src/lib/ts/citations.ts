import { invoke } from "@tauri-apps/api/core";

export type BibliographyFormat = "bibtex" | "hayagriva";

export interface LookupResult {
  key: string;
  title: string;
  format: BibliographyFormat;
  entry: string;
}

export interface AddedEntry {
  key: string;
  title: string;
  file_name: string;
  entry: string;
  created_file: boolean;
}

export interface ZoteroStatus {
  connected: boolean;
  username: string | null;
}

export interface ZoteroLibrary {
  id: string;
  name: string;
}

export interface ZoteroCollection {
  key: string;
  name: string;
  parent_key: string | null;
}

export interface CitationSync {
  id: string;
  target_path: string;
  library_id: string;
  library_name: string;
  collection_key: string | null;
  collection_name: string | null;
  file_name: string;
  item_count: number;
  last_synced_at: string | null;
}

export function formatForFile(fileName: string): BibliographyFormat {
  return fileName.toLowerCase().endsWith(".bib") ? "bibtex" : "hayagriva";
}

export function citationMarkup(key: string, latex: boolean): string {
  return latex ? `\\cite{${key}}` : `@${key}`;
}

export function collectionLabels(collections: ZoteroCollection[]): { key: string; label: string }[] {
  const byKey = new Map(collections.map((collection) => [collection.key, collection]));
  const pathOf = (collection: ZoteroCollection, depth = 0): string => {
    const parent = collection.parent_key ? byKey.get(collection.parent_key) : undefined;
    return parent && depth < 10 ? `${pathOf(parent, depth + 1)} / ${collection.name}` : collection.name;
  };
  return collections
    .map((collection) => ({ key: collection.key, label: pathOf(collection) }))
    .sort((first, second) => first.label.localeCompare(second.label));
}

export const lookupCitation = (identifier: string, format: BibliographyFormat) =>
  invoke<LookupResult>("citation_lookup", { identifier, format });

export const bibliographyFiles = (path: string) => invoke<string[]>("citation_files", { path });

export const addCitation = (path: string, fileName: string, identifier: string) =>
  invoke<AddedEntry>("citation_add", { path, fileName, identifier });

export const zoteroStatus = () => invoke<ZoteroStatus>("zotero_status");
export const connectZotero = (apiKey: string) => invoke<ZoteroStatus>("zotero_connect", { apiKey });
export const disconnectZotero = () => invoke<void>("zotero_disconnect");
export const zoteroLibraries = () => invoke<ZoteroLibrary[]>("zotero_libraries");
export const zoteroCollections = (library: string) => invoke<ZoteroCollection[]>("zotero_collections", { library });

export const citationSyncs = (path: string) => invoke<CitationSync[]>("citation_syncs", { path });

export const createCitationSync = (
  path: string,
  sync: { libraryId: string; libraryName: string; collectionKey: string | null; collectionName: string | null; fileName: string },
) => invoke<CitationSync>("citation_sync_create", { path, ...sync });

export const runCitationSync = (id: string) => invoke<CitationSync>("citation_sync_run", { id });
export const deleteCitationSync = (id: string) => invoke<void>("citation_sync_delete", { id });
