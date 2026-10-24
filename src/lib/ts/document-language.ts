export type DocumentLanguage = "typst" | "latex";

export function isLatexPath(path: string) {
  return path.toLowerCase().endsWith(".tex");
}

export function sourceExtension(language: DocumentLanguage) {
  return language === "latex" ? ".tex" : ".typ";
}

export function languageLabel(language: DocumentLanguage) {
  return language === "latex" ? "LaTeX" : "Typst";
}
