import React from "react";
import { useState, useRef, useEffect } from "react";
import {
  SlidersHorizontal,
  PanelRightOpen,
  Settings,
  ChevronDown,
} from "lucide-react";
import { ActionButton } from "./ActionButton";
import { FileMetadata, PanelView, VIEW_REGISTRY } from "../utils/viewFields";
import { ViewMenu } from "./ViewMenu";
import { SwitchView } from "./SwitchView";
import "./Header.css";

interface HeaderProps {
  onSetting: () => void;
  visibleFields: Set<keyof FileMetadata>;
  onToggleField: (key: keyof FileMetadata) => void;
  isStampOpen: boolean;
  isSettingOpen: boolean;
  onToggleStamp: () => void;
  onToggleSetting: () => void;
  currentView: PanelView;
  SetActivePanelView: (Panel: PanelView) => void;
}

export const Header: React.FC<HeaderProps> = ({
  onSetting,
  visibleFields,
  onToggleField,
  isStampOpen,
  onToggleStamp,
  isSettingOpen,
  onToggleSetting,
  currentView,
  SetActivePanelView,
}) => {
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
          label="Stamp"
          icon={<PanelRightOpen size={18} />}
          isActive={isStampOpen}
          onClick={onToggleStamp}
        />
        <ActionButton
          icon={<Settings size={18} />}
          isActive={isSettingOpen}
          onClick={() => {
            onToggleSetting();
            onSetting();
          }}
        />
        <div className="user-avatar">MR</div>
      </div>
    </header>
  );
};
