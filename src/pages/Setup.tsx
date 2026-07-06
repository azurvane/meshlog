import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "../components/Button";
import "./Setup.css";
import { invoke } from "@tauri-apps/api/core";

interface SetupProps {
  onPathSelected: (name: string) => void;
}

export function Setup({ onPathSelected }: SetupProps) {
  async function handleClick() {
    try {
      const selected = await open({ directory: true, multiple: false });
      if (selected) {
        await invoke("initialize_project", { path: selected });
        onPathSelected(selected);
      }
    } catch (e) {
      console.error("Failed to initialize project:", e);
    }
  }

  return (
    <div className="setup-container">
      <div className="setup-accent-line" />

      <div className="setup-card">
        {/* Header */}
        <div className="setup-header">
          <span className="setup-logo-text">Palette</span>
          <span className="setup-separator">/</span>
          <span className="setup-app-name">meshlog</span>
        </div>

        {/* Content */}
        <div className="setup-content">
          <h2 className="setup-title">Initialize Workspace</h2>
          <p className="setup-subtitle">
            Select your local assets folder to begin tracking versions,
            textures, and 3D meshes.
          </p>

          <Button onClick={handleClick}>Open Folder</Button>
        </div>
      </div>
    </div>
  );
}
