export type SortField = "name" | "modified" | "size";
export type SortDirection = "asc" | "desc";
export type ViewMode = "grid" | "list";

export interface ViewFilters {
  sortField: SortField;
  sortDirection: SortDirection;
}

export interface SortOption {
  value: SortField;
  label: string;
}

export interface SortKeys {
  name: string;
  modified: string | null;
  size?: number;
}

export const defaultViewFilters: ViewFilters = {
  sortField: "name",
  sortDirection: "asc",
};

export const allSortOptions: SortOption[] = [
  { value: "name", label: "Name" },
  { value: "modified", label: "Last modified" },
  { value: "size", label: "Size" },
];

export function parseTimestamp(value: string | null) {
  if (!value) return 0;
  const normalized =
    value.includes("T") || value.endsWith("Z") ? value : value.replace(" ", "T") + "Z";
  const time = Date.parse(normalized);
  return Number.isNaN(time) ? 0 : time;
}

export function matchesQuery(name: string, query: string) {
  const normalized = query.trim().toLowerCase();
  return !normalized || name.toLowerCase().includes(normalized);
}

export function sortByFilters<T>(items: T[], filters: ViewFilters, keysOf: (item: T) => SortKeys): T[] {
  const direction = filters.sortDirection === "asc" ? 1 : -1;
  return [...items].sort((firstItem, secondItem) => {
    const first = keysOf(firstItem);
    const second = keysOf(secondItem);
    let difference = 0;
    if (filters.sortField === "modified") {
      difference = parseTimestamp(first.modified) - parseTimestamp(second.modified);
    } else if (filters.sortField === "size") {
      difference = (first.size ?? 0) - (second.size ?? 0);
    }
    if (difference === 0) {
      difference = first.name.localeCompare(second.name, undefined, {
        numeric: true,
        sensitivity: "base",
      });
    }
    return difference * direction;
  });
}

export function readStoredPreference<T>(key: string, fallback: T): T {
  try {
    const stored = localStorage.getItem(key);
    return stored ? (JSON.parse(stored) as T) : fallback;
  } catch {
    return fallback;
  }
}

export function writeStoredPreference(key: string, value: unknown) {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {}
}
