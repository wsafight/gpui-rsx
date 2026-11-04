const assert = require('node:assert/strict');
const path = require('node:path');
const vscode = require('vscode');

async function eventually(label, predicate, timeout = 90_000) {
  const deadline = Date.now() + timeout;
  let lastError;
  while (Date.now() < deadline) {
    try {
      const result = await predicate();
      if (result) return result;
    } catch (error) {
      lastError = error;
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error(`Timed out waiting for ${label}${lastError ? `: ${lastError}` : ''}`);
}

function positionOf(document, text, offset = 0) {
  const start = document.getText().indexOf(text);
  assert.ok(start >= 0, `Missing source text: ${text}`);
  return document.positionAt(start + offset);
}

async function replaceDocument(document, source) {
  const edit = new vscode.WorkspaceEdit();
  edit.replace(document.uri, new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)), source);
  assert.ok(await vscode.workspace.applyEdit(edit));
}

exports.run = async function run() {
  const gpui = vscode.extensions.getExtension('wsafight.gpui-rsx');
  const rust = vscode.extensions.getExtension('rust-lang.rust-analyzer');
  assert.ok(gpui, 'GPUI RSX must be loaded');
  assert.ok(rust, 'rust-analyzer must be loaded');
  const document = await vscode.workspace.openTextDocument(vscode.Uri.file(
    path.join(process.env.GPUI_RSX_INTEGRATION_WORKSPACE, 'src/main.rs'),
  ));
  await vscode.window.showTextDocument(document);
  await Promise.all([gpui.activate(), rust.activate()]);
  assert.ok(gpui.isActive && rust.isActive);

  const diagnostic = await eventually('GPUI RSX diagnostics alongside rust-analyzer', () =>
    vscode.languages.getDiagnostics(document.uri).find((item) => item.source === 'gpui-rsx' && item.message.includes('flex-colum')),
  );
  const fixes = await vscode.commands.executeCommand('vscode.executeCodeActionProvider', document.uri, diagnostic.range, 'quickfix');
  assert.ok(fixes.some((fix) => fix.title.includes('flex-col')), 'RSX quick fix must remain available');
  await eventually('RSX class completion', async () => {
    const completion = await vscode.commands.executeCommand('vscode.executeCompletionItemProvider', document.uri, positionOf(document, 'flex-colum', 8));
    const labels = completion.items.map((item) => typeof item.label === 'string' ? item.label : item.label.label);
    if (!labels.includes('flex-col')) throw new Error(`Completion labels: ${JSON.stringify(labels.slice(0, 20))}`);
    return true;
  });

  await eventually('rust-analyzer native Rust hover', async () => {
    const hovers = await vscode.commands.executeCommand('vscode.executeHoverProvider', document.uri, positionOf(document, 'native_value'));
    return hovers.some((hover) => hover.contents.some((content) => (content.value ?? String(content)).includes('String')));
  });
  await eventually('rust-analyzer native Rust completion', async () => {
    const completions = await vscode.commands.executeCommand('vscode.executeCompletionItemProvider', document.uri, positionOf(document, 'native_value.len', 'native_value.'.length));
    return completions.items.some((item) => (typeof item.label === 'string' ? item.label : item.label.label).startsWith('len'));
  });
  console.log('PASS: both extensions activate; RSX diagnostics, quick fixes and completion coexist with native Rust hover and completion');

  assert.equal(vscode.workspace.getConfiguration('gpuiRsx').get('format.enable'), false);
  await eventually('rust-analyzer formatting with GPUI formatting disabled', async () => {
    await vscode.commands.executeCommand('editor.action.formatDocument');
    return document.getText().includes('let native_value = String::from');
  });

  await vscode.workspace.getConfiguration('gpuiRsx').update('format.enable', true, vscode.ConfigurationTarget.Workspace);
  await vscode.commands.executeCommand('gpuiRsx.restart');
  await vscode.workspace.getConfiguration().update('[rust]', { 'editor.defaultFormatter': 'wsafight.gpui-rsx' }, vscode.ConfigurationTarget.Workspace);
  await replaceDocument(document, document.getText()
    .replace('let native_value =', 'let native_value=')
    .replace(/rsx_strict!\s*\{[\s\S]*?\};/, 'rsx_strict!{<div class="flex-colum gap-4"/>};'));
  await eventually('opt-in GPUI RSX formatter after restart', async () => {
    await vscode.commands.executeCommand('editor.action.formatDocument');
    return /\n\s+<div class="flex-colum gap-4" \/>\n/.test(document.getText());
  });
  assert.ok(document.getText().includes('let native_value='), 'RSX formatting preserves surrounding Rust');
  await vscode.workspace.getConfiguration('gpuiRsx').update('format.enable', false, vscode.ConfigurationTarget.Workspace);
  await vscode.commands.executeCommand('gpuiRsx.restart');
  await vscode.workspace.getConfiguration().update('[rust]', { 'editor.defaultFormatter': 'rust-lang.rust-analyzer' }, vscode.ConfigurationTarget.Workspace);
  await eventually('rust-analyzer formatting after GPUI formatting is disabled again', async () => {
    await vscode.commands.executeCommand('editor.action.formatDocument');
    return document.getText().includes('let native_value =');
  });
  console.log('PASS: rust-analyzer formatting stays the default; explicit GPUI formatting and configuration restarts work');
};
