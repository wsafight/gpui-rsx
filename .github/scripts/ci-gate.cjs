const { categories } = require('./ci-changes.cjs');

function failures(jobs) {
  const problems = [];
  if (jobs.changes?.result !== 'success') return ['Check selection did not succeed'];
  const outputs = jobs.changes.outputs || {};
  if (categories.some(name => !['true', 'false'].includes(outputs[name]))) {
    return ['Check selection returned missing or invalid outputs'];
  }
  const enabled = name => outputs[name] === 'true';
  const rust = enabled('core') || enabled('tooling');
  const required = {
    automation: true,
    msrv: enabled('core'),
    'tooling-msrv': enabled('tooling'),
    test: rust,
    demo: enabled('demo'),
    'gpui-022': enabled('core'),
    fmt: rust || enabled('demo') || enabled('editor'),
    clippy: rust,
    docs: rust,
    security_audit: enabled('security'),
    coverage: rust,
    'gpui-api': enabled('core') || enabled('demo'),
    benchmarks: rust,
    wasm: enabled('docs') || enabled('tooling'),
    site: enabled('docs'),
    tooling: enabled('editor') || enabled('tooling'),
  };
  for (const [name, expected] of Object.entries(required)) {
    const result = jobs[name]?.result;
    if (expected ? result !== 'success' : !['success', 'skipped'].includes(result)) {
      problems.push(`${name}: ${result || 'missing'}${expected ? ' (required for these changes)' : ''}`);
    }
  }
  return problems;
}

if (require.main === module) {
  const problems = failures(JSON.parse(process.env.RESULTS));
  if (problems.length) {
    console.error(problems.join('\n'));
    process.exitCode = 1;
  } else console.log('All selected checks passed.');
}

module.exports = { failures };
