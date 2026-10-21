const fs = require('node:fs');
const { spawnSync } = require('node:child_process');

function parts(version) {
  if (!/^\d+\.\d+\.\d+$/.test(version)) throw new Error(`Expected a stable release: ${version}`);
  return version.split('.').map(Number);
}

function compare(a, b) {
  const av = parts(a), bv = parts(b);
  for (let i = 0; i < 3; i++) if (av[i] !== bv[i]) return av[i] - bv[i];
  return 0;
}

function latestRelease(index, current) {
  const versions = index.trim().split('\n').filter(Boolean).map(line => JSON.parse(line))
    .filter(entry => entry.name === 'gpui-kit' && !entry.yanked && /^\d+\.\d+\.\d+$/.test(entry.vers))
    .map(entry => entry.vers).sort(compare);
  const latest = versions.at(-1);
  if (!latest || compare(latest, current) < 0) throw new Error('Registry has no current or newer stable Kit release');
  return latest;
}

const dependency = /^(gpui\s*=\s*\{\s*package\s*=\s*"gpui-kit",\s*version\s*=\s*")=([^"\n]+)("[^\n]*)$/gm;

function manifestVersion(source) {
  const matches = [...source.matchAll(dependency)];
  if (matches.length !== 1) throw new Error('Expected one exact gpui-kit dependency aliased as gpui');
  parts(matches[0][2]);
  return matches[0][2];
}

function updateManifest(source, version) {
  const current = manifestVersion(source);
  if (compare(version, current) < 0) throw new Error('Refusing to downgrade Kit');
  return source.replace(dependency, (_, start, _old, end) => `${start}=${version}${end}`);
}

async function main() {
  const path = 'demo/Cargo.toml';
  const source = fs.readFileSync(path, 'utf8');
  const response = await fetch('https://index.crates.io/gp/ui/gpui-kit', { signal: AbortSignal.timeout(30000) });
  if (!response.ok) throw new Error(`Kit registry request failed: ${response.status}`);
  const version = latestRelease(await response.text(), manifestVersion(source));
  fs.writeFileSync(path, updateManifest(source, version));
  console.log(`Resolve GPUI Kit ${version} and its matching snapshot`);
  const result = spawnSync('cargo', ['update', '--manifest-path', path,
    '-p', 'gpui-kit', '-p', 'gpui-component', '-p', 'gpui-base', '-p', 'gpui-kit-assets'], { stdio: 'inherit' });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`cargo update exited with ${result.status}`);
}

module.exports = { latestRelease, manifestVersion, updateManifest };
if (require.main === module) main().catch(error => { console.error(error); process.exitCode = 1; });
