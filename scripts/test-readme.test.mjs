import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

test('both READMEs use the compiled starter source', async () => {
  const root = new URL('../', import.meta.url);
  const starter = (await readFile(new URL('demo/src/bin/starter.rs', root), 'utf8')).trim();
  for (const name of ['README.md', 'README_CN.md']) {
    const readme = await readFile(new URL(name, root), 'utf8');
    assert.ok(readme.includes(`\`\`\`rust\n${starter}\n\`\`\``), `${name} differs from starter.rs`);
  }
});
