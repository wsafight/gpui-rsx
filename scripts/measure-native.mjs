import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { cpus, release as osRelease } from 'node:os';
import { setTimeout as delay } from 'node:timers/promises';
import { fileURLToPath, pathToFileURL } from 'node:url';

export function validate(report, rows) {
  assert.equal(report.schema, 1);
  assert.equal(report.scenario, 'scroll-and-select');
  assert.equal(report.records, rows);
  assert.equal(report.sample_frames, 60);
  assert.ok(Number.isInteger(report.max_rendered_rows) && report.max_rendered_rows > 0);
  assert.ok(report.max_rendered_rows < 100, 'Virtualization built too many rows');
  for (const name of ['draw_ms', 'present_interval_ms', 'dirty_to_present_ms', 'frame_callback_interval_ms']) {
    const metric = report[name];
    assert.ok(Number.isInteger(metric?.count) && metric.count > 0, `${name} has no native samples`);
    assert.ok(Number.isFinite(metric.p50) && metric.p50 >= 0);
    assert.ok(Number.isFinite(metric.p95) && metric.p95 >= metric.p50);
  }
  return report;
}

export async function probe({ command, args = [], rows, output, timeout = 30_000, env = {} }) {
  await mkdir(path.dirname(output), { recursive: true });
  await rm(output, { force: true });
  let log = '';
  let timer;
  let force;
  let abort;
  try {
    await new Promise((resolve, reject) => {
      let failure;
      const child = spawn(command, args, {
        env: { ...process.env, ...env, RSX_DEMO_ROWS: String(rows), RSX_NATIVE_REPORT: output },
        stdio: ['ignore', 'pipe', 'pipe'],
      });
      for (const stream of [child.stdout, child.stderr]) {
        stream.setEncoding('utf8');
        stream.on('data', chunk => { log += chunk; });
      }
      child.once('error', reject);
      child.once('close', (code, signal) => {
        if (failure) reject(failure);
        else if (code === 0) resolve();
        else reject(new Error(`Native probe exited ${code ?? signal}`));
      });
      const stop = message => {
        failure = new Error(message);
        child.kill('SIGTERM');
        force = setTimeout(() => child.kill('SIGKILL'), 1000);
        force.unref();
      };
      abort = () => stop('Native probe interrupted');
      process.once('SIGINT', abort);
      process.once('SIGTERM', abort);
      timer = setTimeout(() => stop('Native probe timed out; a visible desktop session is required'), timeout);
    });
    return validate(JSON.parse(await readFile(output, 'utf8')), rows);
  } finally {
    clearTimeout(timer);
    clearTimeout(force);
    if (abort) {
      process.off('SIGINT', abort);
      process.off('SIGTERM', abort);
    }
    await writeFile(`${output}.log`, log);
  }
}

async function main() {
  const root = fileURLToPath(new URL('..', import.meta.url));
  const release = process.argv.slice(2).includes('--release');
  assert.ok(process.argv.slice(2).every(argument => argument === '--release'), 'Only --release is accepted');
  const build = spawnSync('cargo', ['+1.98.1', 'build', '--manifest-path', 'demo/Cargo.toml',
    '--locked', '--features', 'native-profiler', '--bin', 'incident_console', ...(release ? ['--release'] : [])],
  { cwd: root, stdio: 'inherit' });
  if (build.error) throw build.error;
  assert.equal(build.status, 0, 'Native profiler build failed');
  const target = path.join(root, 'target/native-baselines', new Date().toISOString().replaceAll(':', '-'));
  const reports = [];
  for (const rows of [1000, 10_000, 100_000]) {
    await delay(500);
    const report = await probe({
      command: path.join(root, 'demo/target', release ? 'release' : 'debug',
        `incident_console${process.platform === 'win32' ? '.exe' : ''}`),
      rows,
      output: path.join(target, `${rows}.json`),
    });
    reports.push(report);
    console.log(`${rows} rows: draw p95 ${report.draw_ms.p95.toFixed(3)} ms; built at most ${report.max_rendered_rows} rows`);
  }
  const revision = spawnSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' });
  const changed = spawnSync('git', ['diff', '--name-only', 'HEAD'], { cwd: root, encoding: 'utf8' });
  const untracked = spawnSync('git', ['ls-files', '--others', '--exclude-standard'], { cwd: root, encoding: 'utf8' });
  const rustc = spawnSync('rustc', ['+1.98.1', '--version'], { encoding: 'utf8' });
  await writeFile(path.join(target, 'results.json'), JSON.stringify({
    date: new Date().toISOString(), platform: `${process.platform}/${process.arch}`,
    cpu: cpus()[0]?.model, osRelease: osRelease(),
    revision: revision.stdout.trim(), modifiedFiles: [...new Set(`${changed.stdout}\n${untracked.stdout}`.split('\n').filter(Boolean))].sort(),
    rustc: rustc.stdout.trim(), reports,
  }, null, 2));
  console.log(`Native measurements saved in ${target}`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  main().catch(error => { console.error(error); process.exitCode = 1; });
}
