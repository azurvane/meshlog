import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "../components/Button";
import "./Setup.css";
import { invoke } from "@tauri-apps/api/core";

interface SetupProps {
  onPathSelected: (name: string) => void;
}

/**
 * Onboarding screen displayed when the application does not have a registered project workspace folder.
 * Renders a layout greeting the user and offering a button to browse their system directories
 * to choose a workspace directory that the application should index and display.
 */
export function Setup({ onPathSelected }: SetupProps) {
  // Triggers the Tauri dialog plugin to open the operating system's native folder picker.
  // Once a path is selected, it calls the backend Rust API to initialize metadata tracking
  // and database structures for that folder, before notifying the root App component.
  async function handleClick() {
    try {
      const selected = await open({ directory: true, multiple: false });
      if (selected) {
        await invoke("initialize_project", { rootPath: selected });
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
