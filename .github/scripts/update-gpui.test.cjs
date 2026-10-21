const { test } = require('node:test');
const assert = require('node:assert/strict');
const { latestRelease, manifestVersion, updateManifest } = require('./update-gpui.cjs');

test('chooses the newest stable published Kit, excluding yanked and prerelease entries', () => {
  const index = ['0.8.0', '0.9.0', '0.10.0', '1.0.0-beta.1'].map(vers => ({ name: 'gpui-kit', vers }));
  index.push({ name: 'gpui-kit', vers: '1.0.0', yanked: true });
  const text = index.map(entry => JSON.stringify(entry)).join('\n');
  assert.equal(latestRelease(text, '0.8.0'), '0.10.0');
  assert.throws(() => latestRelease(text, '0.11.0'), /no current or newer/);
});

test('updates the exact Kit pin while preserving the gpui alias and other dependencies', () => {
  const source = '# Keep gpui:: available\ngpui = { package = "gpui-kit", version = "=0.8.0" }\ngpui-rsx = { path = ".." }\n';
  const updated = updateManifest(source, '0.9.0');
  assert.equal(manifestVersion(updated), '0.9.0');
  assert.equal(updated, source.replace('=0.8.0', '=0.9.0'));
  assert.throws(() => updateManifest(source, '0.7.0'), /downgrade/);
  assert.throws(() => updateManifest(source, '0.9.0" }\nother = "x'), /stable release/);
  assert.throws(() => manifestVersion(source + source), /one exact/);
  assert.throws(() => manifestVersion(source.replace('=0.8.0', '0.8.0')), /one exact/);
});
