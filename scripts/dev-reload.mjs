import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { readdir, stat, writeFile } from 'node:fs/promises';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';

const ignored = new Set(['target', 'node_modules', '.git', 'dist', '.vscode-test']);

export async function snapshot(roots) {
  const files = [];
  const visited = new Set();
  async function visit(root, directory = false) {
    root = path.resolve(root);
    if (visited.has(root)) return;
    visited.add(root);
    let info;
    try { if (!directory) info = await stat(root); } catch (error) {
      if (error.code === 'ENOENT') return;
      throw error;
    }
    if (directory || info.isDirectory()) {
      let entries;
      try { entries = await readdir(root, { withFileTypes: true }); } catch (error) {
        if (error.code === 'ENOENT') return;
        throw error;
      }
      for (const entry of entries) {
        if (entry.isSymbolicLink() || ignored.has(entry.name)) continue;
        const childDirectory = entry.isDirectory();
        if (childDirectory || /\.(rs|toml|lock)$/.test(entry.name)) {
          await visit(path.join(root, entry.name), childDirectory);
        }
      }
    } else if (/\.(rs|toml|lock)$/.test(root)) {
      files.push([root, info.mtimeMs, info.size]);
    }
  }
  for (const root of roots) await visit(root);
  return JSON.stringify(files.sort(([left], [right]) => left.localeCompare(right)));
}

export async function stop(child) {
  if (!child?.pid || child.exitCode !== null || child.signalCode !== null) return;
  const exited = once(child, 'exit');
  child.kill('SIGTERM');
  const force = setTimeout(() => child.kill('SIGKILL'), 2_000);
  try { await exited; } finally { clearTimeout(force); }
}

export class ReloadSession {
  constructor({ build, launch, report = () => {} }) {
    this.build = build;
    this.launch = launch;
    this.report = report;
    this.pending = undefined;
    this.running = undefined;
    this.child = undefined;
    this.closed = false;
    this.abort = new AbortController();
  }

  request(changedAt = performance.now()) {
    if (this.closed) return Promise.resolve();
    this.pending ??= changedAt;
    if (!this.running) {
      this.running = this.drain().finally(() => { this.running = undefined; });
    }
    return this.running;
  }

  async drain() {
    let successful;
    while (this.pending !== undefined && !this.closed) {
      const changedAt = this.pending;
      this.pending = undefined;
      const started = performance.now();
      let executable;
      try {
        executable = await this.build(this.abort.signal);
      } catch (error) {
        this.report({ ok: false, buildMs: performance.now() - started, error: String(error) });
        if (!this.closed && !this.child && successful && this.pending === undefined) {
          this.startApplication(successful, true);
        }
        continue;
      }
      const built = performance.now();
      successful = { executable, changedAt, started, built };
      if (this.closed) break;
      if (this.pending !== undefined) {
        this.pending = Math.min(changedAt, this.pending);
        continue;
      }
      await stop(this.child);
      this.child = undefined;
      if (this.closed) break;
      if (this.pending !== undefined) {
        this.pending = Math.min(changedAt, this.pending);
        continue;
      }
      this.startApplication(successful);
    }
  }

  startApplication({ executable, changedAt, started, built }, fallback = false) {
    const launched = performance.now();
    const child = this.launch(executable);
    this.child = child;
    let buffered = '';
    let ready = false;
    child.stdout?.on('data', (data) => {
      const output = buffered + data.toString();
      buffered = output.slice(-4096);
      if (!ready && output.includes('GPUI_RSX_READY')) {
        ready = true;
        this.report({ ok: true, ready: true, fallback, buildMs: built - started,
          restartMs: performance.now() - launched, editToReadyMs: performance.now() - changedAt });
      }
    });
    child.on('error', (error) => this.report({ ok: false, error: String(error) }));
    this.report({ ok: true, ready: false, fallback, buildMs: built - started, pid: child.pid });
  }

  async close() {
    this.closed = true;
    this.abort.abort();
    this.pending = undefined;
    try { await this.running; } finally { await stop(this.child); }
  }
}

async function cargoBuild(manifest, binary, toolchain, offline, signal) {
  const args = [...(toolchain ? [`+${toolchain}`] : []), 'build', '--manifest-path', manifest,
    '--bin', binary, '--locked', '--message-format=json-render-diagnostics', ...(offline ? ['--offline'] : [])];
  const child = spawn('cargo', args, { signal, stdio: ['ignore', 'pipe', 'inherit'] });
  let output = '';
  child.stdout.on('data', (data) => { output += data; });
  const [code] = await once(child, 'close');
  if (code !== 0) throw new Error(`cargo build exited with ${code}`);
  for (const line of output.split('\n').reverse()) {
    try {
      const message = JSON.parse(line);
      if (message.reason === 'compiler-artifact' && message.target.name === binary && message.executable) return message.executable;
    } catch {}
  }
  throw new Error('Cargo did not return a binary executable');
}

async function main(args) {
  let manifest = 'demo/Cargo.toml';
  let binary = 'counter';
  let toolchain = '1.98.1';
  let reportFile;
  let offline = false;
  const roots = [];
  for (let index = 0; index < args.length; index += 1) {
    const flag = args[index];
    if (flag === '--help') {
      console.log('Usage: node scripts/dev-reload.mjs [--manifest-path FILE] [--bin NAME] [--toolchain VERSION] [--watch PATH] [--report FILE] [--offline]');
      return;
    }
    if (flag === '--offline') { offline = true; continue; }
    const value = args[++index];
    if (!value) throw new Error(`Missing value for ${flag}`);
    if (flag === '--manifest-path') manifest = value;
    else if (flag === '--bin') binary = value;
    else if (flag === '--toolchain') toolchain = value;
    else if (flag === '--watch') roots.push(value);
    else if (flag === '--report') reportFile = value;
    else throw new Error(`Unknown option ${flag}`);
  }
  if (!roots.length) roots.push(path.dirname(manifest), 'Cargo.toml', 'src',
    'crates/gpui-rsx-core/Cargo.toml', 'crates/gpui-rsx-core/src');
  const metrics = [];
  const session = new ReloadSession({
    build: (signal) => cargoBuild(manifest, binary, toolchain, offline, signal),
    launch: (executable) => {
      const child = spawn(executable, [], { env: { ...process.env, GPUI_RSX_DEV_READY: '1' }, stdio: ['ignore', 'pipe', 'inherit'] });
      child.stdout.pipe(process.stdout);
      return child;
    },
    report: (entry) => { metrics.push(entry); console.error(JSON.stringify(entry)); },
  });
  let previous = await snapshot(roots);
  let polling = false;
  let failure;
  const timer = setInterval(async () => {
    if (polling || failure) return;
    polling = true;
    try {
      const current = await snapshot(roots);
      if (current !== previous) {
        previous = current;
        session.request().catch((error) => {
          failure = error;
          process.emit('SIGTERM');
        });
      }
    } catch (error) {
      failure = error;
      process.emit('SIGTERM');
    } finally { polling = false; }
  }, 300);
  await new Promise((resolve, reject) => {
    process.once('SIGINT', resolve);
    process.once('SIGTERM', resolve);
    session.request().catch(reject);
  }).finally(async () => {
    clearInterval(timer);
    await session.close();
    if (reportFile) await writeFile(reportFile, JSON.stringify(metrics, null, 2) + '\n');
  });
  if (failure) throw failure;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).catch((error) => { console.error(error); process.exitCode = 1; });
}
