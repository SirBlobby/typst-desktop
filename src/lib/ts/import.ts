import { open } from "@tauri-apps/plugin-dialog";

export const IMAGE_EXTENSIONS = ["png", "jpg", "jpeg", "gif", "svg", "webp"];
export const FONT_EXTENSIONS = ["ttf", "otf", "ttc", "otc"];
export const DATA_EXTENSIONS = ["bib", "csl", "json", "yaml", "yml", "csv", "toml"];

export type PickKind = "all" | "assets" | "images" | "fonts";

export async function pickFiles(kind: PickKind = "all"): Promise<string[]> {
  const filters =
    kind === "images"
      ? [{ name: "Images", extensions: IMAGE_EXTENSIONS }]
      : kind === "fonts"
        ? [{ name: "Fonts", extensions: FONT_EXTENSIONS }]
        : kind === "assets"
          ? [
              {
                name: "Images and fonts",
                extensions: [...IMAGE_EXTENSIONS, ...FONT_EXTENSIONS],
              },
              { name: "Images", extensions: IMAGE_EXTENSIONS },
              { name: "Fonts", extensions: FONT_EXTENSIONS },
            ]
          : [
              {
                name: "Typst files",
                extensions: ["typ", ...DATA_EXTENSIONS, ...IMAGE_EXTENSIONS, ...FONT_EXTENSIONS],
              },
            ];

  const selected = await open({ multiple: true, filters });

  if (!selected) return [];
  return Array.isArray(selected) ? selected : [selected];
}
