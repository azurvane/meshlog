import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  FileMetadata,
  DEFAULT_VISIBLE,
  GitCommitData,
  fileDetails,
  PanelView,
} from "../utils/viewFields";
import { Header } from "../components/Header";
import { StampView } from "../components/Stamp";
import { MillerColumns } from "../components/MillerColumns";
import { LogView } from "../components/LogView.tsx";
import { DbView } from "../components/DbView.tsx";
import "../theme/colors.ts";
import "./Home.css";

// get the dom from app.tsx instead of creating it here

interface FileNode {
  name: string;
  is_dir: boolean;
  children?: FileNode[] | null;
}

interface VisibleFolder {
  path: string;
  nodes: FileNode[];
}

interface HomeProps {
  filePath: string;
  onSetting: () => void;
}

export function Home({ filePath, onSetting }: HomeProps) {
  const [treeData, setTreeData] = useState<FileNode[]>([]);
  const [activePathIndices, setActivePathIndices] = useState<number[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [isStampOpen, SetIsStampOpen] = useState(false);
  const [isSettingOpen, SetIsSettingOpen] = useState(false);
  const [activeView, SetActiveView] = useState<PanelView>(PanelView.Repository);
  const [metadataMap, SetMetadataMap] = useState<
    Map<string, Map<string, FileMetadata>>
  >(new Map());
  const [activeFields, setActiveFields] =
    useState<Set<keyof FileMetadata>>(DEFAULT_VISIBLE);
  const [eligibleSet, setEligibleSet] = useState<Set<string>>(new Set());
  const previousFoldersRef = useRef<string[]>([]);
  const [fileInfo, SetFileInfo] = useState<fileDetails>({
    name: "",
    path: "",
    isDir: false,
  });
  const populateFileInfo = (name: string, path: string, isDir: boolean) => {
    SetFileInfo({
      name: name,
      path: path,
      isDir: isDir,
    });
  };

  // Toggles the visibility state of columns in the grid view. Adds or removes selected
  // metadata fields (such as asset ID, hash, description, size) to control which data points are shown.
  const toggleActiveFields = (field: keyof FileMetadata) => {
    setActiveFields((prev) => {
      const next = new Set(prev);
      if (next.has(field)) next.delete(field);
      else next.add(field);
      return next;
    });
  };

  const handleEligibleSet = async () => {
    try {
      const list: string[] = await invoke("get_uncommited_files", {
        rootPath: filePath,
      });
      const allEligiblePaths = new Set<string>();
      list.forEach((filePathStr) => {
        allEligiblePaths.add(filePathStr);
        const parts = filePathStr.split("/");
        let currentPath = "";
        for (let i = 0; i < parts.length - 1; i++) {
          currentPath = currentPath ? `${currentPath}/${parts[i]}` : parts[i];
          allEligiblePaths.add(currentPath);
        }
      });
      setEligibleSet(allEligiblePaths);
    } catch (err) {
      console.error("Failed to refresh eligible set:", err);
    }
  };

  const loadFileTree = async () => {
    try {
      const tree: FileNode[] = await invoke("get_file_tree", {
        absoluteFolderPath: filePath,
      });

      setTreeData(tree);
      setActivePathIndices([]);

      handleEligibleSet();
      await fetchMetadataForNodes(tree, filePath);
    } catch (err: any) {
      setError(err.toString());
    }
  };

  // Trigger project workspace setup on path changes. Instructs the backend database manager
  // to sync files, fetch directory listings, and initially cache metadata parameters for all root files.
  useEffect(() => {
    async function loadProject() {
      if (!filePath) return;
      try {
        setLoading(true);
        setError(null);

        await invoke("initialize_project", { rootPath: filePath });
        await invoke("populate_db", { rootPath: filePath });
        await invoke("populate_log_md", { rootPath: filePath });
        await invoke("start_watching", { rootPath: filePath });
        await loadFileTree();
      } catch (err: any) {
        setError(err.toString());
      } finally {
        setLoading(false);
      }
    }
    loadProject();

    return () => {
      invoke("stop_watching").catch((err) =>
        console.error("Failed to stop watcher:", err)
      );
    };
  }, [filePath]);

  useEffect(() => {
    const unlistenPromise = listen<string>("fs-changed", () => {
      loadFileTree();
    });

    return () => {
      unlistenPromise.then((unlistenFn) => unlistenFn());
    };
  }, [filePath]);

  function diff(oldList: string[], newList: string[]) {
    const oldSet = new Set(oldList);
    const newSet = new Set(newList);

    const added = newList.filter((item) => !oldSet.has(item));
    const removed = oldList.filter((item) => !newSet.has(item));

    return { added, removed };
  }

  // evicting an entire folder — O(1), regardless of how many files were inside it
  function evictFolder(
    store: Map<string, Map<string, FileMetadata>>,
    folder: string
  ): Map<string, Map<string, FileMetadata>> {
    const next = new Map(store);
    next.delete(folder);
    return next;
  }

  useEffect(() => {
    async function useVisibleFolderSync(
      indices: number[],
      treeData: FileNode[]
    ) {
      const visibleFolders = getVisibleFolderPaths(indices, treeData);
      const currentPaths = visibleFolders.map((folder) => folder.path);
      const { added, removed } = diff(previousFoldersRef.current, currentPaths);

      if (removed.length > 0) {
        SetMetadataMap((prev) => {
          let nextMap = prev;
          removed.forEach((folderPath) => {
            nextMap = evictFolder(nextMap, folderPath);
          });
          return nextMap;
        });
      }

      const folderLookup = new Map<string, FileNode[]>(
        visibleFolders.map((folder) => [folder.path, folder.nodes])
      );

      if (added.length > 0) {
        await Promise.all(
          added.map((folderPath) => {
            const nodes = folderLookup.get(folderPath) || [];
            return fetchMetadataForNodes(nodes, folderPath);
          })
        );
      }

      previousFoldersRef.current = currentPaths;
    }
    useVisibleFolderSync(activePathIndices, treeData);
  }, [activePathIndices, treeData]);

  // Iterates through list nodes, making asynchronous requests to the database backend for file parameters.
  // Merges new metadata values into the local React state map to update visible table values.
  const fetchMetadataForNodes = async (nodes: FileNode[], basePath: string) => {
    const results = new Map<string, Map<string, FileMetadata>>();

    for (let i = 0; i < nodes.length; i++) {
      const node = nodes[i];
      if (!node.is_dir) {
        const absolutePath = `${basePath}/${node.name}`;
        try {
          const meta = await invoke<FileMetadata>("get_file_metadata", {
            rootPath: filePath,
            absoluteFilePath: absolutePath,
          });
          if (!results.has(basePath)) {
            results.set(basePath, new Map());
          }
          results.get(basePath)!.set(node.name, meta);
        } catch (err) {
          console.error(`Metadata fetch failed for ${absolutePath}:`, err);
        }
      } else {
        const absolutePath = `${basePath}/${node.name}`;
        try {
          const meta = await invoke<FileMetadata>("get_directory_metadata", {
            absoluteFilePath: absolutePath,
          });
          if (!results.has(basePath)) {
            results.set(basePath, new Map());
          }
          results.get(basePath)!.set(node.name, meta);
        } catch (err) {
          console.error(`Metadata fetch failed for ${absolutePath}:`, err);
        }
      }
    }
    SetMetadataMap((prevMap) => {
      const nextOuterMap = new Map(prevMap);
      for (const [currentPath, incomingFileMetadata] of results) {
        const existingInnerMap = prevMap.get(currentPath);
        const nextInnerMap = existingInnerMap
          ? new Map(existingInnerMap)
          : new Map();
        for (const [fileName, metadata] of incomingFileMetadata) {
          nextInnerMap.set(fileName, metadata);
        }
        nextOuterMap.set(currentPath, nextInnerMap);
      }
      return nextOuterMap;
    });
  };

  // Callback triggered when a user clicks a row in the Miller columns directory layout.
  // Updates selected indexes, crawls nested paths, fetches folder children metadata,
  // and purges out-of-scope files from the metadata cache to optimize memory footprint.
  const handleSelectNode = async (indices: number[]) => {
    setActivePathIndices(indices);
  };

  const handleToggleStamp = () => {
    SetIsStampOpen((prev) => !prev);
  };

  const handleToggleSetting = () => {
    SetIsSettingOpen((prev) => !prev);
  };

  const hanndleActivePanel = async (Panel: PanelView) => {
    SetActiveView(Panel);
  };

  const handleGitCommitData = async (data: GitCommitData): Promise<boolean> => {
    try {
      const oldPath = await invoke<string | null>("get_old_path", {
        rootPath: filePath,
        newRelativeFilePath: data.path,
      });
      if (eligibleSet.has(data.path) && !oldPath) {
        await invoke<string>("get_new_asset_id", {
          rootPath: filePath,
          filename: data.name,
        });
      }

      await invoke<FileMetadata>("commit_stamp", {
        rootPath: filePath,
        relativeFilePath: data.path,
        tag: data.tag,
        summary: data.summary,
        detail: data.detail,
      });
      handleEligibleSet();

      return true;
    } catch (err) {
      console.error("Commit failed:", err);
      return false;
    }
  };

  const getVisibleFolderPaths = (
    indices: number[],
    treeData: FileNode[]
  ): VisibleFolder[] => {
    const result: VisibleFolder[] = [];
    let currentNodes = treeData;
    let currentPath = filePath;
    for (const index of indices) {
      const targetNode = currentNodes[index];
      if (targetNode && targetNode.is_dir) {
        currentPath = `${currentPath}/${targetNode.name}`;
        if (targetNode.children) {
          currentNodes = targetNode.children;
          result.push({ path: currentPath, nodes: currentNodes });
        } else {
          result.push({ path: currentPath, nodes: [] });
        }
      }
    }
    return result;
  };

  return (
    <div className="home-layout">
      <Header
        onSetting={onSetting}
        visibleFields={activeFields}
        onToggleField={toggleActiveFields}
        isStampOpen={isStampOpen}
        onToggleStamp={handleToggleStamp}
        isSettingOpen={isSettingOpen}
        onToggleSetting={handleToggleSetting}
        currentView={activeView}
        SetActivePanelView={hanndleActivePanel}
      />

      {/* Main core layout zone split into workspace panels and the right Stamp sidebar */}
      <div className="workspace-container">
        <div className="left-workspace-stack">
          <main className="content-viewport">
            {loading && (
              <div className="status-overlay">Loading folder tree state...</div>
            )}
            {error && (
              <div className="status-overlay error">Error: {error}</div>
            )}

            {!loading && !error && activeView === PanelView.Repository && (
              <MillerColumns
                filePath={filePath}
                treeData={treeData}
                activePathIndices={activePathIndices}
                onSelectNode={handleSelectNode}
                visibleFields={activeFields}
                metadataMap={metadataMap}
                eligible={eligibleSet}
                setFileDetails={populateFileInfo}
              />
            )}

            {!loading && !error && activeView === PanelView.LogView && (
              <LogView rootPath={filePath} />
            )}

            {!loading && !error && activeView === PanelView.Database && (
              <DbView rootPath={filePath} />
            )}
          </main>
        </div>

        {/* Draggable Stamp column sidebar rendering on the far right */}
        {isStampOpen && (
          <StampView
            rootPath={filePath}
            fileInfo={fileInfo}
            versionPrefix=""
            eligibleSet={eligibleSet}
            handleGitCommitData={handleGitCommitData}
          />
        )}
      </div>
    </div>
  );
}
