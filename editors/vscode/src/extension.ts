import * as vscode from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';
let client: LanguageClient | undefined;

async function hasRsxProject(): Promise<boolean> {
  if (!vscode.workspace.workspaceFolders?.length) return false;
  if ((await vscode.workspace.findFiles('**/rsx.toml', '**/{target,node_modules}/**', 1)).length) return true;
  const manifests = await vscode.workspace.findFiles('**/Cargo.toml', '**/{target,node_modules}/**', 20);
  for (const manifest of manifests) {
    const text = Buffer.from(await vscode.workspace.fs.readFile(manifest)).toString('utf8');
    if (/\bgpui-rsx(?:-core|-tools)?\b/.test(text)) return true;
  }
  return false;
}

function installServer(): void {
  const terminal = vscode.window.createTerminal('GPUI RSX');
  terminal.show();
  terminal.sendText('cargo install cargo-rsx --version 0.8.1 --locked');
}

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const start = async (force = false) => {
    const autoStart = vscode.workspace.getConfiguration('gpuiRsx').get<boolean>('autoStart', true);
    if (!force && autoStart && !(await hasRsxProject())) return;
    await client?.stop();
    const command = vscode.workspace.getConfiguration('gpuiRsx').get<string>('serverPath', 'cargo-rsx');
    const server: ServerOptions = { command, args: ['lsp'] };
    const options: LanguageClientOptions = {
      documentSelector: [{ scheme: 'file', language: 'rust' }],
      synchronize: { fileEvents: vscode.workspace.createFileSystemWatcher('**/rsx.toml') },
    };
    client = new LanguageClient('gpuiRsx', 'GPUI RSX', server, options);
    try { await client.start(); }
    catch {
      const action = await vscode.window.showWarningMessage(
        'GPUI RSX: cargo-rsx was not found.',
        'Install cargo-rsx',
        'Open settings',
      );
      if (action === 'Install cargo-rsx') installServer();
      if (action === 'Open settings') void vscode.commands.executeCommand('workbench.action.openSettings', 'gpuiRsx.serverPath');
    }
  };
  context.subscriptions.push(vscode.commands.registerCommand('gpuiRsx.restart', () => start(true)));
  context.subscriptions.push(vscode.commands.registerCommand('gpuiRsx.install', installServer));
  await start();
}
export async function deactivate(): Promise<void> { await client?.stop(); }
