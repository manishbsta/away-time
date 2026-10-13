export default {
  '*.{js,jsx,ts,tsx}': ['oxlint --deny-warnings', 'prettier --write'],
  '*.{json,jsonc,css,md,yml,yaml}': 'prettier --write',
};
