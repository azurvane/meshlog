import React from "react";
import { Check } from "lucide-react";
import { VIEW_REGISTRY, PanelView } from "../utils/viewFields";
import "./SwitchView.css";

interface DropdownProps {
  currentView: PanelView;
  onSelect: (view: PanelView) => void;
}

export const SwitchView: React.FC<DropdownProps> = ({
  currentView,
  onSelect,
}) => {
  return (
    <div className="switch-view">
      <p className="switch-view-section-label">SWITCH VIEW</p>

      <div className="switch-view-list">
        {VIEW_REGISTRY.map((field) => (
          <div
            key={field.label}
            className="switch-view-row"
            onClick={() => onSelect(field.view)}
          >
            <div className="switch-view-row-text">
              <span
                className={
                  field.view === currentView
                    ? "switch-view-row-label switch-view-row-label--active"
                    : "switch-view-row-label"
                }
              >
                {field.label}
              </span>
              <span className="switch-view-row-description">
                {field.description}
              </span>
            </div>
            {field.view === currentView && (
              <Check className="switch-view-check" size={16} />
            )}
          </div>
        ))}
      </div>
    </div>
  );
};
