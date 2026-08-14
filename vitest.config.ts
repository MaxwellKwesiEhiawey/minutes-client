import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    // jsdom so theme/readingPrefs/highlight tests can touch
    // localStorage/document/matchMedia without a real browser.
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    environmentOptions: {
      jsdom: {
        // An explicit non-opaque origin gives jsdom a real Storage object on
        // newer Node releases instead of inheriting Node's incomplete global.
        url: "http://localhost/",
      },
    },
    include: ["src/**/*.test.{ts,tsx}"],
    coverage: {
      provider: "v8",
      include: ["src/**/*.{ts,tsx}"],
      exclude: [
        "src/**/*.test.{ts,tsx}",
        "src/**/*.d.ts",
        "src/types.ts",
        "src/main.tsx",
        "src/vite-env.d.ts",
      ],
      // Initial floor per the engineering audit (was 3.31% with no gate at
      // all). Raise these as more of src/ gets test coverage — components
      // and App.tsx state logic are the biggest remaining gaps.
      thresholds: {
        statements: 25,
        branches: 60,
        functions: 30,
        lines: 25,
      },
    },
  },
});
