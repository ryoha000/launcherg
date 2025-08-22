import antfu from '@antfu/eslint-config'

export default antfu({
  ignores: ['script', '**/script/**', '**/scripts/**', 'src-tauri/gen/**', 'src-tauri/target/**', '**/*.md', 'browser-extension/shared/src/typeshare/**', 'browser-extension/shared/src/proto/**'],
  svelte: true,
  unocss: true,
  rules: {
    'prefer-const': 'off',
  },
})
