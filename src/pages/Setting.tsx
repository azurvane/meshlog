import { useEffect, useState } from "react";
import { AppSettings } from "../utils/appSettings.ts";
import "../theme/colors.ts";
import "./Setting.css";

interface SettingProps {
  onResetPath: () => void;
  onBack: () => void;
}

export function Setting({ onResetPath, onBack }: SettingProps) {
  // Use string | number to gracefully handle empty inputs while typing
  const [similarityThreshold, setSimilarityThreshold] = useState<
    number | string
  >("");

  // Load stored similarity threshold on component mount
  useEffect(() => {
    const loadStoredThreshold = async () => {
      const storedVal = await AppSettings.get("similarityThreshold");
      setSimilarityThreshold(storedVal ?? "");
    };

    loadStoredThreshold();
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
              type="number"
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

      {/* Side-by-Side Tables Placeholder Container */}
      <section className="tables-container">
        <div className="table-wrapper">
          <h4>Table 1 (Placeholder)</h4>
          <div className="table-placeholder">
            <table>
              <thead>
                <tr>
                  <th>ID</th>
                  <th>Key</th>
                  <th>Value</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td>1</td>
                  <td>Sample Item A</td>
                  <td>Active</td>
                </tr>
                <tr>
                  <td>2</td>
                  <td>Sample Item B</td>
                  <td>Pending</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <div className="table-wrapper">
          <h4>Table 2 (Placeholder)</h4>
          <div className="table-placeholder">
            <table>
              <thead>
                <tr>
                  <th>ID</th>
                  <th>Property</th>
                  <th>Status</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td>101</td>
                  <td>Setting Alpha</td>
                  <td>Enabled</td>
                </tr>
                <tr>
                  <td>102</td>
                  <td>Setting Beta</td>
                  <td>Disabled</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </section>
    </div>
  );
}
