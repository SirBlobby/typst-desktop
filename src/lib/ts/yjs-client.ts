import * as Y from "yjs";
import { WebsocketProvider } from "y-websocket";

export interface CollabSession {
  doc: Y.Doc;
  text: Y.Text;
  provider: WebsocketProvider;
}

function wsUrl(serverUrl: string) {
  return serverUrl
    .replace(/^https:/, "wss:")
    .replace(/^http:/, "ws:")
    .replace(/\/$/, "");
}

export function openCollabSession(
  serverUrl: string,
  deviceToken: string,
  roomId: string,
): CollabSession {
  const doc = new Y.Doc();
  const text = doc.getText("typst");

  const provider = new WebsocketProvider(`${wsUrl(serverUrl)}/yjs`, roomId, doc, {
    params: { token: deviceToken },
    disableBc: true,
  });

  return { doc, text, provider };
}

export function closeCollabSession(session: CollabSession | null) {
  if (!session) return;
  session.provider.disconnect();
  session.provider.destroy();
  session.doc.destroy();
}
