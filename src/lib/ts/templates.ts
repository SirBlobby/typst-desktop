import { invoke } from "@tauri-apps/api/core";

export interface UniverseTemplate {
  name: string;
  version: string;
  description: string;
  authors: string[];
  categories: string[];
  disciplines: string[];
  keywords: string[];
  thumbnail_url: string | null;
  updated_at: number | null;
}

export interface TemplateAuthor {
  name: string;
  affiliation?: string;
}

export interface CreatedEntry {
  kind: "document" | "project";
  path: string;
  authors_filled: boolean | null;
}

export type TemplateSort = "name" | "recent";

const SAVED_AUTHORS_KEY = "typst-desktop-template-authors";

const categoryLabels: Record<string, string> = {
  cv: "CV",
  paper: "Paper",
  thesis: "Thesis",
  report: "Report",
  presentation: "Presentation",
  poster: "Poster",
  flyer: "Flyer",
  book: "Book",
  office: "Office",
  fun: "Fun",
};

export const fetchTemplates = () => invoke<UniverseTemplate[]>("list_templates");

export const createFromTemplate = (
  template: UniverseTemplate,
  title: string,
  parent: string,
  authors: TemplateAuthor[],
) =>
  invoke<CreatedEntry>("create_from_template", {
    request: { name: template.name, version: template.version, title, parent, authors },
  });

export function universeUrl(template: UniverseTemplate) {
  return `https://typst.app/universe/package/${encodeURIComponent(template.name)}`;
}

export function categoryLabel(category: string) {
  return categoryLabels[category] ?? category.charAt(0).toUpperCase() + category.slice(1).replace(/-/g, " ");
}

export function readableName(name: string) {
  return name
    .split("-")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}

export function matchesQuery(template: UniverseTemplate, query: string) {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  if (words.length === 0) return true;
  const haystack = [
    template.name,
    template.description,
    ...template.keywords,
    ...template.authors,
    ...template.categories,
    ...template.disciplines,
  ]
    .join(" ")
    .toLowerCase();
  return words.every((word) => haystack.includes(word));
}

export function sortTemplates(templates: UniverseTemplate[], sort: TemplateSort) {
  const sorted = [...templates];
  if (sort === "recent") sorted.sort((left, right) => (right.updated_at ?? 0) - (left.updated_at ?? 0));
  else sorted.sort((left, right) => left.name.localeCompare(right.name));
  return sorted;
}

export function categoryCounts(templates: UniverseTemplate[]) {
  const counts = new Map<string, number>();
  for (const template of templates) {
    for (const category of template.categories) counts.set(category, (counts.get(category) ?? 0) + 1);
  }
  return [...counts.entries()].sort((left, right) => right[1] - left[1]);
}

export function loadSavedAuthors(): TemplateAuthor[] {
  try {
    const saved = JSON.parse(localStorage.getItem(SAVED_AUTHORS_KEY) ?? "[]");
    return Array.isArray(saved) ? saved.filter((author) => typeof author?.name === "string") : [];
  } catch {
    return [];
  }
}

export function saveAuthors(authors: TemplateAuthor[]) {
  try {
    localStorage.setItem(SAVED_AUTHORS_KEY, JSON.stringify(authors));
  } catch {}
}
