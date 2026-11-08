const assert = require('node:assert/strict');
const { spawnSync } = require('node:child_process');
const { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } = require('node:fs');
const { tmpdir } = require('node:os');
const path = require('node:path');
const test = require('node:test');

const installer = path.resolve(__dirname, '../../scripts/install-ci-tool.sh');
const versions = [
  ['bun', '1.4.4', 'bun', '1.4.4'],
  ['binaryen', '130.0.0', 'wasm-opt', 'wasm-opt version 130 (version_130)'],
  ['wasm-pack', '0.15.1', 'wasm-pack', 'wasm-pack 0.15.1'],
  ['wasm-bindgen-cli', '0.2.130', 'wasm-bindgen', 'wasm-bindgen 0.2.130'],
  ['cargo-audit', '0.22.2', 'cargo-audit', 'cargo-audit 0.22.2'],
];

function fixture(context, entry = versions[0]) {
  const directory = mkdtempSync(path.join(tmpdir(), 'rsx-ci-tools-'));
  context.after(() => rmSync(directory, { recursive: true, force: true }));
  const shims = path.join(directory, 'shims');
  const installRoot = path.join(directory, 'install with spaces');
  const log = path.join(directory, 'commands.json');
  mkdirSync(shims);
  const binarySource = `#!${process.execPath}\nconsole.log(process.env.TEST_TOOL_OUTPUT);\n`;
  const packageManager = `#!${process.execPath}
const fs = require('node:fs');
const path = require('node:path');
const args = process.argv.slice(2);
fs.writeFileSync(process.env.TEST_TOOL_LOG, JSON.stringify({ command: path.basename(process.argv[1]), args }));
if (process.env.TEST_INSTALL_FAILURE) process.exit(19);
const prefixIndex = args.findIndex(argument => argument === '--prefix' || argument === '--root');
const bin = path.join(args[prefixIndex + 1], 'bin');
fs.mkdirSync(bin, { recursive: true });
fs.writeFileSync(path.join(bin, process.env.TEST_TOOL_BINARY), ${JSON.stringify(binarySource)}, { mode: 0o755 });
`;
  for (const manager of ['npm', 'cargo']) {
    writeFileSync(path.join(shims, manager), packageManager, { mode: 0o755 });
  }
  const [tool, version, binary, output] = entry;
  return {
    installRoot,
    cache() {
      mkdirSync(path.join(installRoot, 'bin'), { recursive: true });
      writeFileSync(path.join(installRoot, 'bin', binary), binarySource, { mode: 0o755 });
    },
    command() {
      return JSON.parse(readFileSync(log, 'utf8'));
    },
    run(overrides = {}, args = [tool, version, installRoot]) {
      return spawnSync('bash', [installer, ...args], {
        encoding: 'utf8',
        env: {
          ...process.env,
          PATH: `${shims}${path.delimiter}${process.env.PATH}`,
          TEST_TOOL_LOG: log,
          TEST_TOOL_BINARY: binary,
          TEST_TOOL_OUTPUT: output,
          ...overrides,
        },
      });
    },
  };
}

for (const entry of versions) {
  test(`installs and verifies pinned ${entry[0]} from its package registry`, context => {
    const setup = fixture(context, entry);
    const result = setup.run();
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stdout.trim(), entry[3]);
    const { command, args } = setup.command();
    if (entry[0] === 'bun' || entry[0] === 'binaryen') {
      assert.equal(command, 'npm');
      assert.ok(args.includes(`${entry[0]}@${entry[1]}`));
      for (const flag of ['--registry=https://registry.npmjs.org', '--fetch-retries=3', '--fetch-timeout=60000']) {
        assert.ok(args.includes(flag));
      }
    } else {
      assert.equal(command, 'cargo');
      assert.deepEqual(args, ['install', '--locked', '--version', entry[1], '--root', setup.installRoot, entry[0]]);
    }
  });
}

test('a verified warm cache needs no package-manager invocation', context => {
  const setup = fixture(context);
  setup.cache();
  const result = setup.run({ TEST_INSTALL_FAILURE: '1' });
  assert.equal(result.status, 0, result.stderr);
  assert.throws(() => setup.command(), { code: 'ENOENT' });
});

test('a cached tool with the wrong version fails closed', context => {
  const setup = fixture(context);
  setup.cache();
  const result = setup.run({ TEST_TOOL_OUTPUT: '1.4.3' });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /version mismatch/);
  assert.throws(() => setup.command(), { code: 'ENOENT' });
});

test('a fresh install with the wrong version fails closed', context => {
  const setup = fixture(context);
  const result = setup.run({ TEST_TOOL_OUTPUT: '1.4.3' });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /version mismatch/);
  assert.equal(setup.command().command, 'npm');
});

test('package-manager failures are propagated instead of skipping checks', context => {
  for (const entry of [versions[0], versions[2]]) {
    const setup = fixture(context, entry);
    assert.equal(setup.run({ TEST_INSTALL_FAILURE: '1' }).status, 19);
  }
});

test('unknown tools, floating versions and missing arguments are rejected before installation', context => {
  const setup = fixture(context);
  for (const args of [[], ['other', '1.4.4', setup.installRoot], ['bun', 'latest', setup.installRoot]]) {
    assert.equal(setup.run({}, args).status, 1);
  }
  assert.throws(() => setup.command(), { code: 'ENOENT' });
});

test('workflows share the cached installer without weakening optimization or audits', () => {
  const workflows = ['ci.yml', 'docs.yml'].map(name => readFileSync(path.join(__dirname, '../workflows', name), 'utf8'));
  for (const workflow of workflows) {
    assert.doesNotMatch(workflow, /oven-sh\/setup-bun|bun-version:/);
    assert.match(workflow, /uses: \.\/\.github\/actions\/setup-ci-tool\n\s+with:\n\s+tool: bun\n\s+version: 1\.4\.4/);
  }
  for (const [tool, version] of versions.slice(1)) {
    assert.ok(workflows[0].includes(`tool: ${tool}\n          version: ${version}`));
  }
  assert.match(workflows[0], /cargo audit --file Cargo\.lock/);
  assert.match(workflows[0], /bun audit --cwd docs/);
  assert.match(workflows[0], /bun audit --cwd editors\/vscode/);
  assert.match(workflows[0], /run: scripts\/build-playground\.sh/);
  const manifest = readFileSync(path.join(__dirname, '../../crates/gpui-rsx-wasm/Cargo.toml'), 'utf8');
  assert.doesNotMatch(manifest, /wasm-opt\s*=\s*false/);
  const action = readFileSync(path.join(__dirname, '../actions/setup-ci-tool/action.yml'), 'utf8');
  assert.match(action, /runner\.os.*runner\.arch.*inputs\.tool.*inputs\.version/);
  assert.match(action, /install-ci-tool\.sh.*\n\s+echo.*GITHUB_PATH/);
});
