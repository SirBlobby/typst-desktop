import { safeSvg } from "./safe-svg";

export interface PageSize {
  width: number;
  height: number;
}

export interface PreparedPage {
  svg: string;
  size: PageSize | null;
}

const PIXELS_PER_UNIT: Record<string, number> = { pt: 96 / 72, px: 1, "": 1 };

export function pageSize(svg: string): PageSize | null {
  const rootTag = svg.slice(0, svg.indexOf(">") + 1);
  const widthMatch = /\swidth="([\d.]+)(pt|px)?"/.exec(rootTag);
  const heightMatch = /\sheight="([\d.]+)(pt|px)?"/.exec(rootTag);
  if (!widthMatch || !heightMatch) return null;
  const width = parseFloat(widthMatch[1]) * PIXELS_PER_UNIT[widthMatch[2] ?? ""];
  const height = parseFloat(heightMatch[1]) * PIXELS_PER_UNIT[heightMatch[2] ?? ""];
  return width > 0 && height > 0 ? { width, height } : null;
}

export class PreviewPageCache {
  private prepared = new Map<string, PreparedPage>();

  prepare(rawPages: string[]): PreparedPage[] {
    const nextPrepared = new Map<string, PreparedPage>();
    const pages = rawPages.map((raw) => {
      let page = nextPrepared.get(raw) ?? this.prepared.get(raw);
      if (!page) {
        const svg = safeSvg(raw);
        page = { svg, size: pageSize(svg) };
      }
      nextPrepared.set(raw, page);
      return page;
    });
    this.prepared = nextPrepared;
    return pages;
  }
}
