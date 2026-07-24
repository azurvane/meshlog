import React from "react";
import { useState } from "react";
import { FileMetadata, FIELD_REGISTRY } from "../utils/viewFields";
import "./MillerColumns.css";

interface FileNode {
  name: string;
  is_dir: boolean;
  children?: FileNode[] | null;
}

interface MillerColumnsProps {
  filePath: string;
  treeData: FileNode[];
  activePathIndices: number[];
  onSelectNode: (indices: number[]) => void;
  visibleFields: Set<keyof FileMetadata>;
  metadataMap: Map<string, Map<string, FileMetadata>>;
  eligible: Set<string>;
  setFileDetails: (name: string, path: string, isDir: boolean) => void;
}

/**
 * Miller Columns layout component. Displays hierarchical folder paths as adjacent scrollable panes.
 * Renders sub-directories dynamically as users select folder items, supporting layout custom sizing,
 * adjustable column dividers, and metadata grid fields display per item node.
 */
export const MillerColumns: React.FC<MillerColumnsProps> = ({
  filePath,
  treeData,
  activePathIndices,
  onSelectNode,
  visibleFields,
  metadataMap,
  eligible,
  setFileDetails,
}) => {
  // Converts raw byte integers into display-friendly values. Rounds down size parameters
  // and appends appropriate unit labels (e.g., 'B', 'KB', 'MB') for readable layout rendering.
  const formatBytes = (bytes: number | undefined | null): string => {
    if (bytes === undefined || bytes === null) return "—";
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1048576) return `${(bytes / 1024).toFixed(0)} KB`;
    return `${(bytes / 1048576).toFixed(0)} MB`;
  };

  const columns: {
    title: string;
    nodes: FileNode[];
    basePath: string;
    relPath: string;
    subtitle?: string;
  }[] = [
    {
      title: "Repository",
      nodes: treeData,
      basePath: filePath,
      relPath: "",
      subtitle: "palette / nightfall",
    },
  ];

  let currentLevelNodes = treeData;
  let runningAbs = filePath;
  let runningRel = "";

  for (let i = 0; i < activePathIndices.length; i++) {
    const selectedIdx = activePathIndices[i];
    const node = currentLevelNodes[selectedIdx];
    if (node && node.is_dir && node.children) {
      const parentName = node.name;
      runningAbs = `${runningAbs}/${parentName}`;
      runningRel = runningRel ? `${runningRel}/${parentName}` : parentName;

      const subtitleName =
        i === 0
          ? "Project — Nightfall"
          : String(currentLevelNodes[selectedIdx]?.name || "");

      columns.push({
        title: parentName,
        nodes: node.children,
        basePath: runningAbs,
        relPath: runningRel,
        subtitle: subtitleName,
      });
      currentLevelNodes = node.children;
    }
  }

  const [columnWidths, setColumnWidths] = useState<Record<number, number>>({});
  const [activeResizeCol, setActiveResizeCol] = useState<number | null>(null);
  const getColumnWidth = (colIdx: number) => columnWidths[colIdx] ?? 290;

  const visibleFieldList = FIELD_REGISTRY.filter(
    (field) => field.key === "name" || visibleFields.has(field.key)
  );

  // Installs active drag tracking event handlers on mouse down. Tracks horizontal mouse moves
  // across the window object to adjust column width in state, capping sizes to a 200px minimum.
  const startResize = (colIdx: number, startX: number) => {
    setActiveResizeCol(colIdx);
    const startWidth = getColumnWidth(colIdx);

    const onMove = (e: MouseEvent) => {
      const delta = e.clientX - startX;
      setColumnWidths((prev) => ({
        ...prev,
        [colIdx]: Math.max(200, startWidth + delta),
      }));
    };

    const onUp = () => {
      setActiveResizeCol(null);
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };

    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  };

  // Resolves the string value to display for a specific cell item row field.
  // Handles formatting constraints and displays directories with a null dash marker placeholder.
  const getFieldValue = (
    key: keyof FileMetadata,
    node: FileNode,
    metadata: FileMetadata | null
  ): string => {
    if (key === "name") return node.name;
    if (node.is_dir) return "—";

    if (key === "size_bytes") {
      return formatBytes(metadata?.size_bytes);
    }

    const value = metadata?.[key];
    return value !== undefined && value !== null ? String(value) : "—";
  };

  return (
    <div className="miller-columns-container">
      {columns.map((column, colIdx) => {
        const selectedNodeIdx = activePathIndices[colIdx];

        // Formats CSS Grid template tracks. Translates registered field sizes (minimum pixel width, flex weights)
        // into standard css grid track templates (e.g., minmax(80px, 1fr)) to structure the column sub-headers.
        const getGridColumnStyle = (
          field: (typeof FIELD_REGISTRY)[number]
        ): string => {
          const minW = field.minWidth || "80px";
          const flexW = field.flexWeight || "1fr";
          return `minmax(${minW}, ${flexW})`;
        };

        const gridLayoutString = visibleFieldList
          .map((f) => getGridColumnStyle(f))
          .join(" ");

        return (
          <div
            className="miller-column"
            key={column.basePath}
            style={{ width: `${getColumnWidth(colIdx)}px` }}
          >
            <div
              className={`column-resize-handle ${
                activeResizeCol === colIdx ? "resizing" : ""
              }`}
              onMouseDown={(e) => {
                e.preventDefault();
                startResize(colIdx, e.clientX);
              }}
            />

            <div className="column-header">
              <div className="header-titles">
                <span className="column-title">{column.title}</span>
                {column.subtitle && (
                  <span className="column-subtitle">{column.subtitle}</span>
                )}
              </div>
              <span className="column-count">{column.nodes.length}</span>
            </div>

            <div className="column-scrollable-container">
              <div className="column-scroll-content-wrapper">
                {/* Sub-Header Row Label Fields */}
                <div
                  className="column-fields-sub-header"
                  style={{ gridTemplateColumns: gridLayoutString }}
                >
                  {visibleFieldList.map((field) => (
                    <span key={field.key} className="sub-header-field-label">
                      {field.label}
                    </span>
                  ))}
                </div>

                <div className="column-body hide-scrollbar">
                  {column.nodes.map((node, nodeIdx) => {
                    const isSelected = selectedNodeIdx === nodeIdx;
                    const metadata =
                      metadataMap.get(column.basePath)?.get(node.name) || null;
                    const relPath = column.relPath
                      ? `${column.relPath}/${node.name}`
                      : node.name;
                    const isEligible = eligible.has(relPath);
                    const className = `column-row-grid ${
                      isSelected ? "selected" : isEligible ? "eligible" : ""
                    }`.trim();

                    return (
                      <div
                        key={nodeIdx}
                        className={className}
                        style={{ gridTemplateColumns: gridLayoutString }}
                        onClick={() => {
                          const newIndices = activePathIndices.slice(0, colIdx);
                          newIndices.push(nodeIdx);
                          onSelectNode(newIndices);
                          setFileDetails(node.name, relPath, node.is_dir);
                        }}
                      >
                        {visibleFieldList.map((field) => (
                          <span key={field.key} className="grid-cell">
                            {getFieldValue(field.key, node, metadata)}
                          </span>
                        ))}
                        {node.is_dir && (
                          <span className="grid-row-arrow">›</span>
                        )}
                      </div>
                    );
                  })}
                </div>
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
};
