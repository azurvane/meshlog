import { useState, useEffect } from "react";
import { Setup } from "./pages/Setup";
import { Home } from "./pages/Home";
import { AppSettings } from "./utils/settings";
import "./App.css";

function App() {
  const [projectPath, setProjectPath] = useState<string>("");
  const [isLoading, setIsLoading] = useState<boolean>(true);

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

  const handlePathSelected = async (path: string) => {
    if (!path) return;
    setProjectPath(path);
    await AppSettings.set("projectPath", path);
  };

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
