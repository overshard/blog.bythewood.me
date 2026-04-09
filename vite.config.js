import { resolve } from "path";
import { defineConfig } from "vite";

export default defineConfig({
  root: "static_src",
  base: "/static/",
  build: {
    outDir: resolve(__dirname, "static"),
    emptyOutDir: true,
    manifest: true,
    rollupOptions: {
      input: resolve(__dirname, "static_src/index.js"),
      output: {
        entryFileNames: "base-[hash].js",
        assetFileNames: (assetInfo) => {
          if (/\.(woff2?|eot|ttf|otf)$/.test(assetInfo.name)) {
            return "fonts/[name][extname]";
          }
          if (/\.css$/.test(assetInfo.name)) {
            return "base-[hash].css";
          }
          return "assets/[name]-[hash][extname]";
        },
      },
    },
  },
  css: {
    preprocessorOptions: {
      scss: {
        quietDeps: true,
      },
    },
  },
});
