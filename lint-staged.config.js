export default {
  '*.{js,ts,vue,json,css,scss}': ['prettier -w --log-level error'],
  '*.{js,ts,vue}': ['eslint --cache --fix --no-warn-ignored'],
};
