import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FileMetadata, DEFAULT_VISIBLE } from "../utils/viewFields";
import { Header } from "../components/Header";
import { MillerColumns } from "../components/MillerColumns";
import { TerminalView } from "../components/Terminal";
import "../theme/colors.ts";
import "./Home.css";

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
  onResetPath: () => void;
}

/**
 * Main workspace dashboard component. It coordinates directory browser navigation,
 * handles communication with the backend Rust API to initialize projects and retrieve
 * file listings, manages custom file metadata fields, and displays the command line terminal drawer.
 */
export function Home({ filePath, onResetPath }: HomeProps) {
  const [treeData, setTreeData] = useState<FileNode[]>([]);
  const [activePathIndices, setActivePathIndices] = useState<number[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [userName, setUserName] = useState<string | null>(null);
  const [hostname, setHostname] = useState<string | null>(null);
  const [isTerminalOpen, setIsTerminalOpen] = useState(false);
  const [metadataMap, setMetadataMap] = useState<
    Map<string, Map<string, FileMetadata>>
  >(new Map());
  const [activeFields, setActiveFields] =
    useState<Set<keyof FileMetadata>>(DEFAULT_VISIBLE);
  const previousFoldersRef = useRef<string[]>([filePath]);

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

  // Query backend Rust shell services to retrieve local OS username and hostname information.
  // This metadata is used to build a realistic prompt label (e.g. user@host ~ %) inside the terminal.
  useEffect(() => {
    async function fetchUserInfo() {
      try {
        const [user, host] = await invoke<[string, string]>("get_user_info");
        setUserName(user);
        setHostname(host);
      } catch (err: any) {
        console.error("Failed to fetch user info:", err);
      }
    }
    fetchUserInfo();
  }, []);

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

        const tree: FileNode[] = await invoke("get_file_tree", {
          absoluteFolderPath: filePath,
        });
        setTreeData(tree);
        setActivePathIndices([]);

        await fetchMetadataForNodes(tree, filePath);
      } catch (err: any) {
        setError(err.toString());
      } finally {
        setLoading(false);
      }
    }
    loadProject();
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
        setMetadataMap((prev) => {
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
            absoluteFilePath: absolutePath,
            rootPath: filePath,
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
    setMetadataMap((prevMap) => {
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

  const handleToggleTerminal = () => {
    setIsTerminalOpen((prev) => !prev);
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
        onResetWorkspace={onResetPath}
        visibleFields={activeFields}
        onToggleField={toggleActiveFields}
        isTerminalOpen={isTerminalOpen}
        onToggleTerminal={handleToggleTerminal}
      />
      <main className="content-viewport">
        {loading && (
          <div className="status-overlay">Loading folder tree state...</div>
        )}
        {error && <div className="status-overlay error">Error: {error}</div>}

        {!loading && !error && (
          <MillerColumns
            filePath={filePath}
            treeData={treeData}
            activePathIndices={activePathIndices}
            onSelectNode={handleSelectNode}
            visibleFields={activeFields}
            metadataMap={metadataMap}
          />
        )}
      </main>

      {isTerminalOpen && userName && hostname && (
        <TerminalView
          userName={userName}
          hostName={hostname}
          folderName={filePath.split("/").pop()}
        />
      )}
    </div>
  );
}
