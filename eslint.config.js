import { includeIgnoreFile } from "@eslint/compat";
import js from "@eslint/js";
import prettier from "eslint-config-prettier";
import svelte from "eslint-plugin-svelte";
import { fileURLToPath } from "node:url";
import ts from "typescript-eslint";
import svelteConfig from "./svelte.config.js";

const gitignorePath = fileURLToPath(new URL("./.gitignore", import.meta.url));

export default [
  includeIgnoreFile(gitignorePath),
  js.configs.recommended,
  ...ts.configs.recommended,
  ...svelte.configs.recommended,
  prettier,
  ...svelte.configs.prettier,
  // TypeScript configuration
  {
    files: ["**/*.ts", "**/*.tsx"],
    languageOptions: {
      parserOptions: {
        projectService: true,
      },
    },
    rules: {
      // Enforce no unused variables (keep underscore prefix convention)
      "@typescript-eslint/no-unused-vars": ["warn", { argsIgnorePattern: "^_", varsIgnorePattern: "^_" }],
      // Allow explicit any when needed (warn instead of error)
      "@typescript-eslint/no-explicit-any": "warn",
      // Enforce consistent type imports
      "@typescript-eslint/consistent-type-imports": [
        "warn",
        { prefer: "type-imports", disallowTypeAnnotations: false },
      ],
    },
  },
  {
    files: ["**/*.svelte", "**/*.svelte.ts", "**/*.svelte.js"],
    languageOptions: {
      parserOptions: {
        projectService: true,
        extraFileExtensions: [".svelte"],
        parser: ts.parser,
        svelteConfig,
      },
    },
    rules: {
      // Svelte specific - keep useful ones
      "svelte/valid-compile": "error",
      "@typescript-eslint/no-unused-vars": "warn",
      "@typescript-eslint/no-explicit-any": "warn",
    },
  },
  {
    languageOptions: {
      globals: {
        window: "readonly",
        document: "readonly",
        localStorage: "readonly",
        console: "readonly",
        setTimeout: "readonly",
        clearTimeout: "readonly",
        setInterval: "readonly",
        clearInterval: "readonly",
        fetch: "readonly",
        URL: "readonly",
        process: "readonly",
      },
    },
    rules: {
      // Disable problematic rules
      "no-undef": "off",
      "no-unused-expressions": "off",
      "@typescript-eslint/no-unused-expressions": "off",
      // Keep useful rules but as warnings
      "no-console": "off", // Allow console for debugging
      "no-debugger": "warn",
      "prefer-const": "warn",
      "no-var": "error",
      // Security rules
      "no-eval": "error",
      "no-implied-eval": "error",
    },
  },

  // Override global rules for Svelte 5 rune patterns
  {
    files: ["**/*.svelte", "**/*.svelte.ts", "**/*.svelte.js"],
    rules: {
      // Svelte 5 runes often require `let { ... } = $props()` to stay reactive;
      // don't penalize this pattern.
      "prefer-const": "off",
    },
  },
];
