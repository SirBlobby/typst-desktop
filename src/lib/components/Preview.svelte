<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { CompileResult, Diagnostic } from "$lib/ts/api";

  interface Props {
    compiled: CompileResult | null;
    diagnostics: Diagnostic[];
    compiling: boolean;
  }

  let { compiled, diagnostics, compiling }: Props = $props();

  let zoom = $state(1);

  const errors = $derived(diagnostics.filter((d) => d.severity === "error"));
</script>

<div class="flex h-full flex-col bg-[var(--color-surface-sunken)]">
  <div
    class="flex items-center gap-2 border-b border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-xs"
  >
    <Icon icon="ph:file-text" class="text-[var(--color-ink-muted)]" />
    <span class="text-[var(--color-ink-muted)]">
      {#if compiled}
        {compiled.stats.pages} pages, {compiled.stats.words} words
      {:else}
        Preview
      {/if}
    </span>

    <div class="flex-1"></div>

    {#if compiling}
      <Icon icon="ph:circle-notch" class="animate-spin text-[var(--color-accent)]" />
    {/if}

    <button
      class="rounded p-1 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]"
      onclick={() => (zoom = Math.max(0.4, zoom - 0.15))}
      aria-label="Zoom out"
    >
      <Icon icon="ph:minus" />
    </button>
    <span class="w-10 text-center tabular-nums text-[var(--color-ink-muted)]">
      {Math.round(zoom * 100)}%
    </span>
    <button
      class="rounded p-1 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]"
      onclick={() => (zoom = Math.min(2.5, zoom + 0.15))}
      aria-label="Zoom in"
    >
      <Icon icon="ph:plus" />
    </button>
  </div>

  {#if errors.length > 0}
    <div
      class="scroll-thin max-h-40 shrink-0 overflow-y-auto border-b border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2"
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

  <div class="scroll-thin flex-1 overflow-auto p-6">
    {#if compiled && compiled.pages.length > 0}
      <div
        class="mx-auto flex flex-col items-center gap-6"
        style="width: {Math.round(zoom * 100)}%; max-width: {zoom > 1 ? 'none' : '820px'};"
      >
        {#each compiled.pages as page}
          <div
            class="preview-page w-full overflow-hidden rounded bg-white shadow-lg ring-1 ring-black/5"
          >
            {@html page}
          </div>
        {/each}
      </div>
    {:else}
      <div
        class="flex h-full flex-col items-center justify-center gap-2 text-[var(--color-ink-muted)]"
      >
        <Icon icon="ph:file-dashed" class="text-4xl" />
        <p class="text-sm">
          {errors.length > 0 ? "Fix the errors above to see a preview" : "Nothing to preview yet"}
        </p>
      </div>
    {/if}
  </div>
</div>
