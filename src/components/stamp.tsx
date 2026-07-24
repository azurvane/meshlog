import React, { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "./Button";
import { GitCommitData, fileDetails } from "../utils/viewFields";
import "./Stamp.css";

interface StampViewProps {
  fileInfo: fileDetails;
  versionPrefix: string;
  eligibleSet: Set<string>;
  handleGitCommitData: (data: GitCommitData) => Promise<boolean>;
  handleAssetid: (fileInfo: fileDetails) => Promise<string>;
}

/**
 * StampView (Version Update) component.
 * Renders on the right side of the screen as an expandable, draggable panel column.
 * Uses flex layout adjustments to scale all child fields cleanly without empty black gaps.
 */
export const StampView: React.FC<StampViewProps> = ({
  fileInfo,
  versionPrefix,
  eligibleSet,
  handleGitCommitData,
  handleAssetid,
}) => {
  // ==========================================
  // PLACEHOLDER STATE VARIABLES FOR USER INPUTS
  // ==========================================
  const [versionInput, setVersionInput] = useState<string>("");
  const [summaryInput, setSummaryInput] = useState<string>("");
  const [detailedMessageInput, setDetailedMessageInput] = useState<string>("");
  const [tag, SetTag] = useState<string>("");

  // ==========================================
  // PLACEHOLDER STATE FOR FETCHED DATA
  // ==========================================
  const [previousVersion, setPreviousVersion] = useState<string>("");
  const [isValidVersion, SetIsValidVersion] = useState<boolean>(false);
  const [canSubmit, SetCanSubmit] = useState<boolean>(false);

  // DRAG RESIZING STATES (Default width mimics standard miller columns: 320px)
  const [width, setWidth] = useState<number>(320);
  const isDragging = useRef<boolean>(false);
  const startX = useRef<number>(0);
  const startWidth = useRef<number>(0);
  const [assetid, SetAssetid] = useState<string>("assetid");
  const latestRequestId = useRef(0);

  // Handles click-drag initialization on the left edge border handle
  const handleMouseDown = (e: React.MouseEvent) => {
    isDragging.current = true;
    startX.current = e.clientX;
    startWidth.current = width;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
  };

  useEffect(() => {
    const handleCanSubmit = () => {
      SetCanSubmit(
        !fileInfo.isDir &&
          eligibleSet.has(fileInfo.path) &&
          tag !== "" &&
          summaryInput.trim() !== "" &&
          assetid !== ""
      );
    };
    handleCanSubmit();
    if (canSubmit) {
      console.log("commit button open");
    } else {
      console.log("commit button blocked");
    }
  }, [fileInfo, eligibleSet, tag, summaryInput, assetid]);

  useEffect(() => {
    async function fetchAsset() {
      if (!fileInfo.isDir && fileInfo.path) {
        try {
          const id = await handleAssetid(fileInfo);
          SetAssetid(id);
        } catch (err) {
          console.error("Failed to resolve asset id for", fileInfo.path, err);
          SetAssetid(""); // empty, not a fake placeholder — see point 4
        }
      }
    }
    fetchAsset();
  }, [fileInfo]);

  // Listens to global mouse events to scale the column width relative to the right edge window constraint
  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isDragging.current) return;
      // Moving left increases width because it's anchored on the right side
      const deltaX = startX.current - e.clientX;
      const newWidth = Math.max(
        260,
        Math.min(800, startWidth.current + deltaX)
      );
      setWidth(newWidth);
    };

    const handleMouseUp = () => {
      if (!isDragging.current) return;
      isDragging.current = false;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [width]);

  // Simulation effect to fetch historical data records
  useEffect(() => {
    const fetchPreviousVersion = async () => {
      try {
        const mockPreviousVersion = "v4.0";
        setPreviousVersion(mockPreviousVersion);
      } catch (error) {
        console.error("Failed to fetch previous version", error);
      }
    };
    fetchPreviousVersion();
  }, [fileInfo.name]);

  const handleCommitUpdate = async () => {
    try {
      const dataToCommit: GitCommitData = {
        name: fileInfo.name,
        path: fileInfo.path,
        tag: tag,
        summary: summaryInput,
        detail: detailedMessageInput,
      };

      console.log("Submitting version update details:", {
        fullVersion: `${versionPrefix}${versionInput}`,
        summary: summaryInput,
        detailedMessage: detailedMessageInput,
        tag: tag,
      });

      // Call the prop function directly with the prepared data object
      const succeeded = await handleGitCommitData(dataToCommit);

      // reset evey value
      if (succeeded) {
        SetTag("");
        setVersionInput("");
        setDetailedMessageInput("");
        setSummaryInput("");
      }
    } catch (error) {
      console.error("Failed to commit the file", error);
    }
  };

  async function handleInvalidVersion(currentVersion: string) {
    const requestId = ++latestRequestId.current; // this call claims the newest ticket
    try {
      const tag = await invoke<string>("stamp_version", {
        assetid,
        version: currentVersion,
      });
      if (requestId === latestRequestId.current) {
        // only apply if still the newest
        SetIsValidVersion(false);
        SetTag(tag);
      }
    } catch {
      if (requestId === latestRequestId.current) {
        SetIsValidVersion(true);
        SetTag("");
      }
    }
  }

  return (
    <div className="stamp-view-container" style={{ width: `${width}px` }}>
      {/* Left-aligned resize handle column line */}
      <div className="stamp-resize-handle" onMouseDown={handleMouseDown} />

      {/* Header bar section */}
      <div className="stamp-view-header">
        <span className="stamp-header-title">VERSION UPDATE</span>
      </div>

      {/* Main Core Form: Flex-grow layout ensures no empty black backgrounds */}
      <div className="stamp-view-content">
        {/* Target File Info Section */}
        <div className="stamp-form-group">
          <label className="stamp-form-label">TARGET FILE</label>
          <div className="stamp-file-info-card">
            <div className="stamp-file-meta">
              <span className="stamp-file-name">
                {fileInfo.isDir
                  ? "— folder —"
                  : fileInfo.name || "place holder name"}
              </span>
              <span className="stamp-file-path">
                {fileInfo.path || "place holder path"}
              </span>
            </div>
          </div>
        </div>

        {/* Version Input Section with dynamic conditional prefix block */}
        <div className="stamp-form-group">
          <label className="stamp-form-label">
            VERSION NAME <span className="stamp-required-indicator">*</span>
          </label>
          <div
            className="stamp-version-input-wrapper"
            style={{
              border: isValidVersion
                ? "2px solid red"
                : "2px solid transparent",
            }}
          >
            <span className="stamp-version-prefix-block">{assetid + "-v"}</span>
            <input
              type="text"
              className="stamp-form-input stamp-version-field"
              placeholder="xx.xx.xxxx"
              value={versionInput}
              onChange={async (e) => {
                setVersionInput(e.target.value);
                handleInvalidVersion(e.target.value);
              }}
            />
          </div>
          <div className="stamp-version-sub-info">
            <span>Previous: {previousVersion}</span>
          </div>
        </div>

        {/* Summary of Changes Input Section */}
        <div className="stamp-form-group">
          <label className="stamp-form-label">
            SUMMARY OF CHANGES{" "}
            <span className="stamp-required-indicator">*</span>
          </label>
          <input
            type="text"
            className="stamp-form-input"
            placeholder="Warmed midtones and cleaned pore detail"
            value={summaryInput}
            onChange={(e) => setSummaryInput(e.target.value)}
          />
        </div>

        {/* Detailed Message Textarea Section: Using flex-grow auto scaling layout */}
        <div className="stamp-form-group stamp-flex-grow-group">
          <label className="stamp-form-label">DETAILED MESSAGE</label>
          <textarea
            className="stamp-form-textarea stamp-flexible-textarea"
            placeholder="- Describe your specific updates here"
            value={detailedMessageInput}
            onChange={(e) => setDetailedMessageInput(e.target.value)}
          />
        </div>
      </div>

      {/* Footer Commit Wrapper */}
      <div className="stamp-view-footer">
        <Button disabled={!canSubmit} onClick={handleCommitUpdate}>
          Commit / Update Version
        </Button>
      </div>
    </div>
  );
};
