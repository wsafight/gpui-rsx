import { test } from 'node:test';
import { strict as assert } from 'node:assert';
import { compatibleServerVersion, serverVersionRequirement } from '../src/server-version';

test('patch releases are compatible in either direction', () => {
  assert.equal(compatibleServerVersion('0.10.1', '0.10.0'), true);
  assert.equal(compatibleServerVersion('0.10.0', '0.10.8'), true);
  assert.equal(compatibleServerVersion('0.10.1+build.2', '0.10.0'), true);
});

test('other release lines and invalid server versions need replacement', () => {
  for (const server of ['0.9.9', '0.11.0', '1.10.0', 'garbage', '0.10', '0.10.0;echo bad']) {
    assert.equal(compatibleServerVersion(server, '0.10.0'), false, server);
  }
});

test('install selects the newest patch in the extension release line', () => {
  assert.equal(serverVersionRequirement('0.10.0'), '~0.10.0');
  assert.equal(serverVersionRequirement('0.10.9'), '~0.10.0');
  assert.equal(serverVersionRequirement('1.2.3'), '~1.2.0');
  assert.throws(() => serverVersionRequirement('0.10.0;echo bad'));
});
