import { invoke } from "@tauri-apps/api/core";

export interface LocalPackageInfo {
  name: string;
  version: string;
  import_line: string;
  installed_versions: string[];
}

export interface LocalPackageVersion {
  version: string;
  description: string | null;
  authors: string[];
  license: string | null;
  import_line: string;
  folder: string;
}

export interface LocalPackage {
  name: string;
  versions: LocalPackageVersion[];
}

export const listLocalPackages = () => invoke<LocalPackage[]>("list_local_packages");

export const localPackagesFolder = () => invoke<string>("local_packages_folder");

export function matchesPackageQuery(pack: LocalPackage, query: string) {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  if (words.length === 0) return true;
  const latest = pack.versions[0];
  const haystack = [pack.name, latest?.description ?? "", ...(latest?.authors ?? [])].join(" ").toLowerCase();
  return words.every((word) => haystack.includes(word));
}

export const localPackageInfo = (path: string) =>
  invoke<LocalPackageInfo>("local_package_info", { path });

export const installLocalPackage = (path: string) =>
  invoke<LocalPackageInfo>("install_local_package", { path });

export const removeLocalPackage = (name: string, version: string) =>
  invoke<string[]>("remove_local_package", { name, version });
