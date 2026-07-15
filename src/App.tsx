import { useState, useEffect } from "react";
import { Setup } from "./pages/Setup";
import { Home } from "./pages/Home";
import { AppSettings } from "./utils/settings";
import "./App.css";

/**
 * Root entry component of the application. It handles the initial load of settings
 * and manages top-level navigation state. By looking up the stored project workspace path,
 * it directs the user either to the Setup screen (onboarding folder selection) or the main Home 
 * page (interactive workspace dashboard).
 */
function App() {
  const [projectPath, setProjectPath] = useState<string>("");
  const [isLoading, setIsLoading] = useState<boolean>(true);

  // On application mount, load the user's previously saved project path from local configuration file.
  // This avoids prompting the user for workspace folder selection on every launch.
  useEffect(() => {
    async function loadSettings() {
      try {
        const savedPath = await AppSettings.get("projectPath");

        // Safety Catch: Ensure null, undefined, or invalid objects become a clean ""
        if (!savedPath || typeof savedPath !== "string") {
          setProjectPath("");
        } else {
          setProjectPath(savedPath);
        }
      } catch (error) {
        console.error("Failed to load settings:", error);
        setProjectPath(""); // Fallback on failure
      } finally {
        setIsLoading(false);
      }
    }
    loadSettings();
  }, []);

  // Callback invoked when the user selects a valid workspace directory. Updates the React state 
  // to render the main interface and persists the path to disk settings for future sessions.
  const handlePathSelected = async (path: string) => {
    if (!path) return;
    setProjectPath(path);
    await AppSettings.set("projectPath", path);
  };

  // Callback invoked when resetting the current project path (e.g. via Settings button). Clears the stored
  // path both in local React state and disk configuration, redirecting the user to the Setup onboarding screen.
  const handleResetPath = async () => {
    setProjectPath(""); // Force state back to empty string to show Setup page
    await AppSettings.set("projectPath", ""); // Update the permanent disk file
  };

  if (isLoading) {
    // Basic un-styled text to verify if the loader itself is breaking
    return (
      <div style={{ color: "white", padding: "20px", textAlign: "center" }}>
        Loading workspace settings...
      </div>
    );
  }

  // Strict check: Trim white spaces just in case an empty string got saved with spaces
  if (projectPath.trim() === "") {
    return <Setup onPathSelected={handlePathSelected} />;
  } else {
    return <Home filePath={projectPath} onResetPath={handleResetPath} />;
  }
}

export default App;
