import * as vscode from "vscode";
import * as os from "os";

import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export async function activate(context: vscode.ExtensionContext) {
  const name = "Echolysis";
  const outputChannel = vscode.window.createOutputChannel(name);

  context.subscriptions.push(outputChannel);

  const restartCommand = vscode.commands.registerCommand(
    "echolysis.restart",
    async () => {
      if (client && client.needsStop()) {
        await client.stop();
      }

      try {
        client = await createClient(context, name, outputChannel);
      } catch (err) {
        vscode.window.showErrorMessage(
          `${err instanceof Error ? err.message : err}`,
        );
        return;
      }

      await client.start();
    },
  );
  context.subscriptions.push(restartCommand);

  await vscode.commands.executeCommand("echolysis.restart");
}

async function createClient(
  context: vscode.ExtensionContext,
  name: string,
  outputChannel: vscode.OutputChannel,
): Promise<LanguageClient> {
  const env = { ...process.env };

  let config = vscode.workspace.getConfiguration("echolysis");
  let path = await getServerPath(context, config);

  outputChannel.appendLine("Using echolysis lsp: " + path);

  env.RUST_LOG = config.get("logLevel");

  const run: Executable = {
    command: path,
    options: { env: env },
  };

  const serverOptions: ServerOptions = {
    run: run,
    debug: run,
  };

  const clientOptions: LanguageClientOptions = {
    // Register the server for all documents
    documentSelector: [
      { scheme: "file", language: "rust" },
      { scheme: "file", language: "python" },
    ],
    outputChannel: outputChannel,
    traceOutputChannel: outputChannel,
    initializationOptions: {
      config: config.get("config") ? config.get("config") : null,
      diagnosticSeverity: config.get("diagnosticSeverity"),
    },
  };

  return new LanguageClient(
    name.toLowerCase(),
    name,
    serverOptions,
    clientOptions,
  );
}

async function getServerPath(
  context: vscode.ExtensionContext,
  config: vscode.WorkspaceConfiguration,
): Promise<string> {
  let path =
    process.env.ECHOLYSIS_LSP_PATH ?? config.get<null | string>("path");

  if (path) {
    if (path.startsWith("~/")) {
      path = os.homedir() + path.slice("~".length);
    }
    const pathUri = vscode.Uri.file(path);

    return await vscode.workspace.fs.stat(pathUri).then(
      () => pathUri.fsPath,
      () => {
        throw new Error(
          `${path} does not exist. Please check echolysis.path in Settings.`,
        );
      },
    );
  }

  const ext = process.platform === "win32" ? ".exe" : "";
  const bundled = vscode.Uri.joinPath(
    context.extensionUri,
    "bundled",
    `echolysis-lsp${ext}`,
  );

  return await vscode.workspace.fs.stat(bundled).then(
    () => bundled.fsPath,
    () => {
      throw new Error(
        "Unfortunately we don't ship binaries for your platform yet. " +
          "Try specifying echolysis.path in Settings. " +
          "Or raise an issue [here](https://github.com/yingmanwumen/echolysis/issues) " +
          "to request a binary for your platform.",
      );
    },
  );
}

// This method is called when your extension is deactivated
export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}

