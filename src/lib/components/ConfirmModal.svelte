<script lang="ts">
  import Modal from "./Modal.svelte";

  interface Props {
    title: string;
    message: string;
    confirmLabel?: string;
    onconfirm: () => void;
    onclose: () => void;
  }

  let {
    title,
    message,
    confirmLabel = "Delete",
    onconfirm,
    onclose,
  }: Props = $props();
</script>

<Modal {title} icon="ph:warning-circle" {onclose}>
  <p class="text-[13px] leading-relaxed text-[var(--color-ink-muted)]">{message}</p>

  {#snippet footer()}
    <button
      class="btn btn-ghost"
      onclick={onclose}
    >
      Cancel
    </button>
    <button
      class="btn btn-danger"
      onclick={onconfirm}
    >
      {confirmLabel}
    </button>
  {/snippet}
</Modal>
