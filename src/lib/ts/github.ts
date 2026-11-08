import { invoke } from "@tauri-apps/api/core";

export type GitResolution = "ask" | "keep_repository" | "keep_mine";

export interface GithubAccount {
  username: string;
}

export interface GitLink {
  repo_url: string;
  branch: string;
  folder: string;
  has_token: boolean;
  account_username: string | null;
  excluded_paths: string[];
  base_commit: string | null;
  last_synced_at: string | null;
}

export interface GitLinkRequest {
  repo_url: string;
  branch: string;
  folder: string;
  token: string;
  excluded_paths: string[];
  replace_project_files: boolean;
}

export interface GitPreviewFile {
  path: string;
  size: number | null;
}

export interface GitImportPreview {
  commit: string;
  files: GitPreviewFile[];
  too_large: GitPreviewFile[];
  ignored_count: number;
  total_bytes: number;
  replaced: string[];
  removed: string[];
}

export interface GitChange {
  path: string;
  change: "added" | "modified" | "deleted";
}

export interface GitChanges {
  link: GitLink;
  head: string;
  behind: boolean;
  local: GitChange[];
  remote: GitChange[];
  conflicts: string[];
}

export interface GitPullResult {
  link: GitLink;
  updated: string[];
  removed: string[];
  conflicts: string[];
  too_large: string[];
  applied: boolean;
}

export interface GitPushResult {
  link: GitLink;
  commit: string;
  pushed: string[];
}

export interface GitLinkUpdate {
  token?: string;
  use_account?: boolean;
  excluded_paths?: string[];
}

export const TOKEN_PAGE_URL = "https://github.com/settings/personal-access-tokens/new";

export const githubAccount = () => invoke<GithubAccount | null>("github_account");
export const connectGithub = (token: string) => invoke<GithubAccount>("github_connect", { token });
export const disconnectGithub = () => invoke<void>("github_disconnect");

export const gitLinkStatus = (path: string) => invoke<GitLink | null>("git_link_status", { path });
export const gitLinkedProjects = () => invoke<string[]>("git_linked_projects");
export const previewGitLink = (path: string, request: GitLinkRequest) =>
  invoke<GitImportPreview>("git_preview_link", { path, request });
export const createGitLink = (path: string, request: GitLinkRequest) =>
  invoke<GitLink>("git_create_link", { path, request });
export const updateGitLink = (path: string, update: GitLinkUpdate) =>
  invoke<GitLink>("git_update_link", { path, update });
export const deleteGitLink = (path: string) => invoke<void>("git_delete_link", { path });
export const gitChanges = (path: string) => invoke<GitChanges>("git_changes", { path });
export const pullGit = (path: string, resolution: GitResolution = "ask") =>
  invoke<GitPullResult>("git_pull", { path, resolution });
export const pushGit = (path: string, message: string, paths: string[]) =>
  invoke<GitPushResult>("git_push", { path, message, paths });

export const previewGitImport = (request: GitLinkRequest) =>
  invoke<GitImportPreview>("git_preview_import", { request });
export const importGitProject = (parent: string, name: string, request: GitLinkRequest) =>
  invoke<string>("git_import_project", { parent, name, request });

export function shortCommit(commit: string | null): string {
  return commit ? commit.slice(0, 7) : "";
}

export function repositoryName(repoUrl: string): string {
  const segments = repoUrl.trim().replace(/\/+$/, "").replace(/\.git$/, "").split("/");
  return segments.length > 4 ? segments[segments.length - 1] : "";
}

export function isGithubComUrl(repoUrl: string): boolean {
  try {
    const host = new URL(repoUrl.trim()).host.toLowerCase();
    return host === "github.com" || host === "www.github.com";
  } catch {
    return false;
  }
}

export function parseExcludedPaths(text: string): string[] {
  return text
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}
