import * as vscode from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';
let client: LanguageClient | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const start = async () => {
    await client?.stop();
    const command = vscode.workspace.getConfiguration('gpuiRsx').get<string>('serverPath', 'cargo-rsx');
    const server: ServerOptions = { command, args: ['lsp'] };
    const options: LanguageClientOptions = {
      documentSelector: [{ scheme: 'file', language: 'rust' }],
      synchronize: { fileEvents: vscode.workspace.createFileSystemWatcher('**/rsx.toml') },
    };
    client = new LanguageClient('gpuiRsx', 'GPUI RSX', server, options);
    try { await client.start(); }
    catch { void vscode.window.showErrorMessage('GPUI RSX: install the language server with `cargo install cargo-rsx`, or configure gpuiRsx.serverPath.'); }
  };
  context.subscriptions.push(vscode.commands.registerCommand('gpuiRsx.restart', start));
  await start();
}
export async function deactivate(): Promise<void> { await client?.stop(); }
