export interface ExportFormat {
  id: string;
  label: string;
  extension: string;
  icon: string;
  pandoc: boolean;
}

export const exportFormats: ExportFormat[] = [
  { id: "pdf", label: "PDF document", extension: "pdf", icon: "ph:file-pdf", pandoc: false },
  { id: "png", label: "PNG image", extension: "png", icon: "ph:image", pandoc: false },
  { id: "svg", label: "SVG graphics", extension: "svg", icon: "ph:file-svg", pandoc: false },
  { id: "html", label: "HTML (Typst)", extension: "html", icon: "ph:file-html", pandoc: false },
  { id: "typ", label: "Source file", extension: "typ", icon: "ph:file-code", pandoc: false },
  { id: "pandoc-docx", label: "Word", extension: "docx", icon: "ph:file-doc", pandoc: true },
  { id: "pandoc-latex", label: "LaTeX", extension: "tex", icon: "ph:file-text", pandoc: true },
  { id: "pandoc-markdown", label: "Markdown", extension: "md", icon: "ph:file-md", pandoc: true },
  { id: "pandoc-html", label: "HTML (Pandoc)", extension: "html", icon: "ph:file-html", pandoc: true },
];

export const pandocImportExtensions = ["docx", "tex", "md", "markdown", "html", "htm"];

export function exportFormat(id: string): ExportFormat | undefined {
  return exportFormats.find((format) => format.id === id);
}
