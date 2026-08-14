// vitest.config.ts
import { defineConfig } from "file:///sessions/kind-affectionate-babbage/mnt/desksec-client/node_modules/vitest/dist/config.js";
var vitest_config_default = defineConfig({
  test: {
    // Pure-function unit tests run in Node; no DOM needed.
    environment: "node",
    include: ["src/**/*.test.ts"],
    coverage: {
      provider: "v8",
      include: ["src/**/*.{ts,tsx}"],
      exclude: [
        "src/**/*.test.ts",
        "src/**/*.d.ts",
        "src/types.ts",
        "src/main.tsx",
        "src/vite-env.d.ts"
      ]
    }
  }
});
export {
  vitest_config_default as default
};
//# sourceMappingURL=data:application/json;base64,ewogICJ2ZXJzaW9uIjogMywKICAic291cmNlcyI6IFsidml0ZXN0LmNvbmZpZy50cyJdLAogICJzb3VyY2VzQ29udGVudCI6IFsiY29uc3QgX192aXRlX2luamVjdGVkX29yaWdpbmFsX2Rpcm5hbWUgPSBcIi9zZXNzaW9ucy9raW5kLWFmZmVjdGlvbmF0ZS1iYWJiYWdlL21udC9kZXNrc2VjLWNsaWVudFwiO2NvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9maWxlbmFtZSA9IFwiL3Nlc3Npb25zL2tpbmQtYWZmZWN0aW9uYXRlLWJhYmJhZ2UvbW50L2Rlc2tzZWMtY2xpZW50L3ZpdGVzdC5jb25maWcudHNcIjtjb25zdCBfX3ZpdGVfaW5qZWN0ZWRfb3JpZ2luYWxfaW1wb3J0X21ldGFfdXJsID0gXCJmaWxlOi8vL3Nlc3Npb25zL2tpbmQtYWZmZWN0aW9uYXRlLWJhYmJhZ2UvbW50L2Rlc2tzZWMtY2xpZW50L3ZpdGVzdC5jb25maWcudHNcIjtpbXBvcnQgeyBkZWZpbmVDb25maWcgfSBmcm9tIFwidml0ZXN0L2NvbmZpZ1wiO1xuXG5leHBvcnQgZGVmYXVsdCBkZWZpbmVDb25maWcoe1xuICB0ZXN0OiB7XG4gICAgLy8gUHVyZS1mdW5jdGlvbiB1bml0IHRlc3RzIHJ1biBpbiBOb2RlOyBubyBET00gbmVlZGVkLlxuICAgIGVudmlyb25tZW50OiBcIm5vZGVcIixcbiAgICBpbmNsdWRlOiBbXCJzcmMvKiovKi50ZXN0LnRzXCJdLFxuICAgIGNvdmVyYWdlOiB7XG4gICAgICBwcm92aWRlcjogXCJ2OFwiLFxuICAgICAgaW5jbHVkZTogW1wic3JjLyoqLyoue3RzLHRzeH1cIl0sXG4gICAgICBleGNsdWRlOiBbXG4gICAgICAgIFwic3JjLyoqLyoudGVzdC50c1wiLFxuICAgICAgICBcInNyYy8qKi8qLmQudHNcIixcbiAgICAgICAgXCJzcmMvdHlwZXMudHNcIixcbiAgICAgICAgXCJzcmMvbWFpbi50c3hcIixcbiAgICAgICAgXCJzcmMvdml0ZS1lbnYuZC50c1wiLFxuICAgICAgXSxcbiAgICB9LFxuICB9LFxufSk7XG4iXSwKICAibWFwcGluZ3MiOiAiO0FBQXdWLFNBQVMsb0JBQW9CO0FBRXJYLElBQU8sd0JBQVEsYUFBYTtBQUFBLEVBQzFCLE1BQU07QUFBQTtBQUFBLElBRUosYUFBYTtBQUFBLElBQ2IsU0FBUyxDQUFDLGtCQUFrQjtBQUFBLElBQzVCLFVBQVU7QUFBQSxNQUNSLFVBQVU7QUFBQSxNQUNWLFNBQVMsQ0FBQyxtQkFBbUI7QUFBQSxNQUM3QixTQUFTO0FBQUEsUUFDUDtBQUFBLFFBQ0E7QUFBQSxRQUNBO0FBQUEsUUFDQTtBQUFBLFFBQ0E7QUFBQSxNQUNGO0FBQUEsSUFDRjtBQUFBLEVBQ0Y7QUFDRixDQUFDOyIsCiAgIm5hbWVzIjogW10KfQo=
