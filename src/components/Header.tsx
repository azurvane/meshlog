import React from "react";
import { useState, useRef, useEffect } from "react";
import {
  SlidersHorizontal,
  PanelLeftOpen,
  Terminal,
  Settings,
  Search,
} from "lucide-react";
import { ActionButton } from "./ActionButton";
import { FileMetadata } from "../utils/viewFields";
import { ViewMenu } from "./ViewMenu";
import "./Header.css";

interface HeaderProps {
  onResetWorkspace?: () => void;
  visibleFields: Set<keyof FileMetadata>;
  onToggleField: (key: keyof FileMetadata) => void;
}

export const Header: React.FC<HeaderProps> = ({
  onResetWorkspace,
  visibleFields,
  onToggleField,
}) => {
  // Track each panel's visibility independently
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
            <Search size={18} />{" "}
          </span>
          <input
            type="text"
            placeholder="Find asset, version, or hash..."
            className="search-input"
          />
          <kbd className="search-shortcut">⌘K</kbd>
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
          isActive={panels.terminal}
          onClick={() => togglePanel("terminal")}
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
