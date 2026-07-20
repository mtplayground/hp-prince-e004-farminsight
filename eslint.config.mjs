import js from '@eslint/js';
import globals from 'globals';
import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  {
    ignores: [
      'frontend/dist',
      'node_modules',
      'target',
      'package-lock.json',
      'frontend/*.tsbuildinfo',
      'frontend/vite.config.js',
      'frontend/vite.config.d.ts',
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      ecmaVersion: 'latest',
      globals: {
        ...globals.browser,
        ...globals.es2022,
      },
    },
    plugins: {
      'react-hooks': reactHooks,
      'react-refresh': reactRefresh,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      'react-refresh/only-export-components': ['warn', { allowConstantExport: true }],
    },
  },
  {
    files: [
      '*.config.{js,ts}',
      'vite.config.ts',
      'frontend/*.config.{js,ts}',
      'frontend/vite.config.ts',
    ],
    languageOptions: {
      globals: {
        ...globals.node,
      },
    },
  },
);
