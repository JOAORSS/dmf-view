import * as vscode from 'vscode';
import * as cp from 'child_process';
import * as path from 'path';
import * as fs from 'fs';

export function activate(context: vscode.ExtensionContext) {
  const cmd = vscode.commands.registerCommand('dfmPreview.open', () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showWarningMessage('No active editor found.');
      return;
    }

    const dfmPath = editor.document.uri.fsPath;
    if (!dfmPath.endsWith('.dfm')) {
      vscode.window.showWarningMessage('Current file is not a .dfm file.');
      return;
    }

    const scannerPath = vscode.workspace
      .getConfiguration('dfmPreview')
      .get<string>('scannerPath', 'dfm_scanner');

    cp.execFile(scannerPath, ['--json', dfmPath], (err, stdout, stderr) => {
      if (err) {
        vscode.window.showErrorMessage(
          `dfm_scanner failed: ${stderr || err.message}`
        );
        return;
      }

      let componentTree: unknown;
      try {
        componentTree = JSON.parse(stdout);
      } catch {
        vscode.window.showErrorMessage(
          'Invalid JSON returned by dfm_scanner'
        );
        return;
      }

      // Load component aliases if available
      const dfmDir = path.dirname(dfmPath);
      const aliasesPath = path.join(dfmDir, 'component-aliases.json');
      let aliases: Record<string, string> = {};

      if (fs.existsSync(aliasesPath)) {
        try {
          const raw = fs.readFileSync(aliasesPath, 'utf-8');
          const parsed = JSON.parse(raw);
          aliases = parsed.aliases ?? {};
        } catch {
          // aliases empty, continue without
        }
      }

      const panel = vscode.window.createWebviewPanel(
        'dfmPreview',
        `Preview: ${path.basename(dfmPath)}`,
        vscode.ViewColumn.Beside,
        { enableScripts: true }
      );

      panel.webview.html = buildWebviewHtml(
        panel.webview,
        context.extensionUri,
        componentTree,
        aliases
      );
    });
  });

  const cmdAliases = vscode.commands.registerCommand('dfmPreview.generateAliases', () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showWarningMessage('No active editor found.');
      return;
    }

    const dfmPath = editor.document.uri.fsPath;
    const scannerPath = vscode.workspace
      .getConfiguration('dfmPreview')
      .get<string>('scannerPath', 'dfm_scanner');

    vscode.window.withProgress(
      { location: vscode.ProgressLocation.Notification, title: 'Gerando aliases...' },
      () => new Promise<void>((resolve) => {
        cp.execFile(scannerPath, ['--aliases', '--save', dfmPath], (err, stdout, stderr) => {
          if (err) {
            vscode.window.showErrorMessage(`Erro ao gerar aliases: ${stderr || err.message}`);
          } else {
            vscode.window.showInformationMessage(`Aliases gerados: ${stdout.trim()}`);
          }
          resolve();
        });
      })
    );
  });

  context.subscriptions.push(cmd);
  context.subscriptions.push(cmdAliases);
}

function buildWebviewHtml(
  webview: vscode.Webview,
  extensionUri: vscode.Uri,
  componentTree: unknown,
  aliases: Record<string, string> = {}
): string {
  const rendererUri = webview.asWebviewUri(
    vscode.Uri.joinPath(extensionUri, 'renderer', 'renderer.js')
  );
  const stylesUri = webview.asWebviewUri(
    vscode.Uri.joinPath(extensionUri, 'renderer', 'styles.css')
  );

  const jsonStr = JSON.stringify(componentTree);
  const aliasesStr = JSON.stringify(aliases);

  return `<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <link rel="stylesheet" href="${stylesUri}">
</head>
<body>
  <script src="${rendererUri}"></script>
  <script>
    var COMPONENT_ALIASES = ${aliasesStr};
    var json = ${jsonStr};
    document.body.innerHTML = renderDFM(json);
  </script>
</body>
</html>`;
}

export function deactivate() {}
