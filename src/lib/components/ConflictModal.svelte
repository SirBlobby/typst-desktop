<script lang="ts">
  import Icon from "@iconify/svelte";
  import Modal from "./Modal.svelte";
  import type { Conflict, Resolution } from "$lib/ts/api";

  interface Props {
    conflicts: Conflict[];
    onresolve: (resolutions: Resolution[]) => void;
    onclose: () => void;
  }

  let { conflicts, onresolve, onclose }: Props = $props();

  let index = $state(0);
  let choices = $state<string[]>([]);
  let mode = $state<("merged" | "local" | "remote")[]>([]);

  $effect(() => {
    choices = conflicts.map((conflict) =>
      conflict.binary ? conflict.remote_text : conflict.merged_text,
    );
    mode = conflicts.map(() => "merged");
    index = 0;
  });

  const current = $derived(conflicts[index]);

  function choose(option: "merged" | "local" | "remote") {
    mode[index] = option;
    choices[index] =
      option === "local"
        ? current.local_text
        : option === "remote"
          ? current.remote_text
          : current.merged_text;
  }

  const unresolvedMarkers = $derived(
    choices[index]?.includes("<<<<<<<") ?? false,
  );

  const anyUnresolved = $derived(
    choices.some((text) => text.includes("<<<<<<<")),
  );

  function submit() {
    onresolve(
      conflicts.map((conflict, position) => ({
        path: conflict.path,
        content: choices[position],
        server_hash: conflict.server_hash,
      })),
    );
  }
</script>

<Modal
  title="Resolve sync conflicts"
  icon="ph:git-merge"
  width="max-w-4xl"
  {onclose}
>
  <div class="flex flex-col gap-4">
    <p class="text-xs text-[var(--color-ink-muted)]">
      These files changed both on this device and in the cloud. Pick a version or
      edit the merged result, then save to upload your resolution.
    </p>

    <div class="flex flex-wrap gap-1.5">
      {#each conflicts as conflict, position}
        <button
          class="flex items-center gap-1.5 rounded-md border px-2.5 py-1 text-xs transition
            {position === index
            ? 'border-[var(--color-accent)] bg-[var(--color-accent-soft)] text-[var(--color-accent)]'
            : 'border-[var(--color-line)] text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-muted)]'}"
          onclick={() => (index = position)}
        >
          {#if choices[position].includes("<<<<<<<")}
            <Icon icon="ph:warning" class="text-[var(--color-danger)]" />
          {:else}
            <Icon icon="ph:check-circle" class="text-[var(--color-success)]" />
          {/if}
          {conflict.path}
        </button>
      {/each}
    </div>

    {#if current}
      {#if current.binary}
        <div
          class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-4 text-xs"
        >
          <p class="font-medium">{current.path}</p>
          <p class="mt-1 text-[var(--color-ink-muted)]">
            This is a binary file and cannot be merged automatically. The cloud
            version will be kept.
          </p>
        </div>
      {:else}
        <div class="flex items-center gap-1.5">
          {#each [["merged", "Merged"], ["local", "This device"], ["remote", "Cloud"]] as [option, label]}
            <button
              class="rounded-md border px-3 py-1.5 text-xs transition
                {mode[index] === option
                ? 'border-[var(--color-accent)] bg-[var(--color-accent-soft)] text-[var(--color-accent)]'
                : 'border-[var(--color-line)] text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-muted)]'}"
              onclick={() => choose(option as "merged" | "local" | "remote")}
            >
              {label}
            </button>
          {/each}

          {#if unresolvedMarkers}
            <span
              class="ml-2 flex items-center gap-1 text-xs text-[var(--color-danger)]"
            >
              <Icon icon="ph:warning" />
              Conflict markers still present
            </span>
          {/if}
        </div>

        <textarea
          class="scroll-thin h-80 w-full resize-none rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] p-3 font-mono text-xs leading-relaxed focus:border-[var(--color-accent)] focus:outline-none"
          bind:value={choices[index]}
          spellcheck="false"
        ></textarea>
      {/if}
    {/if}
  </div>

  {#snippet footer()}
    <button
      class="rounded-md px-3 py-1.5 text-xs text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]"
      onclick={onclose}
    >
      Cancel
    </button>
    <button
      class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90 disabled:opacity-50"
      disabled={anyUnresolved}
      onclick={submit}
    >
      Save and upload
    </button>
  {/snippet}
</Modal>
