const { execFileSync } = require('node:child_process');
const { appendFileSync } = require('node:fs');

const categories = ['core', 'tooling', 'demo', 'docs', 'editor', 'security'];

function classify(paths, full = false) {
  const result = Object.fromEntries(categories.map(category => [category, full]));
  const enable = (...names) => names.forEach(name => { result[name] = true; });
  for (const path of paths) {
    if (/^(docs\/|README(?:_CN)?\.md$)/.test(path)) enable('docs');
    else if (/^demo\//.test(path) || /^tests\/fixtures\/hot-reload\//.test(path)) enable('demo');
    else if (/^editors\//.test(path)) enable('editor');
    else if (/^crates\/gpui-rsx-(tools|wasm)\//.test(path)) enable('tooling', 'docs', 'editor');
    else if (/^crates\/cargo-rsx\//.test(path)) enable('tooling', 'editor');
    else if (/^(src\/|tests\/|crates\/gpui-rsx-core\/|crates\/gpui-stub)/.test(path)) enable('core', 'tooling', 'demo', 'docs', 'editor');
    else if (/^scripts\/test-hot-reload/.test(path)) enable('demo');
    else if (/^scripts\/(build-playground|test-playground)/.test(path)) enable('tooling', 'docs');
    else if (/^scripts\/dev-reload/.test(path)) enable('tooling');
    else enable(...categories);
    if (/(^|\/)(Cargo\.(toml|lock)|package\.json|bun\.lockb?)$/.test(path)) enable('security');
  }
  return result;
}

if (require.main === module) {
  const full = process.env.GITHUB_EVENT_NAME !== 'pull_request';
  const paths = full ? [] : execFileSync('git', ['diff', '--name-only', '-z', process.env.BASE_REV, 'HEAD'], { encoding: 'utf8' }).split('\0').filter(Boolean);
  const result = classify(paths, full);
  appendFileSync(process.env.GITHUB_OUTPUT, Object.entries(result).map(([key, value]) => `${key}=${value}\n`).join(''));
  console.log(JSON.stringify({ paths, checks: result }, null, 2));
}

module.exports = { categories, classify };
