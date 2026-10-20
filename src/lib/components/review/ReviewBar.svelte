<script lang="ts">
  import Icon from "@iconify/svelte";
  import { app } from "$lib/ts/state.svelte";
  import { editorViewStore, reviewSessionStore } from "$lib/ts/review/session";
  import { historyOpenStore, reviewPanelOpenStore } from "$lib/ts/review/client";
  import { startCommentOnSelection } from "$lib/ts/review/comments";
  import { saveVersion } from "$lib/ts/review/versions";
  import CollaboratorAvatars from "./CollaboratorAvatars.svelte";
  import EditingModeSwitch from "./EditingModeSwitch.svelte";

  type SaveStatus = "idle" | "saving" | "saved" | "unchanged" | "error";

  const saveStatusLabels: Record<SaveStatus, string> = {
    idle: "Save version",
    saving: "Saving...",
    saved: "Version saved",
    unchanged: "No changes",
    error: "Save failed",
  };

  let saveStatus = $state<SaveStatus>("idle");
  let saveStatusTimer: ReturnType<typeof setTimeout> | undefined;

  let canEdit = $derived($reviewSessionStore?.role === "owner" || $reviewSessionStore?.role === "editor");
  let canComment = $derived(!!$reviewSessionStore && $reviewSessionStore.role !== "viewer");

  async function handleSaveVersion() {
    const session = $reviewSessionStore;
    if (!session || saveStatus === "saving") return;
    saveStatus = "saving";
    try {
      const result = await saveVersion(session.documentId);
      saveStatus = result.created ? "saved" : "unchanged";
    } catch {
      saveStatus = "error";
    }
    clearTimeout(saveStatusTimer);
    saveStatusTimer = setTimeout(() => (saveStatus = "idle"), 2500);
  }
</script>

{#snippet barButton(icon: string, label: string, run: () => void, active = false)}
  <button
    class="flex items-center gap-1.5 rounded-md px-2 py-1 text-sm transition {active
      ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]'
      : 'text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]'}"
    title={label}
    aria-label={label}
    onclick={run}
  >
    <Icon {icon} class="text-base" />
  </button>
{/snippet}

{#if $reviewSessionStore}
  <div class="ml-auto flex items-center gap-2.5">
    {#if app.collab}
      <CollaboratorAvatars awareness={app.collab.provider.awareness} />
    {/if}
    {#if canComment}
      <EditingModeSwitch {canEdit} />
      {@render barButton("mdi:comment-plus-outline", "Comment on selection (Ctrl+Alt+M)", () =>
        startCommentOnSelection($editorViewStore, $reviewSessionStore?.text ?? null),
      )}
    {/if}
    {#if canEdit}
      <button
        class="flex items-center gap-1.5 rounded-md px-2 py-1 text-sm transition {saveStatus === 'error'
          ? 'text-[var(--color-danger)]'
          : 'text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]'}"
        title="Save a version of this document"
        disabled={saveStatus === "saving"}
        onclick={handleSaveVersion}
      >
        <Icon icon={saveStatus === "saving" ? "mdi:loading" : "mdi:content-save-outline"} class="text-base {saveStatus === 'saving' ? 'animate-spin' : ''}" />
        {saveStatusLabels[saveStatus]}
      </button>
    {/if}
    {@render barButton("mdi:history", "Version history", () => historyOpenStore.set(true))}
    {@render barButton("mdi:comment-text-multiple-outline", "Comments and suggestions", () => reviewPanelOpenStore.update((open) => !open), $reviewPanelOpenStore)}
  </div>
{/if}
