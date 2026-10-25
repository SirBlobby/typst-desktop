<script lang="ts">
  import type { EditorView } from "@codemirror/view";
  import type { FileEntry } from "#lib/ts/api";
  import { insertText } from "#lib/ts/editor-actions";
  import { insertBlock } from "#lib/ts/editor/insert";
  import { insertDialogStore } from "#lib/ts/editor/insert-dialog";
  import { symbolCategories, type PickerCharacter } from "#lib/ts/editor/symbols";
  import { emojiCategories } from "#lib/ts/editor/emoji";
  import InsertImageModal from "./InsertImageModal.svelte";
  import InsertTableModal from "./InsertTableModal.svelte";
  import InsertDocumentModal from "./InsertDocumentModal.svelte";
  import CharacterPickerModal from "./CharacterPickerModal.svelte";

  let { view, targetPath, files, activePath }: {
    view: EditorView | null;
    targetPath: string;
    files: FileEntry[];
    activePath: string | null;
  } = $props();

  function close() {
    insertDialogStore.set(null);
  }

  function insertAsBlock(block: string) {
    if (view) insertBlock(view, block);
    close();
  }

  function pickCharacter(entry: PickerCharacter) {
    insertText(view, entry.character);
  }
</script>

{#if $insertDialogStore === "image"}
  <InsertImageModal {targetPath} onInsert={insertAsBlock} onClose={close} />
{:else if $insertDialogStore === "table"}
  <InsertTableModal onInsert={insertAsBlock} onClose={close} />
{:else if $insertDialogStore === "document"}
  <InsertDocumentModal {targetPath} {files} {activePath} onInsert={insertAsBlock} onClose={close} />
{:else if $insertDialogStore === "symbol"}
  <CharacterPickerModal title="Insert symbol" icon="mdi:omega" categories={symbolCategories} onPick={pickCharacter} onClose={close} />
{:else if $insertDialogStore === "emoji"}
  <CharacterPickerModal title="Insert emoji" icon="ph:smiley" categories={emojiCategories} onPick={pickCharacter} onClose={close} />
{/if}
