import React from "react";
import { useState, useRef, useEffect } from "react";
import {
  SlidersHorizontal,
  PanelRightOpen,
  Terminal,
  Settings,
  ChevronDown,
} from "lucide-react";
import { ActionButton } from "./ActionButton";
import { FileMetadata, PanelView, VIEW_REGISTRY } from "../utils/viewFields";
import { ViewMenu } from "./ViewMenu";
import { SwitchView } from "./SwitchView";
import "./Header.css";

interface HeaderProps {
  onResetWorkspace?: () => void;
  visibleFields: Set<keyof FileMetadata>;
  onToggleField: (key: keyof FileMetadata) => void;
  isTerminalOpen: boolean;
  isStampOpen: boolean;
  onToggleTerminal: () => void;
  onToggleStamp: () => void;
  currentView: PanelView;
  SetActivePanelView: (Panel: PanelView) => void;
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
  isStampOpen,
  onToggleStamp,
  currentView,
  SetActivePanelView,
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

  const [isMenuOpen, SetIsMenuOpen] = useState(false);
  const [isSwitchViewOpen, SetIsSwitchViewOpen] = useState(false);

  const viewMenuWrapperRef = useRef<HTMLDivElement>(null);
  const logoWrapperRef = useRef<HTMLDivElement>(null);

  function useClickOutside(
    ref: React.RefObject<HTMLElement | null>,
    onOutside: (isOpen: boolean) => void
  ) {
    useEffect(() => {
      const handleClickOutside = (e: MouseEvent) => {
        if (ref.current && !ref.current.contains(e.target as Node)) {
          onOutside(false);
        }
      };

      document.addEventListener("mousedown", handleClickOutside);
      return () =>
        document.removeEventListener("mousedown", handleClickOutside);
    }, [ref, onOutside]);
  }

  useClickOutside(viewMenuWrapperRef, SetIsMenuOpen);
  useClickOutside(logoWrapperRef, SetIsSwitchViewOpen);

  const activeViewLabel =
    VIEW_REGISTRY.find((v) => v.view === currentView)?.label || "Repository";

  return (
    <header className="app-header">
      <div className="header-left">
        <div className="logo-badge">P</div>
        <div className="switch-view-trigger-container" ref={logoWrapperRef}>
          <button
            className={`switch-view-trigger ${
              isSwitchViewOpen ? "is-open" : ""
            }`}
            onClick={() => SetIsSwitchViewOpen((prev) => !prev)}
            aria-expanded={isSwitchViewOpen}
          >
            <span className="app-title">palette</span>
            <span className="project-divider">/</span>
            <span className="project-name">nightfall</span>
            <span className="project-divider">/</span>
            <span className="active-view-name">{activeViewLabel}</span>
            <ChevronDown className="switch-view-caret" size={14} />
          </button>

          {isSwitchViewOpen && (
            <div className="switch-view-dropdown-wrapper">
              <SwitchView
                currentView={currentView}
                onSelect={(view) => {
                  SetActivePanelView(view);
                  SetIsSwitchViewOpen(false);
                }}
              />
            </div>
          )}
        </div>
      </div>

      <div className="header-right">
        <div ref={viewMenuWrapperRef} className="view-menu-anchor">
          <ActionButton
            label="View"
            icon={<SlidersHorizontal size={18} />}
            isActive={isMenuOpen}
            onClick={() => SetIsMenuOpen((prev) => !prev)}
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
          icon={<PanelRightOpen size={18} />}
          isActive={isStampOpen}
          onClick={onToggleStamp}
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
