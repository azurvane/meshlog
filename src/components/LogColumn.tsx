import React, { useState, useEffect } from "react";
import "./LogColumn.css";

interface LogColumnProps {
  files: string[];
  selectedFileName?: string | null;
  onSelectFile: (fileName: string) => void;
  title?: string;
  subtitle?: string;
}

export const LogColumn: React.FC<LogColumnProps> = ({
  files,
  selectedFileName,
  onSelectFile,
  title = "Files",
  subtitle = "testing sub title",
}) => {
  const [columnWidth, setColumnWidth] = useState<number>(290);
  const [isResizing, setIsResizing] = useState<boolean>(false);

  const startResize = (startX: number) => {
    setIsResizing(true);
    const startWidth = columnWidth;

    const onMove = (e: MouseEvent) => {
      const delta = e.clientX - startX;
      setColumnWidth(Math.max(200, startWidth + delta));
    };

    const onUp = () => {
      setIsResizing(false);
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };

    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  };

  return (
    <div className="file-list-container">
      <div className="file-column" style={{ width: `${columnWidth}px` }}>
        <div
          className={`column-resize-handle ${isResizing ? "resizing" : ""}`}
          onMouseDown={(e) => {
            e.preventDefault();
            startResize(e.clientX);
          }}
        />

        <div className="column-header">
          <div className="header-titles">
            <span className="column-title">{title}</span>
            {subtitle && <span className="column-subtitle">{subtitle}</span>}
          </div>
          <span className="column-count">{files.length}</span>
        </div>

        <div className="column-scrollable-container">
          <div className="column-scroll-content-wrapper">
            <div className="column-body hide-scrollbar">
              {files.map((file, idx) => {
                const isSelected = selectedFileName === file;
                const className = `file-row ${
                  isSelected ? "selected" : ""
                }`.trim();

                return (
                  <div
                    key={idx}
                    className={className}
                    onClick={() => onSelectFile(file)}
                  >
                    <span className="file-cell-name">{file}</span>
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
