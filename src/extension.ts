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

      const panel = vscode.window.createWebviewPanel(
        'dfmPreview',
        `Preview: ${path.basename(dfmPath)}`,
        vscode.ViewColumn.Beside,
        { enableScripts: true }
      );

      panel.webview.html = buildWebviewHtml(
        panel.webview,
        context.extensionUri,
        componentTree
      );
    });
  });

  context.subscriptions.push(cmd);
}

function buildWebviewHtml(
  webview: vscode.Webview,
  extensionUri: vscode.Uri,
  componentTree: unknown
): string {
  const rendererUri = webview.asWebviewUri(
    vscode.Uri.joinPath(extensionUri, 'renderer', 'renderer.js')
  );
  const stylesUri = webview.asWebviewUri(
    vscode.Uri.joinPath(extensionUri, 'renderer', 'styles.css')
  );

  const jsonStr = JSON.stringify(componentTree);

  return `<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <link rel="stylesheet" href="${stylesUri}">
</head>
<body>
  <script src="${rendererUri}"></script>
  <script>
    var json = ${jsonStr};
    document.body.innerHTML = renderDFM(json);
  </script>
</body>
</html>`;
}

export function deactivate() {}
