const { test } = require('node:test');
const assert = require('node:assert/strict');
const report = require('./compatibility-report.cjs');

const context = { repo: { owner: 'owner', repo: 'repo' }, serverUrl: 'https://github.com', runId: 123 };
const botIssue = (number, title) => ({ number, title, user: { type: 'Bot' }, labels: [{ name: 'gpui-compatibility' }] });

function mock(issues) {
  const calls = { updates: [], comments: [], creates: [], failures: [] };
  const api = {
    listForRepo() {},
    async update(input) { calls.updates.push(input); },
    async createComment(input) { calls.comments.push(input); },
    async create(input) { calls.creates.push(input); return { data: { number: 999 } }; },
  };
  return {
    calls, context,
    github: { rest: { issues: api }, async paginate() { return issues; } },
    core: { setFailed(message) { calls.failures.push(message); } },
  };
}

const success = {
  PREPARE_RESULT: 'success', VALIDATE_RESULT: 'success', REPAIR_RESULT: 'success',
  KIT_VERSION: 'gpui-kit@0.8.0', GPUI_VERSION: 'gpui-pre@0.3.9', COMPONENT_VERSION: 'gpui-component@0.8.0',
};

test('failures across revisions reuse one tracker and preserve human issues', async () => {
  const human = { ...botIssue(3, 'My compatibility report'), user: { type: 'User' } };
  const mocks = mock([botIssue(1, 'GPUI compatibility: old'), botIssue(2, 'GPUI compatibility tracking'), human]);
  await report({ ...mocks, env: { ...success, VALIDATE_RESULT: 'failure' } });
  assert.equal(mocks.calls.creates.length, 0);
  assert.deepEqual(mocks.calls.updates.map(x => [x.issue_number, x.state]), [[2, undefined], [1, 'closed']]);
  assert.equal(mocks.calls.comments[1].body, 'Tracking continues in #2.');
  assert.equal(mocks.calls.failures.length, 1);
});

test('a failed PR publication creates a tracker even when builds passed', async () => {
  const mocks = mock([]);
  await report({ ...mocks, env: { ...success, REPAIR_RESULT: 'failure' } });
  assert.equal(mocks.calls.creates[0].title, 'GPUI compatibility tracking');
  assert.equal(mocks.calls.failures.length, 1);
});

test('verified update closes all historical bot reports, preserving PRs and human issues', async () => {
  const mocks = mock([
    botIssue(1, 'GPUI compatibility: old'),
    botIssue(2, 'GPUI compatibility tracking'),
    { ...botIssue(3, 'User report'), user: { type: 'User' } },
    { ...botIssue(4, 'Repair PR'), pull_request: {} },
  ]);
  await report({ ...mocks, env: { ...success, REPAIR_URL: 'https://github.com/owner/repo/pull/5' } });
  assert.deepEqual(mocks.calls.updates.map(x => x.issue_number), [1, 2]);
  assert.equal(mocks.calls.failures.length, 0);
  assert.equal(mocks.calls.creates.length, 0);
  assert.match(mocks.calls.comments[0].body, /pull\/5/);
  for (const version of [success.KIT_VERSION, success.GPUI_VERSION, success.COMPONENT_VERSION]) {
    assert.ok(mocks.calls.comments[0].body.includes(version));
  }
});
