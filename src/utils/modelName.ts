/**
 * Short, friendly display name for a model identifier. Backends report full
 * provider paths (e.g. `accounts/fireworks/models/gpt-oss-120b`); the last
 * path segment is the recognizable model name. Derived generically — no
 * vendor is hard-coded — because the backend model is in flux. Returns the
 * input unchanged when there is no path structure.
 */
export function shortModelName(model: string): string {
  const segments = model.split("/").filter((s) => s.trim().length > 0);
  return segments.length > 0 ? segments[segments.length - 1] : model;
}
