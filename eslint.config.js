import antfu from '@antfu/eslint-config'

export default antfu({
  ignores: ['script', '**/script/**'],
  svelte: true,
  rules: {
    'prefer-const': 'off',
  },
})
