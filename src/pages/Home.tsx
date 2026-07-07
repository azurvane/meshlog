import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Header } from "../components/Header";
import { MillerColumns } from "../components/MillerColumns";
import "../theme/colors.ts";
import "./Home.css";

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

interface HomeProps {
  filePath: string;
  onResetPath: () => void;
}

export function Home({ filePath, onResetPath }: HomeProps) {
  const [treeData, setTreeData] = useState<FileNode[]>([]);
  const [activePathIndices, setActivePathIndices] = useState<number[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function loadProject() {
      if (!filePath) return;
      try {
        setLoading(true);
        setError(null);

        // Initialize git environment & SQLite repository tables
        await invoke("initialize_project", { rootPath: filePath });

        // Fetch structural tree hierarchy dynamically
        const tree: FileNode[] = await invoke("get_file_tree", {
          absoluteFolderPath: filePath,
        });
        setTreeData(tree);
        setActivePathIndices([]);
      } catch (err: any) {
        setError(err.toString());
      } finally {
        setLoading(false);
      }
    }
    loadProject();
  }, [filePath]);

  // Recursively step along indices arrays to build proper systemic directory strings
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

  const handleSelectNode = async (indices: number[], node: FileNode) => {
    setActivePathIndices(indices);
  };

  return (
    <div className="home-layout">
      {/* Pass the clear path callback right up to the header's setting actions if requested */}
      <Header onResetWorkspace={onResetPath} />
      <main className="content-viewport">
        {loading && (
          <div className="status-overlay">Loading folder tree state...</div>
        )}
        {error && <div className="status-overlay error">Error: {error}</div>}

        {!loading && !error && (
          <MillerColumns
            treeData={treeData}
            activePathIndices={activePathIndices}
            onSelectNode={handleSelectNode}
          />
        )}
      </main>
    </div>
  );
}
