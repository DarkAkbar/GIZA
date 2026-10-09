import { moveFilesToTrash, showItemInFolder } from "./tauri";

export async function copyPathToClipboard(path: string): Promise<void> {
  await navigator.clipboard.writeText(path);
}

export async function trashSingleFile(path: string) {
  return moveFilesToTrash([path]);
}

export async function revealFileInExplorer(path: string) {
  return showItemInFolder(path);
}
