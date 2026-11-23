<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import Icon from "@iconify/svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import SlideStage from "#lib/components/presentation/SlideStage.svelte";
  import { safeSvg } from "#lib/ts/safe-svg";
  import {
    forwardedKey,
    listenToSpeaker,
    sendToSpeaker,
    setWindowFullscreen,
    type AudienceState,
    type Point,
    type Slide,
    type SpeakerMessage,
  } from "#lib/ts/presentation-sync";

  const CONTROLS_HIDE_DELAY_MS = 2500;
  const session = page.url.searchParams.get("session");

  let slides = $state<Slide[]>([]);
  let audience = $state<AudienceState>({ index: 0, black: false, strokes: [] });
  let laser = $state<Point | null>(null);
  let ended = $state(false);
  let fullscreen = $state(false);
  let controlsVisible = $state(false);
  let hideControlsTimeout: number | undefined;

  function handleMessage(message: SpeakerMessage) {
    if (message.type === "slides") slides = message.slides.map((slide) => ({ ...slide, markup: safeSvg(slide.markup) }));
    else if (message.type === "state") audience = message.state;
    else if (message.type === "laser") laser = message.point;
    else if (message.type === "end") {
      ended = true;
      getCurrentWindow().close().catch(() => {});
    }
  }

  function changeFullscreen(enabled: boolean) {
    fullscreen = enabled;
    setWindowFullscreen(enabled);
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (!session || ended) return;
    event.preventDefault();
    if (event.key === "Escape" && fullscreen) {
      changeFullscreen(false);
      return;
    }
    sendToSpeaker(session, { type: "key", key: forwardedKey(event) });
  }

  function showControls() {
    controlsVisible = true;
    window.clearTimeout(hideControlsTimeout);
    hideControlsTimeout = window.setTimeout(() => (controlsVisible = false), CONTROLS_HIDE_DELAY_MS);
  }

  onMount(() => {
    if (!session) return;
    let stopListening: (() => void) | null = null;
    let unmounted = false;

    listenToSpeaker(session, handleMessage).then((unlisten) => {
      if (unmounted) {
        unlisten();
        return;
      }
      stopListening = unlisten;
      sendToSpeaker(session, { type: "hello" });
    });

    return () => {
      unmounted = true;
      window.clearTimeout(hideControlsTimeout);
      stopListening?.();
    };
  });
</script>

<svelte:head>
  <title>Presentation</title>
</svelte:head>

<svelte:window onkeydown={handleKeyDown} />

<div
  class="fixed inset-0 z-[100] select-none overflow-hidden bg-black text-white {controlsVisible ? '' : 'cursor-none'}"
  role="presentation"
  onpointermove={showControls}
  onpointerdown={showControls}
>
  {#if !session || ended}
    <div class="flex h-full items-center justify-center">
      <p class="text-lg text-white/50">
        {ended ? "The presentation has ended." : "Open this window from Present, then Speaker view."}
      </p>
    </div>
  {:else}
    <SlideStage
      slide={slides[audience.index]}
      strokes={audience.strokes}
      {laser}
      black={audience.black}
      emptyMessage="Waiting for the speaker..."
    />
    {#if !fullscreen}
      <button
        class="absolute right-5 top-4 flex items-center gap-2 rounded-lg border border-white/10 bg-neutral-900/85 px-3 py-2 text-sm font-medium text-white/80 backdrop-blur-md transition-opacity duration-300 hover:bg-white/10 hover:text-white
          {controlsVisible ? 'opacity-100' : 'pointer-events-none opacity-0'}"
        onclick={() => changeFullscreen(true)}
      >
        <Icon icon="mdi:fullscreen" class="text-lg" />
        Full screen
      </button>
    {/if}
  {/if}
</div>
