import { invoke } from "@tauri-apps/api/core";

export async function initializeProject(path: string): Promise<string> {
    return await invoke<string>("initialize_project", { path });
}

export async function listAssetFiles(path: string): Promise<string[]> {
    return await invoke<string[]>("list_asset_files", { path });
}

export async function stageCommitTag(
    path: string,
    filePath: string,
    summary: string,
    tag: string
): Promise<string> {
    return await invoke<string>("stage_commit_tag", { path, filePath, summary, tag });
}