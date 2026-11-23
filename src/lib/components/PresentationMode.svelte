<script lang="ts">
  import { onMount } from "svelte";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import PresentationHeader from "./presentation/PresentationHeader.svelte";
  import PresentationToolbar from "./presentation/PresentationToolbar.svelte";
  import SlideStage from "./presentation/SlideStage.svelte";
  import SlideOverview from "./presentation/SlideOverview.svelte";
  import SpeakerView from "./presentation/SpeakerView.svelte";
  import { collectSlides, type Slide } from "./presentation/slides";
  import {
    commitStrokes,
    emptyInk,
    redoInk,
    undoInk,
    type DrawingTool,
    type Point,
    type SlideInk,
    type Stroke,
  } from "./presentation/drawing";
  import {
    audienceSlides,
    closeAudienceWindow,
    listenToAudience,
    openAudienceWindow,
    sendToAudience,
    setWindowFullscreen,
    type AudienceMessage,
    type ForwardedKey,
    type PresentationLayout,
  } from "#lib/ts/presentation-sync";
  import { setHotkeysPaused } from "#lib/ts/hotkeys";
  import type { SpeakerNote } from "#lib/ts/api";

  interface Props {
    mode?: PresentationLayout;
    pages: string[];
    notes?: (SpeakerNote | null)[];
    onclose: () => void;
  }

  let { mode = "audience", pages, notes = [], onclose }: Props = $props();

  const TOOL_SHORTCUTS: Record<string, DrawingTool> = { l: "laser", p: "pen", h: "highlighter", e: "eraser" };
  const NEXT_KEYS = ["arrowright", "arrowdown", "pagedown", " "];
  const PREVIOUS_KEYS = ["arrowleft", "arrowup", "pageup", "backspace"];
  const CONTROLS_HIDE_DELAY_MS = 2500;

  const session = crypto.randomUUID();
  const isSpeaker = $derived(mode === "speaker");

  let slides = $state<Slide[]>([]);
  let currentSlide = $state(0);
  let tool = $state<DrawingTool>("laser");
  let color = $state("#ef4444");
  let laser = $state<Point | null>(null);
  let inkBySlide = $state<Record<number, SlideInk>>({});
  let overviewOpen = $state(false);
  let controlsVisible = $state(true);
  let elapsedSeconds = $state(0);
  let timerPaused = $state(true);
  let fullscreen = $state(false);
  let audienceConnected = $state(false);
  let black = $state(false);
  let frozen = $state(false);
  let heldIndex = $state(0);
  let hideControlsTimeout: number | undefined;
  let stopWatchingAudience: UnlistenFn | null = null;

  const currentInk = $derived(inkBySlide[currentSlide] ?? emptyInk());
  const progressPercent = $derived(slides.length > 1 ? (currentSlide / (slides.length - 1)) * 100 : 100);
  const shownIndex = $derived(black || frozen ? heldIndex : currentSlide);
  const visibleLaser = $derived(tool === "laser" ? laser : null);

  function goTo(index: number) {
    currentSlide = Math.max(0, Math.min(slides.length - 1, index));
  }

  function updateInk(ink: SlideInk) {
    inkBySlide[currentSlide] = ink;
  }

  function saveStrokes(strokes: Stroke[]) {
    updateInk(commitStrokes(currentInk, strokes));
  }

  function clearInk() {
    if (currentInk.strokes.length > 0) saveStrokes([]);
  }

  function toggleOverview() {
    overviewOpen = !overviewOpen;
  }

  function selectFromOverview(index: number) {
    goTo(index);
    overviewOpen = false;
  }

  function holdAudience(apply: () => void) {
    if (!black && !frozen) heldIndex = currentSlide;
    apply();
  }

  function toggleBlack() {
    holdAudience(() => (black = !black));
  }

  function toggleFreeze() {
    holdAudience(() => (frozen = !frozen));
  }

  function changeFullscreen(enabled: boolean) {
    fullscreen = enabled;
    setWindowFullscreen(enabled);
  }

  function exit() {
    onclose();
  }

  function showControls() {
    controlsVisible = true;
    window.clearTimeout(hideControlsTimeout);
    if (isSpeaker) return;
    hideControlsTimeout = window.setTimeout(() => {
      if (!overviewOpen) controlsVisible = false;
    }, CONTROLS_HIDE_DELAY_MS);
  }

  function handleKey(input: ForwardedKey): boolean {
    const key = input.key.toLowerCase();

    if ((input.ctrlKey || input.metaKey) && key === "z") {
      updateInk(input.shiftKey ? redoInk(currentInk) : undoInk(currentInk));
      return true;
    }
    if (input.ctrlKey || input.metaKey || input.altKey) return false;

    if (key === "escape") {
      if (fullscreen) changeFullscreen(false);
      else toggleOverview();
    } else if (key === "g") {
      toggleOverview();
    } else if (key === "c") {
      clearInk();
    } else if (isSpeaker && key === "b") {
      toggleBlack();
    } else if (isSpeaker && key === "f") {
      toggleFreeze();
    } else if (TOOL_SHORTCUTS[key]) {
      tool = TOOL_SHORTCUTS[key];
    } else if (NEXT_KEYS.includes(key)) {
      goTo(currentSlide + 1);
    } else if (PREVIOUS_KEYS.includes(key)) {
      goTo(currentSlide - 1);
    } else if (key === "home") {
      goTo(0);
    } else if (key === "end") {
      goTo(slides.length - 1);
    } else {
      return false;
    }
    return true;
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (handleKey(event)) event.preventDefault();
  }

  function sendState() {
    const strokes = $state.snapshot(inkBySlide[shownIndex]?.strokes ?? []);
    sendToAudience(session, { type: "state", state: { index: shownIndex, black, strokes } });
  }

  function sendLaser() {
    const point = shownIndex === currentSlide && !black ? visibleLaser : null;
    sendToAudience(session, { type: "laser", point: point && { ...point } });
  }

  function stopAudienceWatch() {
    stopWatchingAudience?.();
    stopWatchingAudience = null;
  }

  async function showAudienceWindow() {
    stopAudienceWatch();
    audienceConnected = false;
    try {
      stopWatchingAudience = await openAudienceWindow(session, exit);
    } catch {
      audienceConnected = false;
    }
  }

  function handleAudienceMessage(message: AudienceMessage) {
    if (message.type === "hello") {
      audienceConnected = true;
      sendToAudience(session, { type: "slides", slides: audienceSlides($state.snapshot(slides)) });
      sendState();
      sendLaser();
    } else if (message.type === "key") {
      handleKey(message.key);
    }
  }

  $effect(() => {
    if (isSpeaker) sendState();
  });

  $effect(() => {
    if (isSpeaker) sendLaser();
  });

  onMount(() => {
    slides = collectSlides(pages, notes);
    setHotkeysPaused(true);
    (document.activeElement as HTMLElement | null)?.blur();

    const timer = window.setInterval(() => {
      if (!timerPaused) elapsedSeconds += 1;
    }, 1000);
    showControls();

    let stopListening: UnlistenFn | null = null;
    let unmounted = false;

    if (isSpeaker) {
      listenToAudience(session, handleAudienceMessage).then((unlisten) => {
        if (unmounted) unlisten();
        else stopListening = unlisten;
      });
      showAudienceWindow();
    } else {
      changeFullscreen(true);
    }

    return () => {
      unmounted = true;
      window.clearInterval(timer);
      window.clearTimeout(hideControlsTimeout);
      setHotkeysPaused(false);
      if (fullscreen) setWindowFullscreen(false);
      if (isSpeaker) {
        stopAudienceWatch();
        sendToAudience(session, { type: "end" });
        stopListening?.();
        closeAudienceWindow();
      }
    };
  });
</script>

<svelte:window onkeydown={handleKeyDown} />

{#snippet header()}
  <PresentationHeader
    visible={controlsVisible || overviewOpen}
    floating={!isSpeaker}
    {elapsedSeconds}
    {timerPaused}
    {overviewOpen}
    {black}
    {frozen}
    onfullscreen={isSpeaker || fullscreen ? undefined : () => changeFullscreen(true)}
    onblack={isSpeaker ? toggleBlack : undefined}
    onfreeze={isSpeaker ? toggleFreeze : undefined}
    ontoggletimer={() => (timerPaused = !timerPaused)}
    onresettimer={() => (elapsedSeconds = 0)}
    onoverview={toggleOverview}
    onexit={exit}
  />
{/snippet}

{#snippet toolbar()}
  {#if !overviewOpen}
    <PresentationToolbar
      visible={controlsVisible}
      floating={!isSpeaker}
      bind:tool
      bind:color
      {currentSlide}
      slideCount={slides.length}
      canUndo={currentInk.past.length > 0}
      canRedo={currentInk.future.length > 0}
      canClear={currentInk.strokes.length > 0}
      onprevious={() => goTo(currentSlide - 1)}
      onnext={() => goTo(currentSlide + 1)}
      onoverview={toggleOverview}
      onundo={() => updateInk(undoInk(currentInk))}
      onredo={() => updateInk(redoInk(currentInk))}
      onclear={clearInk}
    />
  {/if}
{/snippet}

<div
  class="fixed inset-0 z-[100] select-none overflow-hidden bg-neutral-950 text-white {isSpeaker ? 'flex flex-col' : ''}"
  role="dialog"
  aria-modal="true"
  aria-label={isSpeaker ? "Speaker view" : "Presentation"}
  tabindex="-1"
  onpointermove={showControls}
  onpointerdown={showControls}
>
  {#if isSpeaker}
    {@render header()}
    <SpeakerView
      current={slides[currentSlide]}
      next={slides[currentSlide + 1]}
      strokes={currentInk.strokes}
      laser={visibleLaser}
      {tool}
      {color}
      {audienceConnected}
      audienceBlack={black}
      audienceFrozen={frozen}
      audienceIndex={heldIndex}
      oncommit={saveStrokes}
      onlaser={(point) => (laser = point)}
      onopenaudience={showAudienceWindow}
    />
    {@render toolbar()}
  {:else}
    <div class="absolute inset-0">
      <SlideStage
        slide={slides[currentSlide]}
        strokes={currentInk.strokes}
        laser={visibleLaser}
        {tool}
        {color}
        interactive
        oncommit={saveStrokes}
        onlaser={(point) => (laser = point)}
      />
    </div>
    {@render header()}
    {@render toolbar()}
  {/if}

  {#if overviewOpen}
    <SlideOverview {slides} {currentSlide} onselect={selectFromOverview} />
  {/if}

  <div class="absolute inset-x-0 bottom-0 z-20 h-1 bg-white/10">
    <div class="h-full bg-blue-500 transition-[width] duration-300" style="width: {progressPercent}%"></div>
  </div>
</div>
