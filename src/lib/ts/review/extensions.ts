import { get } from "svelte/store";
import { keymap } from "@codemirror/view";
import type { Extension } from "@codemirror/state";
import { reviewPanelOpenStore } from "./client";
import { reviewSessionStore } from "./session";
import { activeThreadStore, startCommentOnSelection } from "./comments";
import { activeSuggestionStore } from "./suggestions";
import { commentMarks } from "./comment-marks";
import { suggestionMarks } from "./suggestion-marks";

function openThread(commentId: string) {
  activeThreadStore.set(commentId);
  reviewPanelOpenStore.set(true);
}

function openSuggestion(suggestionId: string) {
  activeSuggestionStore.set(suggestionId);
  reviewPanelOpenStore.set(true);
}

export function reviewExtensions(): Extension[] {
  return [
    commentMarks(openThread),
    suggestionMarks(openSuggestion),
    keymap.of([
      {
        key: "Mod-Alt-m",
        preventDefault: true,
        run: (view) => {
          const session = get(reviewSessionStore);
          if (!session || session.role === "viewer") return false;
          startCommentOnSelection(view, session.text);
          return true;
        },
      },
    ]),
  ];
}
