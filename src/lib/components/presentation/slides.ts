import type { SpeakerNote } from "#lib/ts/api";
import { pageSize } from "#lib/ts/preview-pages";
import { safeSvg } from "#lib/ts/safe-svg";

export interface Slide {
  markup: string;
  aspectRatio: number;
  note: SpeakerNote | null;
}

export const defaultAspectRatio = 16 / 9;

export function collectSlides(pages: string[], notes: (SpeakerNote | null)[] = []): Slide[] {
  return pages.map((page, index) => {
    const markup = safeSvg(page);
    const size = pageSize(markup);
    const aspectRatio = size ? size.width / size.height : defaultAspectRatio;
    return { markup, aspectRatio, note: notes[index] ?? null };
  });
}

function twoDigits(value: number) {
  return value.toString().padStart(2, "0");
}

export function formatElapsed(totalSeconds: number) {
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  return hours > 0
    ? `${hours}:${twoDigits(minutes)}:${twoDigits(seconds)}`
    : `${twoDigits(minutes)}:${twoDigits(seconds)}`;
}
