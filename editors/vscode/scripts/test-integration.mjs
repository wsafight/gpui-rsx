import { execFileSync } from 'node:child_process';
import { cp, mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import * as os from 'node:os';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';
import { downloadAndUnzipVSCode, resolveCliArgsFromVSCodeExecutablePath, runTests } from '@vscode/test-electron';

const extensionRoot = fileURLToPath(new URL('..', import.meta.url));
const repository = path.resolve(extensionRoot, '../..');
const temporary = await mkdtemp(path.join(os.tmpdir(), 'gpui-rsx-editor-'));
const workspace = path.join(temporary, 'workspace');
const extensions = path.join(temporary, 'extensions');
const profile = path.join(temporary, 'profile');
const cargoRsx = process.env.GPUI_RSX_SERVER_PATH ?? path.join(repository, 'target/debug', process.platform === 'win32' ? 'cargo-rsx.exe' : 'cargo-rsx');

try {
  const executable = process.env.GPUI_RSX_VSCODE_EXECUTABLE ?? await downloadAndUnzipVSCode({
    version: '1.96.4',
    cachePath: path.join(os.tmpdir(), 'gpui-rsx-vscode-cache'),
  });
  await Promise.all([
    mkdir(path.join(workspace, 'src'), { recursive: true }),
    mkdir(path.join(workspace, '.vscode'), { recursive: true }),
    mkdir(extensions, { recursive: true }),
  ]);
  if (process.env.GPUI_RSX_RUST_ANALYZER_EXTENSION) {
    await cp(process.env.GPUI_RSX_RUST_ANALYZER_EXTENSION, path.join(extensions, 'rust-lang.rust-analyzer'), { recursive: true });
  } else {
    const [cli, ...args] = resolveCliArgsFromVSCodeExecutablePath(executable, { reuseMachineInstall: true });
    execFileSync(cli, [...args, '--extensions-dir', extensions, '--user-data-dir', profile,
      '--install-extension', 'rust-lang.rust-analyzer@0.3.2887', '--force'], {
      stdio: 'inherit',
      timeout: 180_000,
      shell: process.platform === 'win32',
    });
  }
  const tomlPath = (value) => JSON.stringify(value.replaceAll('\\', '/'));
  await Promise.all([
    writeFile(path.join(workspace, 'Cargo.toml'), `[package]
name = "rsx-editor-integration"
version = "0.0.0"
edition = "2024"

[dependencies]
gpui-rsx = { path = ${tomlPath(repository)} }
gpui = { package = "gpui-rsx-gpui-stub", path = ${tomlPath(path.join(repository, 'crates/gpui-stub'))} }

[workspace]
`),
    writeFile(path.join(workspace, 'src/main.rs'), `use gpui::{div, prelude::*};
use gpui_rsx::rsx_strict;

fn main() {
    let native_value=String::from("ready");
    let _native_len = native_value.len();
    let _ = rsx_strict!{<div class="flex-colum gap-4"/>};
}
`),
    writeFile(path.join(workspace, '.vscode/settings.json'), JSON.stringify({
      'gpuiRsx.serverPath': cargoRsx,
      'rust-analyzer.cargo.offline': true,
      'rust-analyzer.checkOnSave': false,
      'rust-analyzer.procMacro.enable': false,
      '[rust]': { 'editor.defaultFormatter': 'rust-lang.rust-analyzer' },
      'telemetry.telemetryLevel': 'off',
    }, null, 2)),
  ]);
  await runTests({
    vscodeExecutablePath: executable,
    extensionDevelopmentPath: extensionRoot,
    extensionTestsPath: path.join(extensionRoot, 'tests/extension-host.cjs'),
    extensionTestsEnv: { GPUI_RSX_INTEGRATION_WORKSPACE: workspace },
    launchArgs: [workspace, '--extensions-dir', extensions, '--user-data-dir', profile,
      '--disable-workspace-trust', '--skip-welcome', '--skip-release-notes', '--disable-updates'],
  });
  await rm(temporary, { recursive: true, force: true, maxRetries: 10, retryDelay: 200 });
} catch (error) {
  console.error(`Editor integration artifacts: ${temporary}`);
  throw error;
}
