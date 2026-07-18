import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface LspHandle {
  root_uri: string;
  document_uri: string;
}

type Handler = (value: string) => void;

export class LspBridge {
  private handlers: Handler[] = [];
  private unlistenMessage: UnlistenFn | null = null;
  private unlistenClosed: UnlistenFn | null = null;

  handle: LspHandle | null = null;

  readonly transport = {
    send: (message: string) => {
      invoke("lsp_send", { message }).catch(() => {});
    },
    subscribe: (handler: Handler) => {
      this.handlers.push(handler);
    },
    unsubscribe: (handler: Handler) => {
      this.handlers = this.handlers.filter((existing) => existing !== handler);
    },
  };

  async start(path: string, onClosed?: () => void): Promise<LspHandle> {
    await this.stop();

    this.unlistenMessage = await listen<string>("lsp://message", (event) => {
      const message = this.filterDiagnostics(event.payload);
      for (const handler of this.handlers) handler(message);
    });

    this.unlistenClosed = await listen("lsp://closed", () => {
      onClosed?.();
    });

    this.handle = await invoke<LspHandle>("lsp_start", { path });
    return this.handle;
  }

  private filterDiagnostics(message: string): string {
    if (!message.includes("publishDiagnostics")) return message;
    try {
      const parsed = JSON.parse(message);
      if (parsed.method === "textDocument/publishDiagnostics") {
        parsed.params.diagnostics = parsed.params.diagnostics.filter(
          (diagnostic: { message: string }) =>
            !diagnostic.message.toLowerCase().includes("unknown font family"),
        );
        return JSON.stringify(parsed);
      }
    } catch {
      return message;
    }
    return message;
  }

  async stop() {
    this.unlistenMessage?.();
    this.unlistenClosed?.();
    this.unlistenMessage = null;
    this.unlistenClosed = null;
    this.handlers = [];
    this.handle = null;
    await invoke("lsp_stop").catch(() => {});
  }
}

export const lspAvailable = () => invoke<boolean>("lsp_running");
