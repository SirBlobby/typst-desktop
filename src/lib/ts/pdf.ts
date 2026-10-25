import * as pdfjs from "pdfjs-dist/legacy/build/pdf.mjs";
import workerUrl from "pdfjs-dist/legacy/build/pdf.worker.min.mjs?url";
import type { PDFDocumentProxy, RenderTask } from "pdfjs-dist";
import * as api from "#lib/ts/api";

pdfjs.GlobalWorkerOptions.workerSrc = workerUrl;

const POINTS_TO_PIXELS = 96 / 72;

export type PdfDocument = PDFDocumentProxy;

export interface PageSize {
  width: number;
  height: number;
}

export interface PageRenderOptions {
  document: PdfDocument;
  pageNumber: number;
  scale: number;
}

export async function openPdf(path: string): Promise<PdfDocument> {
  const bytes = await api.readPdf(path);
  return pdfjs.getDocument({ data: new Uint8Array(bytes) }).promise;
}

export async function pageSizes(document: PdfDocument): Promise<PageSize[]> {
  const pageNumbers = Array.from({ length: document.numPages }, (_, index) => index + 1);
  return Promise.all(
    pageNumbers.map(async (pageNumber) => {
      const page = await document.getPage(pageNumber);
      const viewport = page.getViewport({ scale: POINTS_TO_PIXELS });
      return { width: viewport.width, height: viewport.height };
    }),
  );
}

export function renderPdfPage(canvas: HTMLCanvasElement, initial: PageRenderOptions) {
  let options = initial;
  let visible = false;
  let renderedScale = 0;
  let renderTask: RenderTask | null = null;

  async function draw() {
    if (!visible || renderedScale === options.scale) return;

    const scale = options.scale;
    renderedScale = scale;
    renderTask?.cancel();

    const page = await options.document.getPage(options.pageNumber);
    if (renderedScale !== scale) return;

    const pixelRatio = window.devicePixelRatio || 1;
    const viewport = page.getViewport({ scale: scale * POINTS_TO_PIXELS * pixelRatio });
    canvas.width = Math.floor(viewport.width);
    canvas.height = Math.floor(viewport.height);

    const context = canvas.getContext("2d");
    if (!context) return;

    renderTask = page.render({ canvas, canvasContext: context, viewport });
    try {
      await renderTask.promise;
    } catch {
      return;
    }
  }

  const observer = new IntersectionObserver(
    (entries) => {
      visible = entries.some((entry) => entry.isIntersecting);
      draw();
    },
    { rootMargin: "800px 0px" },
  );
  observer.observe(canvas);

  return {
    update(next: PageRenderOptions) {
      if (next.document !== options.document || next.pageNumber !== options.pageNumber) {
        renderedScale = 0;
      }
      options = next;
      draw();
    },
    destroy() {
      observer.disconnect();
      renderTask?.cancel();
    },
  };
}
