<script lang="ts">
  import SettingsHeader from "./SettingsHeader.svelte";
  import SettingsCard from "./SettingsCard.svelte";
  import SettingsRow from "./SettingsRow.svelte";
  import ToggleSwitch from "./ToggleSwitch.svelte";
  import { editorFonts, findEditorFont, loadEditorFont } from "$lib/ts/editor/fonts";
  import {
    defaultEditorSettings,
    editorSettingsStore,
    MAX_FONT_SIZE,
    MIN_FONT_SIZE,
    spellCheckLanguages,
    updateEditorSettings,
  } from "$lib/ts/editor/settings";
  import { minimapSizeOptions, type MinimapSize } from "$lib/ts/editor/minimap/minimap-sizes";

  let { fieldClass }: { fieldClass: string } = $props();

  const sampleText = "= Introduction\n#let total = 42\nThe quick brown fox jumps over the lazy dog.";

  let previewFont = $derived(findEditorFont($editorSettingsStore.fontFamily));

  $effect(() => {
    loadEditorFont($editorSettingsStore.fontFamily);
  });
</script>

<SettingsHeader title="Editor" description="Customize the code editor. Saved on this device." icon="ph:code" />

<div class="flex flex-col gap-4">
  <SettingsCard title="Text">
    <SettingsRow label="Font size" description="{MIN_FONT_SIZE} to {MAX_FONT_SIZE} pixels.">
      <input
        type="range"
        min={MIN_FONT_SIZE}
        max={MAX_FONT_SIZE}
        value={$editorSettingsStore.fontSize}
        oninput={(event) => updateEditorSettings({ fontSize: Number(event.currentTarget.value) })}
        class="w-32 accent-[var(--color-accent)]"
        aria-label="Font size"
      />
      <span class="w-10 text-right text-xs tabular-nums">{$editorSettingsStore.fontSize}px</span>
    </SettingsRow>
    <SettingsRow label="Font family" description="Web fonts download the first time you use them.">
      <select
        class={fieldClass}
        value={$editorSettingsStore.fontFamily}
        onchange={(event) => updateEditorSettings({ fontFamily: event.currentTarget.value })}
        aria-label="Font family"
      >
        {#each editorFonts as font (font.name)}
          <option value={font.name}>{font.name}</option>
        {/each}
      </select>
    </SettingsRow>
    <div class="px-4 py-3">
      <pre
        class="overflow-x-auto rounded-md border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-3"
        style="font-family: {previewFont.stack}; font-size: {$editorSettingsStore.fontSize}px;">{sampleText}</pre>
    </div>
  </SettingsCard>

  <SettingsCard title="Spell checker">
    <SettingsRow label="Check spelling" description="Checks your writing and skips code, math, labels and links.">
      <ToggleSwitch label="Check spelling" checked={$editorSettingsStore.spellCheck} onchange={(checked) => updateEditorSettings({ spellCheck: checked })} />
    </SettingsRow>
    <SettingsRow label="Language" description="Uses the dictionaries available on this system.">
      <select
        class={fieldClass}
        value={$editorSettingsStore.spellCheckLanguage}
        disabled={!$editorSettingsStore.spellCheck}
        onchange={(event) => updateEditorSettings({ spellCheckLanguage: event.currentTarget.value })}
        aria-label="Spell check language"
      >
        {#each spellCheckLanguages as language (language.code)}
          <option value={language.code}>{language.label}</option>
        {/each}
      </select>
    </SettingsRow>
  </SettingsCard>

  <SettingsCard title="Layout">
    <SettingsRow label="Line numbers" description="Show line numbers next to the code.">
      <ToggleSwitch label="Line numbers" checked={$editorSettingsStore.lineNumbers} onchange={(checked) => updateEditorSettings({ lineNumbers: checked })} />
    </SettingsRow>
    <SettingsRow label="Minimap" description="Show an overview of the whole file on the right side.">
      <ToggleSwitch label="Minimap" checked={$editorSettingsStore.minimap} onchange={(checked) => updateEditorSettings({ minimap: checked })} />
    </SettingsRow>
    <SettingsRow label="Minimap size" description="Width of the minimap and how large its lines are drawn.">
      <select
        class={fieldClass}
        value={$editorSettingsStore.minimapSize}
        disabled={!$editorSettingsStore.minimap}
        onchange={(event) => updateEditorSettings({ minimapSize: event.currentTarget.value as MinimapSize })}
        aria-label="Minimap size"
      >
        {#each minimapSizeOptions as option (option.value)}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </SettingsRow>
  </SettingsCard>

  <div class="flex justify-end">
    <button
      class="rounded-md border border-[var(--color-line)] px-3 py-1.5 text-xs hover:bg-[var(--color-surface-muted)]"
      onclick={() => editorSettingsStore.set(defaultEditorSettings)}
    >
      Reset to defaults
    </button>
  </div>
</div>
