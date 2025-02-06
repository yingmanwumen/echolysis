import esbuild from "esbuild";

esbuild
  .build({
    entryPoints: ["./src/extension.ts"],
    bundle: true,
    outfile: "./dist/extension.js",
    external: ["vscode"],
    format: "cjs",
    platform: "node",
    target: "node16",
    sourcemap: true,
    minify: process.env.NODE_ENV === "production",
  })
  .catch(() => process.exit(1));

