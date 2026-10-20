import { writable } from "svelte/store";

export type InsertDialog = "image" | "table" | "document" | "symbol" | "emoji";

export const insertDialogStore = writable<InsertDialog | null>(null);

export const insertOptions: { dialog: InsertDialog; label: string; icon: string }[] = [
  { dialog: "image", label: "Image", icon: "ph:image" },
  { dialog: "table", label: "Table", icon: "ph:table" },
  { dialog: "document", label: "Document", icon: "ph:file-text" },
  { dialog: "symbol", label: "Symbol", icon: "mdi:omega" },
  { dialog: "emoji", label: "Emoji", icon: "ph:smiley" }
];
