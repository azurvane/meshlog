import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RenameCandidate, RenameTableRow } from "../utils/viewFields";
import "./RenameTable.css";

export interface RenameSelection {
  oldPath: string | null;
  newPath: string | null;
}

interface RenameTableProps {
  rootPath: string;
  threshold: number;
  /** Optional: called every time the old/new path selection changes. */
  onSelectionChange?: (selection: RenameSelection) => void;
}

export function RenameTable({
  rootPath,
  threshold,
  onSelectionChange,
}: RenameTableProps) {
  const [rows, setRows] = useState<RenameTableRow[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [selectedOldPath, setSelectedOldPath] = useState<string | null>(null);
  const [selectedNewPath, setSelectedNewPath] = useState<string | null>(null);

  const fetchRenamedPairs = async (
    rootPath: string,
    threshold: number
  ): Promise<RenameCandidate[]> => {
    return invoke<RenameCandidate[]>("detect_renamed_files", {
      rootPath,
      threshold,
    });
  };

  const fetchMissingAssets = async (rootPath: string): Promise<string[]> => {
    return invoke<string[]>("get_missing_path", { rootPath });
  };

  const fetchUnmatchedFiles = async (rootPath: string): Promise<string[]> => {
    return invoke<string[]>("get_existing_uncommited_files", { rootPath });
  };

  const buildRenameTableRows = (
    renamed: RenameCandidate[],
    missingAssets: string[],
    untrackedFiles: string[]
  ): RenameTableRow[] => {
    const rows: RenameTableRow[] = [];

    // Track paths that are already matched by the similarity detection backend
    const usedOldPaths = new Set<string>();
    const usedNewPaths = new Set<string>();

    // 1. Add detected rename pairs (with real scores)
    renamed.forEach((candidate, index) => {
      usedOldPaths.add(candidate.old_path);
      usedNewPaths.add(candidate.new_path);

      rows.push({
        id: `renamed-${index}`,
        oldPath: candidate.old_path,
        newPath: candidate.new_path,
        score: candidate.score,
        source: "renamed",
      });
    });

    // 2. Filter out paths that were already matched in step 1
    const orphanOldPaths = missingAssets.filter(
      (path) => !usedOldPaths.has(path)
    );
    const orphanNewPaths = untrackedFiles.filter(
      (path) => !usedNewPaths.has(path)
    );

    // 3. Pair remaining orphan files side-by-side with score = NaN
    const maxOrphanCount = Math.max(
      orphanOldPaths.length,
      orphanNewPaths.length
    );

    for (let i = 0; i < maxOrphanCount; i++) {
      const oldPath = orphanOldPaths[i] ?? null;
      const newPath = orphanNewPaths[i] ?? null;

      rows.push({
        id: `orphan-${i}`,
        oldPath,
        newPath,
        score: NaN,
        source: "missing-asset",
      });
    }

    return rows;
  };

  // Fetches all 3 sources in parallel and merges them into `rows`.
  const loadRows = useCallback(async () => {
    if (!rootPath) return;

    setLoading(true);
    setError(null);

    try {
      const [renamed, missingAssets, untrackedFiles] = await Promise.all([
        fetchRenamedPairs(rootPath, threshold),
        fetchMissingAssets(rootPath),
        fetchUnmatchedFiles(rootPath),
      ]);

      setRows(buildRenameTableRows(renamed, missingAssets, untrackedFiles));
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, [rootPath, threshold]);

  // Re-fetch whenever the root path or threshold changes.
  useEffect(() => {
    loadRows();
  }, [loadRows]);

  // Notify the parent (if it wants to know) whenever selection changes.
  useEffect(() => {
    onSelectionChange?.({ oldPath: selectedOldPath, newPath: selectedNewPath });
  }, [selectedOldPath, selectedNewPath, onSelectionChange]);

  // Toggle-select an old path cell. Clicking a "—" (null) cell does nothing.
  const handleSelectOld = (path: string | null) => {
    if (path === null) return;
    setSelectedOldPath((prev) => (prev === path ? null : path));
  };

  // Toggle-select a new path cell.
  const handleSelectNew = (path: string | null) => {
    if (path === null) return;
    setSelectedNewPath((prev) => (prev === path ? null : path));
  };

  const changePairs = async () => {
    try {
      if (!selectedOldPath || !selectedNewPath) return;
      await invoke("update_link", {
        rootPath: rootPath,
        oldRelativeFilePath: selectedOldPath,
        newRelativeFilePath: selectedNewPath,
      });
      setSelectedOldPath(null);
      setSelectedNewPath(null);
    } catch (err) {
      console.error("error during relinking the files: ", err);
    }
  };

  const unlinkPath = async () => {
    try {
      if (!selectedNewPath) return;
      await invoke("delete_link", {
        rootPath: rootPath,
        newRelativeFilePath: selectedNewPath,
      });
      setSelectedOldPath(null);
      setSelectedNewPath(null);
    } catch (err) {
      console.error("error unlinking file: ", err);
    }
  };

  const clearSelection = async () => {
    setSelectedOldPath(null);
    setSelectedNewPath(null);
  };

  return (
    <section className="rename-table-wrapper">
      <div className="rename-table-header">
        <h4>Detected Renames &amp; Unmatched Files</h4>
        <button
          className="btn btn-secondary"
          onClick={loadRows}
          disabled={loading}
        >
          {loading ? "Refreshing..." : "Refresh"}
        </button>
      </div>

      {error && <p className="rename-table-error">{error}</p>}

      <div className="table-placeholder">
        <table>
          <thead>
            <tr>
              <th>Old Path</th>
              <th>New Path</th>
              <th>Score</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row) => (
              <tr key={row.id}>
                <td
                  onClick={() => handleSelectOld(row.oldPath)}
                  className={
                    row.oldPath
                      ? row.oldPath === selectedOldPath
                        ? "selectable selected"
                        : "selectable"
                      : undefined
                  }
                  title={row.oldPath ?? undefined}
                >
                  {row.oldPath ?? "—"}
                </td>
                <td
                  onClick={() => handleSelectNew(row.newPath)}
                  className={
                    row.newPath
                      ? row.newPath === selectedNewPath
                        ? "selectable selected"
                        : "selectable"
                      : undefined
                  }
                  title={row.newPath ?? undefined}
                >
                  {row.newPath ?? "—"}
                </td>
                {/* NaN is shown literally, as requested, for rows that
                    came from the two placeholder (single-column) sources. */}
                <td>{Number.isNaN(row.score) ? "NaN" : row.score}</td>
              </tr>
            ))}

            {rows.length === 0 && !loading && (
              <tr>
                <td colSpan={3} className="rename-table-empty">
                  No entries found.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      <div className="rename-table-selection">
        <span>
          Selected old path: <strong>{selectedOldPath ?? "none"}</strong>
        </span>
        <span>
          Selected new path: <strong>{selectedNewPath ?? "none"}</strong>
        </span>
        <button className="btn btn-secondary" onClick={clearSelection}>
          Clear Selection
        </button>
        <button className="btn btn-secondary" onClick={changePairs}>
          Manual Overwrite
        </button>
        <button className="btn btn-secondary" onClick={unlinkPath}>
          unlink asset
        </button>
      </div>
    </section>
  );
}
