import { expect, mock, test } from 'bun:test';
import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';

mock.module('vscode', () => ({}));

const require = createRequire(import.meta.url);
const clientEntry = require.resolve('vscode-languageclient/node');
const { matchGlobPattern } = require(join(dirname(clientEntry), '../common/utils/globPattern.js'));

test('language client matches watched Rust and configuration files', () => {
  expect(matchGlobPattern('**/*.rs', { fsPath: 'src/main.rs' })).toBe(true);
  expect(matchGlobPattern('**/*.rs', { fsPath: 'src/main.ts' })).toBe(false);
  expect(matchGlobPattern('**/{Cargo,rsx}.toml', { fsPath: 'app/rsx.toml' })).toBe(true);
  expect(matchGlobPattern('**/{Cargo,rsx}.toml', { fsPath: 'app/other.toml' })).toBe(false);
});

test('language client rejects empty and comment glob patterns', () => {
  expect(matchGlobPattern('', { fsPath: 'src/main.rs' })).toBe(false);
  expect(matchGlobPattern('#ignored', { fsPath: 'src/main.rs' })).toBe(false);
});
