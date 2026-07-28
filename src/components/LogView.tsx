import React, { useState, useEffect } from "react";
import { LogColumn } from "./LogColumn";
import { invoke } from "@tauri-apps/api/core";
import "./LogView.css";

interface LogViewProp {
  rootPath: string;
}

export const LogView: React.FC<LogViewProp> = ({ rootPath }) => {
  const [files, setFiles] = useState<string[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [content, setContent] = useState<string>("");
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    async function loadList() {
      try {
        const fetchedFiles = await invoke<string[]>("get_log_files", {
          rootPath,
        });
        const sorted = [...fetchedFiles].sort((a, b) => a.localeCompare(b));
        setFiles(sorted);
      } catch (err) {
        console.error("Failed to list logs:", err);
      }
    }
    loadList();
  }, [rootPath]);

  useEffect(() => {
    if (!selected) return;
    async function loadContent() {
      setIsLoading(true);
      try {
        const text = await invoke<string>("get_log_content", {
          rootPath,
          fileName: selected,
        });
        setContent(text);
      } catch (err) {
        console.error("Failed to read log:", err);
        setContent("");
      } finally {
        setIsLoading(false);
      }
    }
    loadContent();
  }, [selected, rootPath]);

  const handleSelectFile = async (selectedFileName: string) => {
    setSelected(selectedFileName);
  };

  return (
    <div className="log-view-container">
      <LogColumn
        files={files}
        selectedFileName={selected}
        onSelectFile={handleSelectFile}
        title="Logs"
        subtitle="/logs · markdown"
      />

      <div className="log-view-preview">
        {selected ? (
          <div className="log-view-content-wrapper">
            <div className="log-view-header-bar">
              <span className="log-view-filename">{selected}</span>
            </div>

            {isLoading ? (
              <div className="log-view-preview-placeholder">
                Loading content...
              </div>
            ) : (
              <pre className="log-view-raw-content">{content}</pre>
            )}
          </div>
        ) : (
          <div className="log-view-preview-placeholder">
            Select a log file to view content
          </div>
        )}
      </div>
    </div>
  );
};
