import React, { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { LogColumn, MIN_PREVIEW_WIDTH } from "./LogColumn";
import { TableData } from "../utils/viewFields";
import "./DbView.css";

interface DbViewProp {
  rootPath: string;
}

export const DbView: React.FC<DbViewProp> = ({ rootPath }) => {
  const [files, setFiles] = useState<string[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [content, setContent] = useState<TableData | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const [selectedRow, setSelectedRow] = useState<number | null>(null);
  const [selectedCol, setSelectedCol] = useState<string | null>(null);

  function handleRowClick(rowIndex: number) {
    setSelectedRow(rowIndex);
    setSelectedCol(null);
  }

  function handleColClick(colKey: string) {
    setSelectedCol(colKey);
    setSelectedRow(null);
  }

  useEffect(() => {
    async function loadList() {
      try {
        const fetchedFiles = await invoke<string[]>("get_table_name", {
          rootPath,
        });
        const sorted = [...fetchedFiles].sort((a, b) => a.localeCompare(b));
        setFiles(sorted);
      } catch (err) {
        console.error("Failed to list tables:", err);
      }
    }
    loadList();
  }, [rootPath]);

  useEffect(() => {
    if (!selected) return;
    async function loadContent() {
      setIsLoading(true);
      setSelectedRow(null);
      setSelectedCol(null);
      try {
        const entries = await invoke<TableData>("get_table_entries", {
          rootPath,
          tableName: selected,
        });
        setContent(entries);
      } catch (err) {
        console.error("Failed to read table:", err);
        setContent(null);
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
    <div className="Db-view-container" ref={containerRef}>
      <LogColumn
        files={files}
        selectedFileName={selected}
        onSelectFile={handleSelectFile}
        getContainerWidth={() => containerRef.current?.clientWidth ?? 0}
        title="Dbs"
        subtitle="/Db · preview"
      />

      <div className="Db-view-preview" style={{ minWidth: MIN_PREVIEW_WIDTH }}>
        {selected ? (
          <div className="Db-view-content-wrapper">
            <div className="Db-view-header-bar">
              <span className="Db-view-filename">{selected}</span>
            </div>

            {isLoading ? (
              <div className="Db-view-preview-placeholder">
                Loading content...
              </div>
            ) : content && content.columns.length > 0 ? (
              <div className="Db-table-wrapper">
                <table className="Db-table">
                  <thead>
                    <tr>
                      {content.columns.map((col) => (
                        <th
                          key={col}
                          onClick={() => handleColClick(col)}
                          className={selectedCol === col ? "is-selected" : ""}
                        >
                          {col}
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {content.rows.map((row, rowIndex) => (
                      <tr
                        key={rowIndex}
                        onClick={() => handleRowClick(rowIndex)}
                        className={
                          selectedRow === rowIndex ? "is-selected" : ""
                        }
                      >
                        {row.map((cell, colIndex) => {
                          const colName = content.columns[colIndex];
                          const isColSelected = selectedCol === colName;
                          return (
                            <td
                              key={colIndex}
                              className={isColSelected ? "is-col-selected" : ""}
                            >
                              {cell ?? ""}
                            </td>
                          );
                        })}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : (
              <div className="Db-view-preview-placeholder">
                No data available in this table
              </div>
            )}
          </div>
        ) : (
          <div className="Db-view-preview-placeholder">
            Select a table file to view entries
          </div>
        )}
      </div>
    </div>
  );
};
