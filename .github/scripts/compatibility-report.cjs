module.exports = async ({ github, context, core, env = process.env }) => {
  const { owner, repo } = context.repo;
  const title = 'GPUI compatibility tracking';
  const ok = env.PREPARE_RESULT === 'success'
    && env.VALIDATE_RESULT === 'success'
    && env.REPAIR_RESULT === 'success';
  const issues = (await github.paginate(github.rest.issues.listForRepo, {
    owner, repo, state: 'open', per_page: 100,
  })).filter(issue => !issue.pull_request && issue.user.type === 'Bot'
    && (issue.title === title || issue.labels.some(label => label.name === 'gpui-compatibility')));
  const run = `${context.serverUrl}/${owner}/${repo}/actions/runs/${context.runId}`;
  const body = [
    `GPUI Kit: \`${env.KIT_VERSION || 'unresolved'}\``,
    `GPUI: \`${env.GPUI_VERSION || 'unresolved'}\``,
    `gpui-component: \`${env.COMPONENT_VERSION || 'unresolved'}\``,
    `Prepare: ${env.PREPARE_RESULT}; validate: ${env.VALIDATE_RESULT}; repair PR: ${env.REPAIR_RESULT}.`,
    env.REPAIR_URL ? `Verified repair: ${env.REPAIR_URL}` : '',
    `Workflow: ${run}`,
  ].filter(Boolean).join('\n\n');
  let tracker = issues.find(issue => issue.title === title) || issues[0];
  if (!ok) {
    if (tracker) {
      await github.rest.issues.update({ owner, repo, issue_number: tracker.number, title, body });
      await github.rest.issues.createComment({ owner, repo, issue_number: tracker.number, body });
    } else {
      const result = await github.rest.issues.create({ owner, repo, title, body });
      tracker = result.data;
    }
  }
  // Consolidate historical bot issues, preserving issues opened by people.
  for (const issue of issues) {
    if (!ok && issue.number === tracker.number) continue;
    await github.rest.issues.createComment({
      owner, repo, issue_number: issue.number,
      body: ok ? `Compatibility verified.\n\n${body}` : `Tracking continues in #${tracker.number}.`,
    });
    await github.rest.issues.update({
      owner, repo, issue_number: issue.number, state: 'closed',
      state_reason: ok ? 'completed' : 'not_planned',
    });
  }
  if (!ok) core.setFailed('Compatibility requires attention; see the single tracking issue.');
};
