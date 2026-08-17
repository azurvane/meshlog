import { useEffect, useState } from "react";
import { AppSettings } from "../utils/appSettings.ts";
import { RenameTable } from "../components/RenameTable.tsx";
import "../theme/colors.ts";
import "./Setting.css";

interface SettingProps {
  rootPath: string;
  onResetPath: () => void;
  onBack: () => void;
}

export function Setting({ rootPath, onResetPath, onBack }: SettingProps) {
  // Use string | number to gracefully handle empty inputs while typing
  const [similarityThreshold, setSimilarityThreshold] = useState<
    number | string
  >("");

  // Root path of the tracked repo/folder. RenameTable needs this to know
  // where to run `detect_renamed_files` (and the two placeholder lookups).
  // NOTE: adjust the "rootPath" key below if your AppSettings store uses a
  // different key for the tracked folder.

  // Load stored similarity threshold + root path on component mount
  useEffect(() => {
    const loadStoredSettings = async () => {
      const storedThreshold = await AppSettings.get("similarityThreshold");
      setSimilarityThreshold(storedThreshold ?? "");
    };

    loadStoredSettings();
  }, []);

  // Update local display state on keystroke without saving to disk
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setSimilarityThreshold(e.target.value);
  };

  // Persist and clamp value to disk only when user leaves the field
  const handleBlur = async () => {
    const numericValue = Number(similarityThreshold);

    // Default to 0 if NaN/empty, then clamp between 0 and 100
    const clampedValue = Math.min(
      100,
      Math.max(0, isNaN(numericValue) ? 0 : numericValue)
    );

    setSimilarityThreshold(clampedValue);
    await AppSettings.set("similarityThreshold", clampedValue);
  };

  const handleResetSimilarityThreshold = async () => {
    const storedVal = await AppSettings.getDefault("similarityThreshold");
    setSimilarityThreshold(storedVal);
    await AppSettings.reset("similarityThreshold");
  };

  return (
    <div className="settings-container">
      {/* Top Action Bar */}
      <header className="settings-header">
        <button className="btn btn-secondary" onClick={onBack}>
          ← Back
        </button>

        <button className="btn btn-warning" onClick={onResetPath}>
          Reset Path
        </button>
      </header>

      {/* Similarity Threshold Section */}
      <section className="settings-section">
        <h3>Similarity Threshold</h3>
        <div className="threshold-controls">
          <div className="input-wrapper">
            <input
              type="text"
              min="0"
              max="100"
              value={similarityThreshold}
              onChange={handleChange}
              onBlur={handleBlur}
              className="threshold-input"
            />
            <span className="unit-label">%</span>
          </div>

          <button
            className="btn btn-secondary"
            onClick={handleResetSimilarityThreshold}
          >
            Reset Threshold
          </button>
        </div>
      </section>

      <hr className="divider" />

      {/* Renames / unmatched-files table - merges data from
          detect_renamed_files + the two placeholder lookups.
          See renameTableData.ts and RenameTable.tsx for details. */}
      <RenameTable
        rootPath={rootPath}
        threshold={
          typeof similarityThreshold === "number" ? similarityThreshold : 0
        }
        onSelectionChange={(selection) => {
          // Selection = { oldPath, newPath } chosen by the user in the
          // table. Hook your custom logic here (e.g. manually confirming
          // a rename link) whenever either value changes.
          void selection;
        }}
      />
    </div>
  );
}
