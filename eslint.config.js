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
  ...ts.configs.strict,
  ...svelte.configs.recommended,
  prettier,
  ...svelte.configs.prettier,
  // TypeScript type-aware linting configuration
  {
    files: ["**/*.ts", "**/*.tsx"],
    languageOptions: {
      parserOptions: {
        projectService: true,
      },
    },
    rules: {
      // Detect usage of deprecated TypeScript APIs
      "@typescript-eslint/no-deprecated": "error",
      // Disallow `any` type
      "@typescript-eslint/no-explicit-any": "error",
      // Enforce no unused variables
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
        },
      ],
      // Require explicit type annotations on class properties
      "@typescript-eslint/explicit-member-accessibility": [
        "warn",
        {
          accessibility: "explicit",
          overrides: {
            constructors: "no-public",
          },
        },
      ],
      // Forbid non-null assertion for non-optional properties
      "@typescript-eslint/no-non-null-assertion": "warn",
      // Require Promise-like values to be handled
      "@typescript-eslint/no-floating-promises": "error",
      // Require explicit types on exports and public class methods
      "@typescript-eslint/explicit-module-boundary-types": [
        "warn",
        { allowArgumentsExplicitlyTypedAsAny: true },
      ],
      // Enforce consistent type imports
      "@typescript-eslint/consistent-type-imports": [
        "error",
        {
          prefer: "type-imports",
          disallowTypeAnnotations: false,
        },
      ],
      // Enforce consistent type exports
      "@typescript-eslint/consistent-type-exports": [
        "error",
        { fixMixedExportsWithInlineTypeSpecifier: true },
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
      // Detect usage of deprecated TypeScript APIs in Svelte
      "@typescript-eslint/no-deprecated": "error",
      // Disallow `any` type in Svelte
      "@typescript-eslint/no-explicit-any": "error",
      // Svelte best practices
      "svelte/no-unused-svelte-ignore": "error",
      "svelte/no-reactive-reassign": "warn",
      "svelte/valid-compile": "error",
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
      // typescript-eslint strongly recommend that you do not use the no-undef lint rule on TypeScript projects.
      // see: https://typescript-eslint.io/troubleshooting/faqs/eslint/#i-get-errors-from-the-no-undef-rule-about-global-variables-not-being-defined-even-though-there-are-no-typescript-errors
      "no-undef": "off",
      "no-unused-expressions": "off",
      "@typescript-eslint/no-unused-expressions": "off",
      // Additional best practice rules
      "no-console": ["warn", { allow: ["warn", "error"] }],
      "no-debugger": "warn",
      "prefer-const": "error",
      "no-var": "error",
      eqeqeq: ["error", "always", { null: "ignore" }],
      // Security-related rules
      "no-eval": "error",
      "no-implied-eval": "error",
      "no-new-func": "error",
      "no-script-url": "error",
      "no-with": "error",
      // Best practices
      "no-empty-function": ["error", { allow: ["arrowFunctions", "constructors"] }],
      "no-multi-spaces": "error",
      "no-multiple-empty-lines": ["error", { max: 1 }],
      "no-trailing-spaces": "error",
      "object-curly-spacing": ["error", "always"],
      // Control flow
      "no-fallthrough": "error",
      "no-unreachable": "error",
      "require-await": "warn",
    },
  },
];
