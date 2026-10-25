<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { CompileResult, Diagnostic } from "#lib/ts/api";

  interface Props {
    compiled: CompileResult | null;
    diagnostics: Diagnostic[];
    compiling: boolean;
  }

  let { compiled, diagnostics, compiling }: Props = $props();

  let zoom = $state(1);

  const errors = $derived(diagnostics.filter((d) => d.severity === "error"));
</script>

<div class="flex h-full flex-col bg-[var(--color-canvas)]">
  <div
    class="bar gap-2 pl-3 pr-1.5 text-xs"
  >
    <Icon icon="ph:eye" class="text-sm text-[var(--color-ink-subtle)]" />
    <span class="text-[var(--color-ink-muted)]">
      {#if compiled}
        {compiled.stats.pages} {compiled.stats.pages === 1 ? "page" : "pages"} · {compiled.stats.words} {compiled.stats.words === 1 ? "word" : "words"}
      {:else}
        Preview
      {/if}
    </span>

    <div class="flex-1"></div>

    {#if compiling}
      <Icon icon="ph:circle-notch" class="animate-spin text-[var(--color-accent)]" />
    {/if}

    <button
      class="icon-btn icon-btn-sm"
      onclick={() => (zoom = Math.max(0.4, zoom - 0.15))}
      aria-label="Zoom out"
    >
      <Icon icon="ph:minus" />
    </button>
    <span class="w-10 text-center text-[11px] tabular-nums text-[var(--color-ink-muted)]">
      {Math.round(zoom * 100)}%
    </span>
    <button
      class="icon-btn icon-btn-sm"
      onclick={() => (zoom = Math.min(2.5, zoom + 0.15))}
      aria-label="Zoom in"
    >
      <Icon icon="ph:plus" />
    </button>
  </div>

  {#if errors.length > 0}
    <div
      class="scroll-thin max-h-40 shrink-0 overflow-y-auto border-b border-[var(--color-danger)]/20 bg-[var(--color-danger)]/6 px-3 py-2"
    >
      {#each errors as diagnostic}
        <div class="flex items-start gap-2 py-1 text-xs text-[var(--color-danger)]">
          <Icon icon="ph:warning-circle" class="mt-0.5 shrink-0" />
          <span>
            {#if diagnostic.line}
              <span class="font-medium">Line {diagnostic.line}:</span>
            {/if}
            {diagnostic.message}
          </span>
        </div>
      {/each}
    </div>
  {/if}

  <div class="scroll-thin flex-1 overflow-auto px-8 py-6">
    {#if compiled && compiled.pages.length > 0}
      <div
        class="mx-auto flex flex-col items-center gap-6"
        style="width: {Math.round(zoom * 100)}%; max-width: {zoom > 1 ? 'none' : '820px'};"
      >
        {#each compiled.pages as page}
          <div
            class="preview-page w-full overflow-hidden rounded-sm bg-white shadow-lg ring-1 ring-black/5"
          >
            {@html page}
          </div>
        {/each}
      </div>
    {:else}
      <div
        class="empty-state h-full"
      >
        <Icon icon="ph:file-dashed" class="text-3xl" />
        <p class="text-sm">
          {errors.length > 0 ? "Fix the errors above to see a preview" : "Nothing to preview yet"}
        </p>
      </div>
    {/if}
  </div>
</div>
