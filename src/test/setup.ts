// Node 25 exposes an incomplete global `localStorage` unless it is launched
// with a persistence file. Install a small standards-shaped in-memory store
// for unit tests; production still uses the WebView's native Storage object.
const values = new Map<string, string>();
const localStorageMock: Storage = {
  get length() {
    return values.size;
  },
  clear: () => values.clear(),
  getItem: (key) => values.get(String(key)) ?? null,
  key: (index) => [...values.keys()][index] ?? null,
  removeItem: (key) => values.delete(String(key)),
  setItem: (key, value) => values.set(String(key), String(value)),
};

Object.defineProperty(globalThis, "localStorage", {
  configurable: true,
  value: localStorageMock,
});
Object.defineProperty(window, "localStorage", {
  configurable: true,
  value: localStorageMock,
});

// jsdom implements no layout, so it has no scrollIntoView. The transcript views
// call it to keep the newest line in view while a meeting records.
if (!Element.prototype.scrollIntoView) {
  Element.prototype.scrollIntoView = () => {};
}
