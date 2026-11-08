const assert = require('node:assert/strict');
const test = require('node:test');
const { classify } = require('./ci-changes.cjs');

test('documentation changes avoid the native and editor matrices', () => {
  assert.deepEqual(classify(['docs/src/styles/custom.css', 'README_CN.md']), {
    core: false, tooling: false, demo: false, docs: true, editor: false, security: false,
  });
});

test('core changes validate all consumers and tool changes validate editor and WASM consumers', () => {
  const core = classify(['crates/gpui-rsx-core/src/parser.rs']);
  for (const category of ['core', 'tooling', 'demo', 'docs', 'editor']) assert.equal(core[category], true);
  const tools = classify(['crates/gpui-rsx-tools/src/lib.rs']);
  assert.equal(tools.docs && tools.editor && tools.tooling, true);
  assert.equal(tools.demo || tools.core, false);
});

test('combined edits, dependency audits, unknown paths and full runs keep coverage', () => {
  const mixed = classify(['demo/src/main.rs', 'editors/vscode/package.json']);
  assert.equal(mixed.demo && mixed.editor && mixed.security, true);
  assert.ok(Object.values(classify(['Cargo.toml'])).every(Boolean));
  assert.ok(Object.values(classify(['new-tool/config'])).every(Boolean));
  assert.ok(Object.values(classify([], true)).every(Boolean));
  assert.equal(classify(['tests/fixtures/hot-reload/src/live.rs']).demo, true);
  assert.equal(classify(['docs/bun.lock']).security, true);
});
