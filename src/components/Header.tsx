import React from "react";
import { useState, useRef, useEffect } from "react";
import {
  SlidersHorizontal,
  PanelLeftOpen,
  Terminal,
  Settings,
  Search,
  Command,
} from "lucide-react";
import { ActionButton } from "./ActionButton";
import { FileMetadata } from "../utils/viewFields";
import { ViewMenu } from "./ViewMenu";
import "./Header.css";

interface HeaderProps {
  onResetWorkspace?: () => void;
  visibleFields: Set<keyof FileMetadata>;
  onToggleField: (key: keyof FileMetadata) => void;
  isTerminalOpen: boolean;
  onToggleTerminal: () => void;
}

/**
 * Top application header bar component. It renders the project navigation context (branding logotype,
 * directory hierarchy badges), an interactive global asset/hash lookup input bar, and buttons 
 * to toggle sub-windows (metadata column visibility dropdown, embedded shell terminal, inspector layout, or workspace resetting dialogs).
 */
export const Header: React.FC<HeaderProps> = ({
  onResetWorkspace,
  visibleFields,
  onToggleField,
  isTerminalOpen,
  onToggleTerminal,
}) => {
  // Track open/close state transitions for dashboard panels (such as the asset metadata details inspector drawer).
  const [panels, setPanels] = useState({
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

  const [isMenuOpen, setIsMenuOpen] = useState(false);

  const menuWrapperRef = useRef<HTMLDivElement>(null);

  // Listens to global pointer down actions to check if the user clicks away from the open View menu options.
  // If a click falls outside the menu boundary, it updates the toggle state to collapse the menu from view.
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (
        menuWrapperRef.current &&
        !menuWrapperRef.current.contains(e.target as Node)
      ) {
        setIsMenuOpen(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

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
          <span className="search-icon">
            <Search size={18} />
          </span>
          <input
            type="text"
            placeholder="Find asset, version, or hash..."
            className="search-input"
          />
          <kbd className="search-shortcut">
            <Command size={12} />
            <span>k</span>
          </kbd>
        </div>
      </div>

      <div className="header-right">
        <div ref={menuWrapperRef} className="view-menu-anchor">
          <ActionButton
            label="View"
            icon={<SlidersHorizontal size={18} />}
            isActive={isMenuOpen}
            onClick={() => setIsMenuOpen((prev) => !prev)}
          />
          {isMenuOpen && (
            <ViewMenu visibleFields={visibleFields} onToggle={onToggleField} />
          )}
        </div>
        <ActionButton
          label="Terminal"
          icon={<Terminal size={18} />}
          isActive={isTerminalOpen}
          onClick={onToggleTerminal}
        />
        <ActionButton
          label="Stamp"
          icon={<PanelLeftOpen size={18} />}
          isActive={panels.inspector}
          onClick={() => togglePanel("inspector")}
        />
        <ActionButton
          icon={<Settings size={18} />}
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
