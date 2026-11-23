import { TauriEvent, emit, listen, type UnlistenFn } from "@tauri-apps/api/event";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Point, Stroke } from "../components/presentation/drawing";
import type { Slide } from "../components/presentation/slides";

export type { Point, Slide };

export type PresentationLayout = "audience" | "speaker";

export interface AudienceState {
  index: number;
  black: boolean;
  strokes: Stroke[];
}

export interface ForwardedKey {
  key: string;
  shiftKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
}

export type SpeakerMessage =
  | { type: "slides"; slides: Slide[] }
  | { type: "state"; state: AudienceState }
  | { type: "laser"; point: Point | null }
  | { type: "end" };

export type AudienceMessage = { type: "hello" } | { type: "key"; key: ForwardedKey };

interface Envelope<T> {
  session: string;
  message: T;
}

const AUDIENCE_WINDOW_LABEL = "audience";
const SPEAKER_EVENT = "presentation-speaker";
const AUDIENCE_EVENT = "presentation-audience";

function sendTo<T>(event: string, session: string, message: T) {
  emit<Envelope<T>>(event, { session, message }).catch(() => {});
}

function listenFor<T>(event: string, session: string, handler: (message: T) => void): Promise<UnlistenFn> {
  return listen<Envelope<T>>(event, ({ payload }) => {
    if (payload.session === session) handler(payload.message);
  });
}

export function sendToAudience(session: string, message: SpeakerMessage) {
  sendTo(SPEAKER_EVENT, session, message);
}

export function sendToSpeaker(session: string, message: AudienceMessage) {
  sendTo(AUDIENCE_EVENT, session, message);
}

export function listenToSpeaker(session: string, handler: (message: SpeakerMessage) => void) {
  return listenFor(SPEAKER_EVENT, session, handler);
}

export function listenToAudience(session: string, handler: (message: AudienceMessage) => void) {
  return listenFor(AUDIENCE_EVENT, session, handler);
}

export async function openAudienceWindow(session: string, onclosed: () => void): Promise<UnlistenFn> {
  const existing = await WebviewWindow.getByLabel(AUDIENCE_WINDOW_LABEL);
  if (existing) {
    await existing.setFocus();
    return existing.once(TauriEvent.WINDOW_DESTROYED, onclosed);
  }
  const audienceWindow = new WebviewWindow(AUDIENCE_WINDOW_LABEL, {
    url: `/present?session=${encodeURIComponent(session)}`,
    title: "Presentation",
    width: 1280,
    height: 720,
  });
  return audienceWindow.once(TauriEvent.WINDOW_DESTROYED, onclosed);
}

export async function closeAudienceWindow() {
  const existing = await WebviewWindow.getByLabel(AUDIENCE_WINDOW_LABEL);
  await existing?.close();
}

export function setWindowFullscreen(fullscreen: boolean) {
  return getCurrentWindow().setFullscreen(fullscreen).catch(() => {});
}

export function forwardedKey(event: KeyboardEvent): ForwardedKey {
  const { key, shiftKey, ctrlKey, metaKey, altKey } = event;
  return { key, shiftKey, ctrlKey, metaKey, altKey };
}

export function audienceSlides(slides: Slide[]): Slide[] {
  return slides.map(({ markup, aspectRatio }) => ({ markup, aspectRatio, note: null }));
}
