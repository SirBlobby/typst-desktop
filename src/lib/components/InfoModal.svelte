<script lang="ts">
  import Icon from "@iconify/svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import Modal from "./Modal.svelte";
  import * as api from "$lib/ts/api";
  import type { AppInfo } from "$lib/ts/api";

  interface Props {
    onclose: () => void;
  }

  let { onclose }: Props = $props();

  let info = $state<AppInfo | null>(null);

  $effect(() => {
    api
      .appInfo()
      .then((result) => (info = result))
      .catch(() => (info = null));
  });

  const links = [
    {
      label: "Typst Documentation",
      url: "https://typst.app/docs/",
      icon: "ph:book-open",
    },
    {
      label: "Typst Universe",
      url: "https://typst.app/universe/",
      icon: "ph:planet",
    },
  ];
</script>

<Modal title="About Typst Desktop" icon="ph:info" {onclose}>
  <div class="flex flex-col gap-4">
    <div class="flex items-center gap-3">
      <Icon icon="ph:file-code" class="text-3xl text-[var(--color-accent)]" />
      <div>
        <p class="text-sm font-semibold">Typst Desktop</p>
        <p class="text-xs text-[var(--color-ink-muted)]">
          A local editor for Typst documents
        </p>
      </div>
    </div>

    {#if info}
      <dl class="flex flex-col gap-1 text-xs">
        <div class="flex justify-between border-b border-[var(--color-line)] py-1.5">
          <dt class="text-[var(--color-ink-muted)]">Version</dt>
          <dd class="font-medium">{info.version}</dd>
        </div>
        <div class="flex justify-between border-b border-[var(--color-line)] py-1.5">
          <dt class="text-[var(--color-ink-muted)]">Typst</dt>
          <dd class="font-medium">{info.typst_version}</dd>
        </div>
        <div class="flex justify-between border-b border-[var(--color-line)] py-1.5">
          <dt class="text-[var(--color-ink-muted)]">Tauri</dt>
          <dd class="font-medium">{info.tauri_version}</dd>
        </div>
        <div class="flex justify-between border-b border-[var(--color-line)] py-1.5">
          <dt class="text-[var(--color-ink-muted)]">Author</dt>
          <dd class="font-medium">{info.authors}</dd>
        </div>
        <div class="flex justify-between py-1.5">
          <dt class="text-[var(--color-ink-muted)]">License</dt>
          <dd class="font-medium">{info.license}</dd>
        </div>
      </dl>
    {/if}

    <div class="flex flex-col gap-1.5">
      {#each links as link}
        <button
          class="flex items-center gap-2 rounded-md border border-[var(--color-line)] px-3 py-2 text-xs transition hover:border-[var(--color-accent)] hover:bg-[var(--color-surface-muted)]"
          onclick={() => openUrl(link.url)}
        >
          <Icon icon={link.icon} class="text-base text-[var(--color-accent)]" />
          <span class="flex-1 text-left">{link.label}</span>
          <Icon
            icon="ph:arrow-square-out"
            class="text-[var(--color-ink-muted)]"
          />
        </button>
      {/each}
    </div>

    <p
      class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-3 text-[11px] leading-relaxed text-[var(--color-ink-muted)]"
    >
      Typst Desktop is a community application and is not affiliated with,
      endorsed by, or supported by the official Typst project. The links above
      open the official Typst website in your browser.
    </p>
  </div>

  {#snippet footer()}
    <button
      class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90"
      onclick={onclose}
    >
      Close
    </button>
  {/snippet}
</Modal>
