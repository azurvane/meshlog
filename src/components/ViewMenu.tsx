import React from "react";
import { Check } from "lucide-react";
import { FIELD_REGISTRY, FileMetadata } from "../utils/viewFields";
import "./ViewMenu.css";

interface ViewMenuProps {
  visibleFields: Set<keyof FileMetadata>;
  onToggle: (key: keyof FileMetadata) => void;
}

/**
 * Column selector menu dropdown component. Filter-maps the global field registry
 * configuration list to present custom toggleable metadata key row options.
 * Clicking a row raises callbacks to toggle visibility settings in the directory columns.
 */
export const ViewMenu: React.FC<ViewMenuProps> = ({
  visibleFields,
  onToggle,
}) => {
  // Scans the active configuration set to count how many column types are visible, 
  // excluding locked identifiers (such as the default asset file name, which is always shown).
  const visibleToggleableCount = [...visibleFields].filter((key) => {
    const registryField = FIELD_REGISTRY.find((f) => f.key === key);
    return registryField && !registryField.locked;
  }).length;

  return (
    <div className="view-menu">
      <p className="view-menu-section-label">METADATA COLUMNS</p>

      <div className="view-menu-list">
        {FIELD_REGISTRY.filter((field) => !field.locked).map((field) => (
          <div
            key={field.key}
            className="view-menu-row"
            onClick={() => onToggle(field.key)}
          >
            <span>{field.label}</span>
            {visibleFields.has(field.key) && (
              <Check className="view-menu-check" size={16} />
            )}
          </div>
        ))}
      </div>

      <div className="view-menu-footer">
        <span>{visibleToggleableCount} columns shown</span>
      </div>
    </div>
  );
};
