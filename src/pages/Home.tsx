import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FileMetadata, DEFAULT_VISIBLE } from "../utils/viewFields";
import { Header } from "../components/Header";
import { MillerColumns } from "../components/MillerColumns";
import "../theme/colors.ts";
import "./Home.css";

interface FileNode {
  name: string;
  is_dir: boolean;
  children?: FileNode[] | null;
}

interface HomeProps {
  filePath: string;
  onResetPath: () => void;
}

export function Home({ filePath, onResetPath }: HomeProps) {
  const [treeData, setTreeData] = useState<FileNode[]>([]);
  const [activePathIndices, setActivePathIndices] = useState<number[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [metadataMap, setMetadataMap] = useState<Map<string, FileMetadata>>(
    new Map()
  );
  const [activeFields, setActiveFields] =
    useState<Set<keyof FileMetadata>>(DEFAULT_VISIBLE);

  const toggleActiveFields = (field: keyof FileMetadata) => {
    setActiveFields((prev) => {
      const next = new Set(prev);
      if (next.has(field)) next.delete(field);
      else next.add(field);
      return next;
    });
  };

  useEffect(() => {
    async function loadProject() {
      if (!filePath) return;
      try {
        setLoading(true);
        setError(null);

        await invoke("initialize_project", { rootPath: filePath });

        const tree: FileNode[] = await invoke("get_file_tree", {
          absoluteFolderPath: filePath,
        });
        setTreeData(tree);
        setActivePathIndices([]);

        // Fix: Query directly with root path string to bypass async stale state
        await fetchMetadataForNodes(tree, filePath);
      } catch (err: any) {
        setError(err.toString());
      } finally {
        setLoading(false);
      }
    }
    loadProject();
  }, [filePath]);

  const getFullPathFromIndices = (
    indices: number[],
    currentTree: FileNode[]
  ): string => {
    let segments: string[] = [filePath];
    let currentNodes = currentTree;

    for (const index of indices) {
      const targetNode = currentNodes[index];
      if (targetNode) {
        segments.push(String(targetNode.name));
        if (targetNode.children) {
          currentNodes = targetNode.children;
        }
      }
    }
    return segments.join("/");
  };

  // Fixed: Decoupled entirely from treeData state; processes directly via explicit strings
  const fetchMetadataForNodes = async (nodes: FileNode[], basePath: string) => {
    const results = new Map<string, FileMetadata>();

    for (let i = 0; i < nodes.length; i++) {
      const node = nodes[i];
      if (!node.is_dir) {
        try {
          const absolutePath = `${basePath}/${node.name}`;
          const meta = await invoke<FileMetadata>("get_file_metadata", {
            absoluteFilePath: absolutePath,
            rootPath: filePath,
          });
          results.set(absolutePath, meta);
        } catch {
          // Skip missing metadata records safely
        }
      }
    }

    setMetadataMap((prev) => new Map([...prev, ...results]));
  };

  const handleSelectNode = async (indices: number[], node: FileNode) => {
    setActivePathIndices(indices);
    if (node.is_dir && node.children) {
      const folderPath = getFullPathFromIndices(indices, treeData);
      await fetchMetadataForNodes(node.children, folderPath);
    }
  };

  return (
    <div className="home-layout">
      <Header
        onResetWorkspace={onResetPath}
        visibleFields={activeFields}
        onToggleField={toggleActiveFields}
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
    </div>
  );
}
