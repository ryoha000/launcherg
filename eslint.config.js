import antfu from '@antfu/eslint-config'

export default antfu({
  ignores: ['script', '**/script/**', 'src-tauri/gen/**', 'src-tauri/target/**'],
  svelte: true,
  rules: {
    'prefer-const': 'off',
  },
})
