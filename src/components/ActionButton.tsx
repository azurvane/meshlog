import React from "react";
import "./ActionButton.css";

interface ActionButtonProps {
  label?: string;
  icon?: React.ReactNode;
  isActive?: boolean;
  onClick?: () => void;
}

/**
 * A reusable toolbar button component designed for control bars and menus.
 * It renders an optional vector icon alongside an optional text label, and
 * evaluates the 'isActive' boolean to apply custom active-state highlighting styles.
 */
export const ActionButton: React.FC<ActionButtonProps> = ({
  label,
  icon,
  isActive = false,
  onClick,
}) => {
  return (
    <button
      className={`action-btn ${isActive ? "active" : ""}`}
      onClick={onClick}
    >
      {icon && <span className="btn-icon">{icon}</span>}
      {label && <span className="btn-label">{label}</span>}
    </button>
  );
};
