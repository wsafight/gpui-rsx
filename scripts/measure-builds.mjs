import { execFile, spawn } from 'node:child_process';
import { copyFile, mkdir, mkdtemp, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';

const root = fileURLToPath(new URL('..', import.meta.url));
const output = path.join(root, 'target/build-baselines');
await mkdir(output, { recursive: true });
const directory = await mkdtemp(path.join(output, 'measurement-'));
const project = path.join(directory, 'app');
const target = path.join(directory, 'build');
await mkdir(path.join(project, 'src'), { recursive: true });
const sourcePath = path.join(project, 'src/main.rs');
const source = await readFile(path.join(root, 'demo/src/bin/starter.rs'), 'utf8');
const manifest = (await readFile(path.join(root, 'demo/Cargo.toml'), 'utf8'))
  .replace('path = ".."', `path = ${JSON.stringify(root)}`);
await writeFile(path.join(project, 'Cargo.toml'), manifest);
await writeFile(sourcePath, source);
await Promise.all([
  copyFile(path.join(root, 'demo/Cargo.lock'), path.join(project, 'Cargo.lock')),
  copyFile(path.join(root, 'demo/rust-toolchain.toml'), path.join(project, 'rust-toolchain.toml')),
]);
const run = promisify(execFile);
const results = await Promise.allSettled([
  run('rustc', ['--version'], { cwd: project }),
  run('git', ['rev-parse', 'HEAD'], { cwd: root }),
  run('git', ['diff', '--name-only'], { cwd: root }),
]);
if (results.some(result => result.status !== 'fulfilled')) throw new Error('Cannot record build environment');
const report = {
  date: new Date().toISOString(),
  platform: `${process.platform}/${process.arch}`,
  rustc: results[0].value.stdout.trim(),
  revision: results[1].value.stdout.trim(),
  modifiedFiles: results[2].value.stdout.trim().split('\n').filter(Boolean),
  fixture: 'demo/src/bin/starter.rs, demo/Cargo.toml and demo/Cargo.lock; no project rsx.toml',
  command: 'cargo build --locked --offline --timings',
  target,
  note: 'Cold means an empty target directory; registry sources are cached. Durations include rustc, linking and Cargo overhead. Run without other builds for comparisons.',
  measurements: [],
};
console.log(`Build measurements: ${directory}`);
for (const phase of ['cold', 'unchanged', 'edit-view']) {
  if (phase === 'edit-view') await writeFile(sourcePath, source.replace('Count:', 'Total:'));
  console.log(`Measuring ${phase}…`);
  const start = performance.now();
  const child = spawn('cargo', ['build', '--locked', '--offline', '--timings'], {
    cwd: project,
    env: { ...process.env, CARGO_TARGET_DIR: target },
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  let log = '';
  for (const stream of [child.stdout, child.stderr]) stream.on('data', data => { log += data; });
  const code = await new Promise((resolve, reject) => {
    child.on('error', reject);
    child.on('close', resolve);
  });
  await writeFile(path.join(directory, `${phase}.log`), log);
  if (code !== 0) throw new Error(`${phase} build failed; see ${directory}/${phase}.log`);
  const seconds = Number(((performance.now() - start) / 1000).toFixed(3));
  await copyFile(path.join(target, 'cargo-timings/cargo-timing.html'), path.join(directory, `${phase}.html`));
  report.measurements.push({ phase, seconds });
  await writeFile(path.join(directory, 'results.json'), JSON.stringify(report, null, 2));
  console.log(`${phase}: ${seconds}s`);
}
console.log(`Saved results and Cargo timing reports to ${directory}`);
