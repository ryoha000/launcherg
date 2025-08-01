import antfu from '@antfu/eslint-config'

export default antfu({
  ignores: ['script', '**/script/**', '**/scripts/**', 'src-tauri/gen/**', 'src-tauri/target/**', '**/*.md', 'browser-extension/src/proto'],
  svelte: true,
  unocss: true,
  rules: {
    'prefer-const': 'off',
  },
}, {
  files: ['browser-extension/**'],
  rules: {
    'no-console': 'off',
    'no-new': 'off',
  },
})
