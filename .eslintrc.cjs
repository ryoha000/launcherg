module.exports = {
  root: true,
  parserOptions: {
    sourceType: "module",
    ecmaVersion: 2019,
  },
  extends: [
    // add more generic rule sets here, such as:
    // 'eslint:recommended',
    "plugin:svelte/recommended",
    "@unocss",
  ],
  rules: {
    // override/add rules settings here, such as:
    // 'svelte/rule-name': 'error'
  },
};
