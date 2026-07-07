import React from "react";
import "./MillerColumns.css";

interface FileNode {
  name: string;
  is_dir: boolean;
  children?: FileNode[] | null;
}

interface FileMetadata {
  name: String;
  size_bytes: number;
  modified_ddmmyyyy: string;
  created_ddmmyyyy: string;
  is_dir: boolean;
  file_type: String;
  current_version: String;
  current_hash: String;
}

interface MillerColumnsProps {
  treeData: FileNode[];
  activePathIndices: number[];
  onSelectNode: (indices: number[], node: FileNode) => void;
}

export const MillerColumns: React.FC<MillerColumnsProps> = ({
  treeData,
  activePathIndices,
  onSelectNode,
}) => {
  const columns: { title: string; nodes: FileNode[]; subtitle?: string }[] = [
    { title: "Repository", nodes: treeData, subtitle: "palette / nightfall" },
  ];

  let currentLevelNodes = treeData;
  for (let i = 0; i < activePathIndices.length; i++) {
    const selectedIdx = activePathIndices[i];
    const node = currentLevelNodes[selectedIdx];
    if (node && node.is_dir && node.children) {
      // Use parent folder name as column header title, and grandparent as subtitle
      const parentName = node.name;
      const subtitleName =
        i === 0
          ? "Project — Nightfall"
          : String(currentLevelNodes[selectedIdx]?.name || "");

      columns.push({
        title: parentName,
        nodes: node.children,
        subtitle: subtitleName,
      });
      currentLevelNodes = node.children;
    }
  }

  return (
    <div className="miller-columns-container">
      {columns.map((column, colIdx) => {
        const selectedNodeIdx = activePathIndices[colIdx];

        return (
          <div className="miller-column" key={colIdx}>
            <div className="column-header">
              <div className="header-titles">
                <span className="column-title">{column.title}</span>
                {column.subtitle && (
                  <span className="column-subtitle">{column.subtitle}</span>
                )}
              </div>
              <span className="column-count">{column.nodes.length}</span>
            </div>

            <div className="column-body hide-scrollbar">
              {column.nodes.map((node, nodeIdx) => {
                const isSelected = selectedNodeIdx === nodeIdx;

                // Extracting display parameters (falling back elegantly if it's a plain txt/file)
                const isAssetFile = !node.is_dir;
                const displayVersion = isAssetFile ? "v1.0" : "";
                const displayTime = "2h ago";
                const displaySize = isAssetFile ? "4 KB" : "—";
                const displayAuthor = "team";

                return (
                  <div
                    key={nodeIdx}
                    className={`column-row-grid ${
                      isSelected ? "selected" : ""
                    }`}
                    onClick={() => {
                      const newIndices = activePathIndices.slice(0, colIdx);
                      newIndices.push(nodeIdx);
                      onSelectNode(newIndices, node);
                    }}
                  >
                    {/* Left Column Group: Orange status dot indicator + primary asset tag string */}
                    <div className="grid-left">
                      <span className="asset-tag-text">
                        {isAssetFile ? displayVersion : node.name}
                      </span>
                    </div>

                    {/* Middle Column Group: Relative tracking offsets */}
                    <span className="grid-meta-time">{displayTime}</span>
                    <span className="grid-meta-size">{displaySize}</span>

                    {/* Right Column Group: Signatures + directional cascading split arrows */}
                    <div className="grid-right">
                      <span className="grid-meta-author">{displayAuthor}</span>
                      {node.is_dir && <span className="grid-row-arrow">›</span>}
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        );
      })}
    </div>
  );
};
