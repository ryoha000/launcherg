import antfu from '@antfu/eslint-config'

export default antfu({
  ignores: ['script', '**/script/**', '**/scripts/**', 'src-tauri/gen/**', 'src-tauri/target/**', '**/*.md', 'browser-extension/shared/src/typeshare/**', 'browser-extension/shared/src/proto/**', 'src/lib/typeshare/pubsub.ts'],
  svelte: true,
  unocss: true,
  rules: {
    'prefer-const': 'off',
  },
})
