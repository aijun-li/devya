/** @type {import("prettier").Config} */
export default {
  printWidth: 120,
  useTabs: false,
  tabWidth: 2,
  semi: true,
  singleQuote: true,
  quoteProps: 'consistent',
  trailingComma: 'all',
  arrowParens: 'always',
  bracketSpacing: true,
  plugins: ['prettier-plugin-tailwindcss'],
};
