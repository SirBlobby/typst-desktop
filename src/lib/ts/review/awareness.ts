import type { Awareness } from "y-protocols/awareness";

const userColors = [
  "#30bced", "#6eeb83", "#ffbc42", "#ecd444", "#ee6352",
  "#9ac2c9", "#8acb88", "#1be7ff", "#ff0054", "#9e0059",
];

function colorFor(seed: string) {
  let hash = 0;
  for (const character of seed) hash = (hash * 31 + character.charCodeAt(0)) >>> 0;
  return userColors[hash % userColors.length];
}

export function setLocalAwarenessUser(awareness: Awareness, account: { user_id: string; username: string }) {
  const color = colorFor(account.user_id);
  awareness.setLocalStateField("user", {
    id: account.user_id,
    name: account.username,
    color,
    colorLight: color + "33",
    avatar: `/api/users/${account.user_id}/avatar`,
  });
}
