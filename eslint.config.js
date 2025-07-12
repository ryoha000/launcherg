import antfu from '@antfu/eslint-config'

export default antfu({
  ignores: ['script', '**/script/**', 'src-tauri/gen/**'],
  svelte: true,
  rules: {
    'prefer-const': 'off',
  },
})
