import React, { useState } from "react";
import { ActionButton } from "./ActionButton";
import "./Header.css";

interface HeaderProps {
  onResetWorkspace?: () => void;
}

export const Header: React.FC<HeaderProps> = ({ onResetWorkspace }) => {
  // Track each panel's visibility independently
  const [panels, setPanels] = useState({
    view: true, // Active by default
    terminal: false,
    inspector: false,
    settings: false,
  });

  const togglePanel = (key: keyof typeof panels) => {
    setPanels((prev) => ({
      ...prev,
      [key]: !prev[key],
    }));
  };

  return (
    <header className="app-header">
      <div className="header-left">
        <div className="logo-badge">P</div>
        <span className="app-title">Palette</span>
        <span className="project-divider">/</span>
        <span className="project-name">nightfall</span>
      </div>

      <div className="header-center">
        <div className="search-container">
          <span className="search-icon">🔍</span>
          <input
            type="text"
            placeholder="Find asset, version, or hash..."
            className="search-input"
          />
          <kbd className="search-shortcut">⌘K</kbd>
        </div>
      </div>

      <div className="header-right">
        <ActionButton
          label="View"
          icon={<span>📊</span>}
          isActive={panels.view}
          onClick={() => togglePanel("view")}
        />
        <ActionButton
          label="Terminal"
          icon={<span>&gt;_</span>}
          isActive={panels.terminal}
          onClick={() => togglePanel("terminal")}
        />
        <ActionButton
          label="Inspector"
          icon={<span>ℹ️</span>}
          isActive={panels.inspector}
          onClick={() => togglePanel("inspector")}
        />
        <ActionButton
          icon={<span>⚙️</span>}
          isActive={panels.settings}
          onClick={() => {
            togglePanel("settings");
            if (
              onResetWorkspace &&
              window.confirm(
                "Are you sure you want to change your workspace path?"
              )
            ) {
              onResetWorkspace();
            }
          }}
        />
        <div className="user-avatar">MR</div>
      </div>
    </header>
  );
};
