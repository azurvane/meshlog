export interface GitCommitData {
    name: string,
    path: string;
    tag: string;
    summary: string;
    detail: string;
}

export interface fileDetails {
    name: string;
    path: string;
    isDir: boolean;
}

export interface FileMetadata {
    name: string;
    size_bytes: number;
    modified_ddmmyyyy: string;
    created_ddmmyyyy: string;
    is_dir: boolean;
    file_type: string;
    current_version: string;
    current_hash: string;
}

export interface FieldDef {
    key: keyof FileMetadata;
    label: string;
    locked: boolean;
    minWidth?: string;
    flexWeight?: string;
}

export enum PanelView {
    Repository = "repository",
    LogView = "logViewer",
    Database = "database",
}

export interface TableData {
    columns: string[];
    rows: string[][];
}

interface ViewOption {
    view: PanelView;
    label: string;
    description: string;
}

export const VIEW_REGISTRY: ViewOption[] = [
    { view: PanelView.Repository, label: "Repository", description: "Miller-column asset browser" },
    { view: PanelView.LogView, label: "Log View", description: "Flat markdown log index" },
    { view: PanelView.Database, label: "Database View", description: "SQLite asset registry" },
];

export const FIELD_REGISTRY: FieldDef[] = [
    { key: "name",              label: "Name",     locked: true,  minWidth: "120px", flexWeight: "1.5fr" },
    { key: "current_version",   label: "Version",  locked: false, minWidth: "50px",  flexWeight: "0.8fr" },
    { key: "modified_ddmmyyyy", label: "Modified", locked: false, minWidth: "75px",  flexWeight: "1fr" },
    { key: "size_bytes",        label: "Size",     locked: false, minWidth: "50px",  flexWeight: "0.8fr" },
    { key: "created_ddmmyyyy",  label: "Created",  locked: false, minWidth: "75px",  flexWeight: "1fr" },
    { key: "file_type",         label: "Type",     locked: false, minWidth: "50px",  flexWeight: "0.8fr" },
    { key: "current_hash",      label: "Hash",     locked: false, minWidth: "80px",  flexWeight: "1.2fr" },
];

export const DEFAULT_VISIBLE: Set<keyof FileMetadata> = new Set(
    FIELD_REGISTRY
        .filter(f => f.locked || ["current_version", "modified_ddmmyyyy", "size_bytes"].includes(f.key))
        .map(f => f.key)
);


// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------
 
/**
 * Mirrors the Rust struct `RenameCandidate` returned by `detect_renamed_files`
 * in diff.rs (old_path: String, new_path: String, score: u8).
 */
export interface RenameCandidate {
    old_path: string;
    new_path: string;
    score: number;
  }
   
  /**
   * Where a given table row came from. Kept on the row so the UI (or any
   * future logic) can style/filter rows differently per source without
   * re-deriving it from oldPath/newPath being null.
   */
  export type RenameRowSource = "renamed" | "missing-asset" | "untracked-file";
   
  /**
   * A single normalized row for the table. This is the ONLY shape the UI
   * component needs to know about - it doesn't care which of the 3 functions
   * a row originally came from beyond the `source` tag.
   */
  export interface RenameTableRow {
    /** Stable unique key for React lists + selection tracking. */
    id: string;
    /** null when this row has no old path (came from fetchUnmatchedFiles). */
    oldPath: string | null;
    /** null when this row has no new path (came from fetchMissingAssets). */
    newPath: string | null;
    /** NaN when the source function doesn't provide a score. */
    score: number;
    source: RenameRowSource;
  }