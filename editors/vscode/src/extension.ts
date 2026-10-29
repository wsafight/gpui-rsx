import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';
import { compatibleServerVersion, serverVersionRequirement } from './server-version';
let client: LanguageClient | undefined;

async function hasRsxProject(): Promise<boolean> {
  if (!vscode.workspace.workspaceFolders?.length) return false;
  if ((await vscode.workspace.findFiles('**/rsx.toml', '**/{target,node_modules}/**', 1)).length) return true;
  const manifests = await vscode.workspace.findFiles('**/Cargo.toml', '**/{target,node_modules}/**', 200);
  for (const manifest of manifests) {
    const text = Buffer.from(await vscode.workspace.fs.readFile(manifest)).toString('utf8');
    if (/\bgpui-rsx(?:-core|-tools)?\b/.test(text)) return true;
  }
  return false;
}

function installServer(version: string): void {
  const terminal = vscode.window.createTerminal('GPUI RSX');
  terminal.show();
  terminal.sendText(`cargo install cargo-rsx --version '${serverVersionRequirement(version)}' --locked`);
}

/** GUI-launched editors often lack `~/.cargo/bin` on PATH, so prefer Cargo's install location. */
function serverCommand(configured: string): string {
  if (configured !== 'cargo-rsx') return configured;
  const cargoHome = process.env.CARGO_HOME ?? path.join(os.homedir(), '.cargo');
  const installed = path.join(cargoHome, 'bin', process.platform === 'win32' ? 'cargo-rsx.exe' : 'cargo-rsx');
  return fs.existsSync(installed) ? installed : configured;
}

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const version: string = context.extension.packageJSON.version;
  const restart = async (force: boolean) => {
    const config = vscode.workspace.getConfiguration('gpuiRsx');
    await client?.stop();
    client = undefined;
    if (!force && config.get<boolean>('autoStart', true) && !(await hasRsxProject())) return;
    const command = serverCommand(config.get<string>('serverPath', 'cargo-rsx'));
    const server: ServerOptions = { command, args: ['lsp'] };
    const options: LanguageClientOptions = {
      documentSelector: [{ scheme: 'file', language: 'rust' }],
      initializationOptions: { formatting: config.get<boolean>('format.enable', false) },
    };
    const candidate = new LanguageClient('gpuiRsx', 'GPUI RSX', server, options);
    try {
      await candidate.start();
    } catch (error) {
      const missing = (error as NodeJS.ErrnoException)?.code === 'ENOENT';
      const message = missing
        ? `GPUI RSX: ${command} was not found.`
        : `GPUI RSX: the language server failed to start: ${error instanceof Error ? error.message : String(error)}`;
      void vscode.window.showWarningMessage(message, 'Install cargo-rsx', 'Open settings').then((action) => {
        if (action === 'Install cargo-rsx') installServer(version);
        if (action === 'Open settings') void vscode.commands.executeCommand('workbench.action.openSettings', 'gpuiRsx.serverPath');
      });
      return;
    }
    client = candidate;
    const serverVersion = candidate.initializeResult?.serverInfo?.version;
    if (serverVersion && !compatibleServerVersion(serverVersion, version)) {
      void vscode.window
        .showWarningMessage(
          `GPUI RSX: cargo-rsx ${serverVersion} is incompatible with the extension (${version}); use ${serverVersionRequirement(version)}.`,
          'Install cargo-rsx',
        )
        .then((action) => {
          if (action === 'Install cargo-rsx') installServer(version);
        });
    }
  };
  // Serialize restarts so overlapping triggers never leave two servers running.
  let pending: Promise<void> = Promise.resolve();
  const start = (force = false): Promise<void> => {
    pending = pending.catch(() => undefined).then(() => restart(force));
    return pending;
  };
  context.subscriptions.push(vscode.commands.registerCommand('gpuiRsx.restart', () => start(true)));
  context.subscriptions.push(vscode.commands.registerCommand('gpuiRsx.install', () => installServer(version)));
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration('gpuiRsx')) void start();
    }),
  );
  await start();
}
export async function deactivate(): Promise<void> { await client?.stop(); }
