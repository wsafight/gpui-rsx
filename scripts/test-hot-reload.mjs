import assert from 'node:assert/strict';
import { execFile, spawn } from 'node:child_process';
import { cp, mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { promisify } from 'node:util';

export function frames(log) {
  return [...log.matchAll(/GPUI_RSX_LIVE pid=(\d+) seconds=(\d+) clicks=(\d+) revision=([\w-]+)/g)]
    .map(([, pid, seconds, clicks, revision]) => ({ pid: Number(pid), seconds: Number(seconds), clicks: Number(clicks), revision }));
}

export function preserved(before, after) {
  assert.equal(after.pid, before.pid, 'The application restarted instead of applying a patch');
  assert.ok(after.seconds >= before.seconds, 'Timer state was reset');
  assert.ok(after.clicks >= before.clicks, 'Click state was reset');
  assert.notEqual(after.revision, before.revision, 'The rendered revision did not change');
}

export async function probe({
  dx = process.env.RSX_DX || 'dx',
  target,
  temporaryRoot = tmpdir(),
  initialTimeout = 600_000,
  patchTimeout = 180_000,
  pollInterval = 100,
  output = message => process.stdout.write(message),
} = {}) {
  const { stdout: version } = await promisify(execFile)(dx, ['--version']);
  assert.match(version, /\b0\.7\.10\b/, 'This reproduction requires Dioxus CLI 0.7.10; set RSX_DX to its binary');
  const root = fileURLToPath(new URL('..', import.meta.url));
  const fixture = path.join(root, 'tests/fixtures/hot-reload');
  const directory = await mkdtemp(path.join(temporaryRoot, 'gpui-rsx-kit-patch-'));
  target ??= path.join(root, 'target/hot-reload-kit-probe');
  await mkdir(target, { recursive: true });
  await rm(path.join(target, 'verification.json'), { force: true });
  let child;
  let log = '';
  let succeeded = false;
  let aborted = false;
  const abort = () => { aborted = true; };
  process.on('SIGINT', abort);
  process.on('SIGTERM', abort);
  try {
    await cp(path.join(fixture, 'src'), path.join(directory, 'src'), { recursive: true });
    const manifest = (await readFile(path.join(fixture, 'Cargo.toml'), 'utf8'))
      .replace('path = "../../.."', `path = ${JSON.stringify(root)}`);
    await writeFile(path.join(directory, 'Cargo.toml'), manifest);
    const sourcePath = path.join(directory, 'src/live.rs');
    const source = (await readFile(sourcePath, 'utf8')).replaceAll('revision-b', 'revision-a')
      .replace('REVISION B / 绿色', 'REVISION A / 绿色');
    await writeFile(sourcePath, source);
    child = spawn(dx, ['serve', '--desktop', '--renderer', 'native', '--hot-patch',
      '--hot-reload', 'true', '--features', 'hotreload,kit', '--bin', 'live-hot-reload',
      '--interactive', 'false', '--open', 'false'], {
      cwd: directory,
      detached: process.platform !== 'win32',
      env: { ...process.env, CARGO_TARGET_DIR: target },
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    let failure;
    child.on('error', error => { failure = error; });
    for (const stream of [child.stdout, child.stderr]) {
      stream.on('data', data => { log += data; output(data); });
    }
    const waitFor = async (predicate, timeout) => {
      const deadline = Date.now() + timeout;
      while (Date.now() < deadline) {
        if (aborted) throw new Error('Probe interrupted');
        if (failure) throw failure;
        if (child.exitCode !== null || child.signalCode !== null) {
          throw new Error(`dx exited before verification (${child.exitCode ?? child.signalCode})`);
        }
        const result = frames(log).find(predicate);
        if (result) return result;
        await new Promise(resolve => setTimeout(resolve, pollInterval));
      }
      throw new Error(`No matching rendered frame within ${timeout / 1000}s; logs retained in ${target}`);
    };
    const before = await waitFor(frame => frame.revision === 'revision-a' && frame.seconds >= 3, initialTimeout);
    await writeFile(sourcePath, source.replaceAll('revision-a', 'revision-b')
      .replace('REVISION A / 绿色', 'REVISION B / 补丁已应用')
      .replace('bg-emerald-600', 'bg-teal-600'));
    const after = await waitFor(frame => frame.revision === 'revision-b', patchTimeout);
    preserved(before, after);
    const verification = { before, after, fixture: 'gpui-kit 0.9.0 / gpui-pre 0.3.11' };
    await writeFile(path.join(target, 'verification.json'), JSON.stringify(verification, null, 2));
    output(`Kit patch verified: PID ${after.pid}, timer ${before.seconds} → ${after.seconds}, clicks ${before.clicks} → ${after.clicks}\n`);
    succeeded = true;
    return verification;
  } finally {
    if (child?.pid) {
      const stop = signal => {
        try { process.kill(process.platform === 'win32' ? child.pid : -child.pid, signal); }
        catch (error) { if (error.code !== 'ESRCH') throw error; }
      };
      stop('SIGTERM');
      if (child.exitCode === null && child.signalCode === null) {
        await new Promise(resolve => {
          const timeout = setTimeout(resolve, 3000);
          child.once('close', () => { clearTimeout(timeout); resolve(); });
        });
      }
      stop('SIGKILL');
    }
    await writeFile(path.join(target, 'dx.log'), log);
    if (succeeded) await rm(directory, { recursive: true, force: true, maxRetries: 5 });
    else output(`Reproduction source retained at ${directory}\n`);
    process.off('SIGINT', abort);
    process.off('SIGTERM', abort);
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  await probe();
}
