import js from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";
import { defineConfig } from "eslint/config";

export default defineConfig([
  {
    files: ["/**/*.ts"], plugins: { js }, extends: ["js/recommended"]
  },
  {
    files: ["/**/*.ts"], languageOptions: { globals: globals.browser }
  },
  tseslint.configs.recommended,
  {
    rules: {
      semi: "error",
      indent: ["error", 2],
      "prefer-const": "error",
      "no-console": "warn", "no-unused-vars": "warn",
      // ✅ Stylistic
      'quotes': ['error', 'single', { avoidEscape: true }],      // Prefer single quotes
      'comma-dangle': ['error', 'always-multiline'],             // Trailing commas in multiline
      'object-curly-spacing': ['error', 'always'],               // Spaces inside object braces
      'array-bracket-spacing': ['error', 'never'],               // No spaces in arrays
      'space-before-function-paren': ['error', 'never'],         // No space before function ()
      'max-len': ['warn', { code: 100 }],                        // Warn on long lines

      // ✅ Code Safety
      'eqeqeq': ['error', 'always'],                             // Always use === / !==
      'no-implicit-coercion': 'warn',                            // Discourage !!, +x, etc.
      'no-else-return': 'error',                                 // Don't use else after return
      'no-alert': 'warn',                                        // Alert is discouraged
      'no-debugger': 'warn',                                     // Debugger shouldn't ship
  'consistent-return': 'error',                              // Always return or don't

  // ✅ Variable Handling
  'no-var': 'error',                                         // Use let/const instead of var
  'prefer-destructuring': ['warn', { object: true }],        // Use object destructuring
  'no-shadow': 'warn',                                       // Prevent shadowed variables
  'no-undef-init': 'error',                                  // Don't initialize with undefined

  // ✅ Functions and Control Flow
  'arrow-body-style': ['error', 'as-needed'],                // Omit `{}` for simple arrow funcs
  'curly': ['error', 'all'],                                 // Require {} for all control blocks
  'default-case': 'warn',                                    // Require default in switch
  'no-return-await': 'error',                                // Avoid return await in async

  // ✅ Import Rules (especially with Airbnb)
  'import/no-unresolved': 'error',
  'import/prefer-default-export': 'off',
  'import/no-extraneous-dependencies': ['error', {
    devDependencies: ['**/*.test.js', '**/*.config.js'],
  }]
    }
  },
]);
