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