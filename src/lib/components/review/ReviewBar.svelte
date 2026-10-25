<script lang="ts">
  import Icon from "@iconify/svelte";
  import { app } from "#lib/ts/state.svelte";
  import { editorViewStore, reviewSessionStore } from "#lib/ts/review/session";
  import { historyOpenStore, reviewPanelOpenStore } from "#lib/ts/review/client";
  import { startCommentOnSelection } from "#lib/ts/review/comments";
  import { saveVersion } from "#lib/ts/review/versions";
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
    class="icon-btn {active ? 'icon-btn-active' : ''}"
    title={label}
    aria-label={label}
    onclick={run}
  >
    <Icon {icon} class="text-sm" />
  </button>
{/snippet}

{#if $reviewSessionStore}
  <div class="ml-auto flex items-center gap-1">
    {#if app.collab}
      <CollaboratorAvatars awareness={app.collab.provider.awareness} />
    {/if}
    {#if canComment}
      <EditingModeSwitch {canEdit} />
      {@render barButton("ph:chat-circle-text", "Comment on selection (Ctrl+Alt+M)", () =>
        startCommentOnSelection($editorViewStore, $reviewSessionStore?.text ?? null),
      )}
    {/if}
    {#if canEdit}
      <button
        class="btn btn-ghost {saveStatus === 'error' ? 'text-[var(--color-danger)]' : ''}"
        title="Save a version of this document"
        disabled={saveStatus === "saving"}
        onclick={handleSaveVersion}
      >
        <Icon icon={saveStatus === "saving" ? "ph:circle-notch" : "ph:floppy-disk"} class="text-sm {saveStatus === 'saving' ? 'animate-spin' : ''}" />
        {saveStatusLabels[saveStatus]}
      </button>
    {/if}
    {@render barButton("ph:clock-counter-clockwise", "Version history", () => historyOpenStore.set(true))}
    {@render barButton("ph:chats", "Comments and suggestions", () => reviewPanelOpenStore.update((open) => !open), $reviewPanelOpenStore)}
  </div>
{/if}
