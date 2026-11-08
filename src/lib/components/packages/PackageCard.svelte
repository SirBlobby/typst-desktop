<script lang="ts">
  import Icon from "@iconify/svelte";
  import { untrack } from "svelte";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import type { LocalPackage } from "#lib/ts/local-packages";

  let { pack, onremove }: { pack: LocalPackage; onremove: (version: string) => void } = $props();

  let selectedVersion = $state(untrack(() => pack.versions[0]?.version ?? ""));
  let copied = $state(false);

  let current = $derived(pack.versions.find((entry) => entry.version === selectedVersion) ?? pack.versions[0]);

  $effect(() => {
    if (!pack.versions.some((entry) => entry.version === selectedVersion)) {
      selectedVersion = pack.versions[0]?.version ?? "";
    }
  });

  async function copyImport() {
    if (!current) return;
    await writeText(current.import_line);
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }
</script>

{#if current}
  <article class="flex flex-col gap-3 rounded-xl border border-[var(--color-line)] bg-[var(--color-surface)] p-4">
    <header class="flex items-start gap-3">
      <span class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-[var(--color-accent-soft)]">
        <Icon icon="ph:package" class="text-lg text-[var(--color-accent)]" />
      </span>
      <div class="min-w-0 flex-1">
        <h3 class="truncate text-sm font-semibold">{pack.name}</h3>
        <p class="line-clamp-2 text-xs text-[var(--color-ink-muted)]">{current.description || "No description"}</p>
      </div>
      {#if pack.versions.length > 1}
        <select bind:value={selectedVersion} aria-label="Version of {pack.name}" class="field field-sm">
          {#each pack.versions as entry (entry.version)}
            <option value={entry.version}>{entry.version}</option>
          {/each}
        </select>
      {:else}
        <span class="chip">{current.version}</span>
      {/if}
    </header>

    <div class="field flex items-center gap-2 p-0 text-xs">
      <code class="min-w-0 flex-1 truncate px-3 py-1.5 font-mono">{current.import_line}</code>
      <button class="icon-btn icon-btn-sm mr-1" onclick={copyImport} title="Copy import" aria-label="Copy import">
        <Icon icon={copied ? "ph:check" : "ph:copy"} />
      </button>
    </div>

    <footer class="flex items-center gap-2 text-[11px] text-[var(--color-ink-muted)]">
      <span class="min-w-0 flex-1 truncate">
        {[current.authors.join(", "), current.license].filter(Boolean).join(" · ")}
      </span>
      <button class="icon-btn icon-btn-sm" onclick={() => revealItemInDir(current.folder)} title="Show in folder" aria-label="Show in folder">
        <Icon icon="ph:folder-open" />
      </button>
      <button class="icon-btn icon-btn-sm" onclick={() => onremove(current.version)} title="Remove this version" aria-label="Remove version {current.version}">
        <Icon icon="ph:trash" />
      </button>
    </footer>
  </article>
{/if}
