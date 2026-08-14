/* Presentation helpers for the transcript's speaker avatars. Purely visual:
   the colour is derived from the speaker's display name so the same person
   keeps the same swatch across the live and saved transcript views. */

/** Avatar palette, drawn from the design system's brand and secondary hues. */
const SPEAKER_COLORS = [
  "#0d69d4",
  "#ff5a00",
  "#0f7a4a",
  "#040dbf",
  "#5daff6",
  "#8c3200",
  "#2a6fba",
  "#b58800",
];

/** Stable colour for a speaker name; unknown speakers get the neutral navy. */
export function speakerColor(name: string | null): string {
  if (!name) return "#5b6b75";
  let hash = 0;
  for (let i = 0; i < name.length; i += 1) {
    hash = (hash * 31 + name.charCodeAt(i)) % 100000;
  }
  return SPEAKER_COLORS[hash % SPEAKER_COLORS.length];
}

/** Up to two initials for the avatar; "?" when the speaker is unknown. */
export function speakerInitials(name: string | null): string {
  if (!name) return "?";
  const words = name.trim().split(/\s+/).filter(Boolean);
  if (words.length === 0) return "?";
  if (words.length === 1) {
    // "Speaker 2" style labels read better as "S2" than "Sp".
    const single = words[0];
    return single.slice(0, 2).toUpperCase();
  }
  const first = words[0][0] ?? "";
  const last = words[words.length - 1][0] ?? "";
  return `${first}${last}`.toUpperCase();
}
