import React, { useState, useEffect } from "react";
import { LogColumn } from "./LogColumn";
import "./LogView.css";

interface LogViewProp {
  rootPath: string;
  fetchFiles: () => Promise<string[]>;
}

export const LogView: React.FC<LogViewProp> = ({ rootPath, fetchFiles }) => {
  const tempFunction2 = (temp: string) => {};

  return (
    <div style={{ display: "flex", height: "100%", width: "100%" }}>
      <LogColumn
        rootPath={rootPath}
        fetchFiles={fetchFiles}
        selectedFileName=""
        onSelectFile={tempFunction2}
        title="Logs" // <-- Pass a visible title
      />
    </div>
  );
};
