const assert = require('node:assert/strict');
const test = require('node:test');
const { failures } = require('./ci-gate.cjs');
const { classify } = require('./ci-changes.cjs');

function docsRun() {
  const jobs = Object.fromEntries(['msrv', 'tooling-msrv', 'test', 'demo', 'gpui-022', 'fmt',
    'clippy', 'docs', 'security_audit', 'coverage', 'gpui-api', 'benchmarks', 'tooling']
    .map(name => [name, { result: 'skipped' }]));
  for (const name of ['automation', 'wasm', 'site']) jobs[name] = { result: 'success' };
  jobs.changes = { result: 'success', outputs: Object.fromEntries(
    Object.entries(classify(['docs/src/styles/custom.css'])).map(([name, value]) => [name, String(value)]),
  ) };
  return jobs;
}

test('documentation-only runs accept intentional skips', () => {
  assert.deepEqual(failures(docsRun()), []);
});

test('required tasks cannot silently skip, cancel, fail or disappear', () => {
  for (const result of ['skipped', 'cancelled', 'failure', undefined]) {
    const jobs = docsRun();
    jobs.site = { result };
    assert.match(failures(jobs).join('\n'), /site:.*required/);
  }
});

test('selection failures and invalid outputs fail closed', () => {
  const jobs = docsRun();
  jobs.changes.result = 'failure';
  assert.ok(failures(jobs).length);
  jobs.changes.result = 'success';
  delete jobs.changes.outputs.core;
  assert.match(failures(jobs).join('\n'), /missing or invalid/);
});

test('unexpected failures cannot be hidden by path selection', () => {
  const jobs = docsRun();
  jobs.test.result = 'failure';
  assert.match(failures(jobs).join('\n'), /test: failure/);
});

test('full runs require every job, including both performance matrices', () => {
  const jobs = docsRun();
  jobs.changes.outputs = Object.fromEntries(Object.keys(jobs.changes.outputs).map(name => [name, 'true']));
  assert.ok(failures(jobs).length > 5);
  for (const job of Object.values(jobs)) job.result = 'success';
  assert.deepEqual(failures(jobs), []);
  jobs.benchmarks.result = 'skipped';
  assert.match(failures(jobs).join('\n'), /benchmarks/);
});
