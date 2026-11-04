import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { EventEmitter, once } from 'node:events';
import { PassThrough } from 'node:stream';
import { mkdtemp, mkdir, rm, symlink, writeFile } from 'node:fs/promises';
import * as os from 'node:os';
import * as path from 'node:path';
import { test } from 'node:test';
import { ReloadSession, snapshot, stop } from './dev-reload.mjs';

const launch = () => spawn(process.execPath, ['-e', 'setInterval(() => {}, 1000)'], { stdio: 'ignore' });

test('failed rebuild keeps the running application, then a later change recovers', async () => {
  let builds = 0;
  const metrics = [];
  const session = new ReloadSession({
    build: async () => { if (++builds === 2) throw new Error('compile failed'); return 'binary'; },
    launch,
    report: (entry) => metrics.push(entry),
  });
  try {
    await session.request();
    const first = session.child;
    await session.request();
    assert.equal(session.child, first);
    assert.equal(first.exitCode, null);
    await session.request();
    assert.notEqual(session.child, first);
    assert.notEqual(first.signalCode, null);
    assert.equal(metrics.filter((entry) => entry.ok === false).length, 1);
  } finally { await session.close(); }
  assert.notEqual(session.child.signalCode, null);
});

test('changes during a build coalesce and never launch an outdated artifact', async () => {
  let finish;
  let builds = 0;
  let launches = 0;
  const session = new ReloadSession({
    build: async () => { if (++builds === 1) await new Promise((resolve) => { finish = resolve; }); return 'binary'; },
    launch: () => { launches += 1; return launch(); },
  });
  try {
    const first = session.request();
    session.request();
    session.request();
    finish();
    await first;
    assert.equal(builds, 2);
    assert.equal(launches, 1);
  } finally { await session.close(); }
});

test('watch snapshots include added/deleted Rust and config files and ignore build output', async () => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'rsx-watch-test-'));
  try {
    const original = await snapshot([directory]);
    await mkdir(path.join(directory, 'target'));
    await writeFile(path.join(directory, 'target/generated.rs'), 'ignored');
    assert.equal(await snapshot([directory]), original);
    const source = path.join(directory, 'view.rs');
    await writeFile(source, 'fn main() {}');
    assert.notEqual(await snapshot([directory]), original);
    await rm(source);
    assert.equal(await snapshot([directory]), original);
  } finally { await rm(directory, { recursive: true, force: true }); }
});

test('snapshots deduplicate overlapping relative and absolute watch roots in either order', async () => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'rsx-watch-overlap-'));
  try {
    const nested = path.join(directory, 'src');
    await mkdir(nested);
    const source = path.join(nested, 'view.rs');
    await writeFile(source, 'fn view() {}');
    await writeFile(path.join(directory, 'Cargo.toml'), '[package]');
    await writeFile(path.join(directory, 'Cargo.lock'), '# lock');
    const expected = await snapshot([directory]);
    assert.equal(JSON.parse(expected).length, 3);
    for (const roots of [
      [directory, nested, source, path.relative(process.cwd(), directory)],
      [source, nested, directory, directory],
    ]) {
      assert.equal(await snapshot(roots), expected);
    }
    await mkdir(path.join(nested, 'assets'));
    await writeFile(path.join(nested, 'assets', 'image.svg'), '<svg/>');
    assert.equal(await snapshot([directory, nested]), expected);
    await rm(source);
    assert.equal(JSON.parse(await snapshot([directory, nested, source])).length, 2);
    await writeFile(source, 'fn replaced() {}');
    assert.equal(JSON.parse(await snapshot([directory, nested, source])).length, 3);
  } finally { await rm(directory, { recursive: true, force: true }); }
});

test('closing cancels an active build and stops the previous application', async () => {
  let builds = 0;
  const session = new ReloadSession({
    build: async (signal) => {
      if (++builds === 1) return 'binary';
      await new Promise((_, reject) => signal.addEventListener('abort', () => reject(signal.reason), { once: true }));
    },
    launch,
  });
  await session.request();
  const child = session.child;
  session.request();
  await session.close();
  assert.equal(session.abort.signal.aborted, true);
  assert.notEqual(child.signalCode, null);
  await session.request();
  assert.equal(builds, 2);
});

test('readiness metrics wait for a complete marker and report once', async () => {
  let ready;
  const reported = new Promise((resolve) => { ready = resolve; });
  const metrics = [];
  const session = new ReloadSession({
    build: async () => 'binary',
    launch: () => spawn(process.execPath, ['-e', `
      process.stdout.write('GPUI_RSX_');
      process.stdin.once('data', () => {
        process.stdout.write('READY\\nGPUI_RSX_READY\\n');
      });
      setInterval(() => {}, 1000);
    `], { stdio: ['pipe', 'pipe', 'ignore'] }),
    report: (entry) => { metrics.push(entry); if (entry.ready) ready(entry); },
  });
  try {
    await session.request();
    await once(session.child.stdout, 'data');
    assert.equal(metrics.filter((entry) => entry.ready).length, 0);
    session.child.stdin.write('continue');
    const result = await reported;
    assert.equal(metrics.filter((entry) => entry.ready).length, 1);
    assert.ok(result.editToReadyMs >= result.buildMs);
    assert.ok(result.restartMs >= 0);
  } finally { await session.close(); }
});

test('readiness is detected before trimming a large stdout chunk', async () => {
  const child = new EventEmitter();
  child.stdout = new PassThrough();
  const metrics = [];
  const session = new ReloadSession({
    build: async () => 'binary',
    launch: () => child,
    report: (entry) => metrics.push(entry),
  });
  try {
    await session.request();
    child.stdout.write('GPUI_RSX_READY\n' + 'log '.repeat(4096));
    assert.equal(metrics.filter((entry) => entry.ready).length, 1);
  } finally { await session.close(); }
});

test('changes while the old process stops skip an outdated launch', {
  skip: process.platform === 'win32', timeout: 5_000,
}, async () => {
  let builds = 0;
  let launches = 0;
  const session = new ReloadSession({
    build: async () => `binary-${++builds}`,
    launch: () => {
      launches += 1;
      if (launches > 1) return launch();
      return spawn(process.execPath, ['-e', `
        process.on('SIGTERM', () => process.send('stopping'));
        process.on('message', () => process.exit(0));
        process.send('ready');
        setInterval(() => {}, 1000);
      `], { stdio: ['ignore', 'ignore', 'ignore', 'ipc'] });
    },
  });
  try {
    await session.request();
    await once(session.child, 'message');
    const previous = session.child;
    const rebuilding = session.request();
    assert.deepEqual(await once(previous, 'message'), ['stopping', undefined]);
    session.request();
    previous.send('exit');
    await rebuilding;
    assert.equal(builds, 3);
    assert.equal(launches, 2);
  } finally { await session.close(); }
});

test('a failed queued build restores the latest successful application after stopping', {
  skip: process.platform === 'win32', timeout: 5_000,
}, async () => {
  let builds = 0;
  const executables = [];
  const metrics = [];
  const session = new ReloadSession({
    build: async () => {
      if (++builds === 3) throw new Error('latest edit does not compile');
      return `binary-${builds}`;
    },
    launch: (executable) => {
      executables.push(executable);
      if (executables.length > 1) return launch();
      return spawn(process.execPath, ['-e', `
        process.on('SIGTERM', () => process.send('stopping'));
        process.on('message', () => process.exit(0));
        process.send('ready');
        setInterval(() => {}, 1000);
      `], { stdio: ['ignore', 'ignore', 'ignore', 'ipc'] });
    },
    report: (entry) => metrics.push(entry),
  });
  try {
    await session.request();
    await once(session.child, 'message');
    const previous = session.child;
    const rebuilding = session.request();
    await once(previous, 'message');
    session.request();
    previous.send('exit');
    await rebuilding;
    assert.deepEqual(executables, ['binary-1', 'binary-2']);
    assert.equal(previous.exitCode, 0);
    assert.equal(session.child.exitCode, null);
    assert.equal(session.child.signalCode, null);
    assert.ok(metrics.some((entry) => !entry.ok));
    assert.ok(metrics.some((entry) => entry.ok && entry.fallback));
    const restored = session.child;
    await session.request();
    assert.deepEqual(executables, ['binary-1', 'binary-2', 'binary-4']);
    assert.notEqual(restored.signalCode, null);
  } finally { await session.close(); }
});

test('a synchronous launch failure can recover on the next request', async () => {
  let launches = 0;
  const session = new ReloadSession({
    build: async () => 'binary',
    launch: () => { if (++launches === 1) throw new Error('launch failed'); return launch(); },
  });
  try {
    await assert.rejects(session.request(), /launch failed/);
    await session.request();
    assert.ok(session.child.pid);
    assert.equal(launches, 2);
  } finally { await session.close(); }
});

test('stop handles an already exited child and is safe to repeat', async () => {
  const child = spawn(process.execPath, ['-e', 'process.exit(0)'], { stdio: 'ignore' });
  await once(child, 'exit');
  await stop(child);
  await stop(child);
  await stop(undefined);
  assert.equal(child.exitCode, 0);
});

test('stop forcefully terminates an application that ignores SIGTERM', {
  skip: process.platform === 'win32', timeout: 5_000,
}, async (context) => {
  const child = spawn(process.execPath, ['-e', `
    process.on('SIGTERM', () => {});
    process.send('ready');
    setInterval(() => {}, 1000);
  `], { stdio: ['ignore', 'ignore', 'ignore', 'ipc'] });
  context.after(() => {
    if (child.exitCode === null && child.signalCode === null) child.kill('SIGKILL');
  });
  await once(child, 'message');
  await stop(child);
  assert.equal(child.signalCode, 'SIGKILL');
});

test('spawn errors are reported and a later application can launch', async () => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'rsx-missing-executable-'));
  let launches = 0;
  const metrics = [];
  const session = new ReloadSession({
    build: async () => 'binary',
    launch: () => ++launches === 1
      ? spawn(path.join(directory, 'missing'), [], { stdio: 'ignore' })
      : launch(),
    report: (entry) => metrics.push(entry),
  });
  try {
    await session.request();
    await once(session.child, 'error');
    assert.ok(metrics.some((entry) => !entry.ok && entry.error.includes('ENOENT')));
    await session.request();
    assert.ok(session.child.pid);
    assert.equal(launches, 2);
  } finally {
    await session.close();
    await rm(directory, { recursive: true, force: true });
  }
});

test('snapshots ignore unrelated files and nested symbolic link cycles', async () => {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'rsx-watch-links-'));
  try {
    const original = await snapshot([directory]);
    await writeFile(path.join(directory, 'notes.md'), 'not watched');
    await mkdir(path.join(directory, 'nested'));
    await symlink(directory, path.join(directory, 'nested', 'cycle'), process.platform === 'win32' ? 'junction' : 'dir');
    assert.equal(await snapshot([directory]), original);
    await writeFile(path.join(directory, 'Cargo.toml'), '[package]');
    const configured = await snapshot([directory]);
    assert.notEqual(configured, original);
    await writeFile(path.join(directory, 'Cargo.lock'), 'version = 4');
    assert.notEqual(await snapshot([directory]), configured);
    assert.equal(await snapshot([path.join(directory, 'missing')]), '[]');
  } finally { await rm(directory, { recursive: true, force: true }); }
});
