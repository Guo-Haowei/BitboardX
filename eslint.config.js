import js from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";
import { defineConfig } from "eslint/config";

export default defineConfig([
  {
    files: ["src/**/*.{js,mjs,cjs,ts,mts,cts}"], plugins: { js }, extends: ["js/recommended"]
  },
  {
    files: ["src/**/*.{js,mjs,cjs,ts,mts,cts}"], languageOptions: { globals: globals.browser }
  },
  tseslint.configs.recommended,
  {
    rules: {
      semi: "error",
      indent: ["error", 2],
      "prefer-const": "error",
      "no-console": "warn", "no-unused-vars": "warn"
    }
  },
]);
