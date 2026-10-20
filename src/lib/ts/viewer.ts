import * as api from "$lib/ts/api";

export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

export function viewableFiles(paths: string[]): string[] {
  return paths.filter(api.isViewablePath);
}
