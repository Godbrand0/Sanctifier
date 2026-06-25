import * as vscode from 'vscode';
import { analyzeSorobanSource, looksLikeSorobanSource, type EditorFinding } from './analyzer';
import { findingsToSarif, parseSarif, serialiseSarif, validateSarifShape, sarifToFindings } from './sarif';
import { validateSarifContent, validateSarifResultCount, isPathWithinWorkspace, MAX_SARIF_BYTES } from './security';
import { spawn } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';

const SOURCE = 'sanctifier';

let sorobanWorkspaceCache: boolean | null = null;

function getConfig() {
  return vscode.workspace.getConfiguration('sanctifier');
}

async function workspaceLooksLikeSorobanProject(): Promise<boolean> {
  if (sorobanWorkspaceCache !== null) {
    return sorobanWorkspaceCache;
  }
  const files = await vscode.workspace.findFiles('**/Cargo.toml', '**/target/**', 40);
  for (const uri of files) {
    try {
      const doc = await vscode.workspace.openTextDocument(uri);
      const t = doc.getText();
      if (/soroban-sdk|soroban_sdk/.test(t)) {
        sorobanWorkspaceCache = true;
        return true;
      }
    } catch {
      /* skip */
    }
  }
  sorobanWorkspaceCache = false;
  return false;
}

function findingToDiagnostic(doc: vscode.TextDocument, f: EditorFinding): vscode.Diagnostic {
  const lineIdx = Math.max(0, Math.min(doc.lineCount - 1, f.line - 1));
  const line = doc.lineAt(lineIdx);
  const range =
    f.endLine !== undefined
      ? new vscode.Range(
          lineIdx,
          0,
          Math.max(lineIdx, f.endLine - 1),
          f.endCharacter ?? Number.MAX_SAFE_INTEGER
        )
      : new vscode.Range(lineIdx, 0, lineIdx, line.range.end.character || line.text.length);

  const sev =
    f.severity === 'error'
      ? vscode.DiagnosticSeverity.Error
      : f.severity === 'information'
        ? vscode.DiagnosticSeverity.Information
        : vscode.DiagnosticSeverity.Warning;

  const d = new vscode.Diagnostic(range, f.message, sev);
  d.code = f.code;
  d.source = SOURCE;
  return d;
}

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const collection = vscode.languages.createDiagnosticCollection(SOURCE);

  const debouncers = new Map<string, ReturnType<typeof setTimeout>>();

  const runAnalysis = (doc: vscode.TextDocument) => {
    if (doc.languageId !== 'rust') {
      return;
    }
    const cfg = getConfig();
    if (!cfg.get<boolean>('enable')) {
      collection.delete(doc.uri);
      return;
    }
    const text = doc.getText();
    if (!looksLikeSorobanSource(text)) {
      collection.delete(doc.uri);
      return;
    }

    const findings = analyzeSorobanSource(text);
    const diags = findings.map((f) => findingToDiagnostic(doc, f));
    collection.set(doc.uri, diags);
  };

  const schedule = (doc: vscode.TextDocument) => {
    const onlySorobanWs = getConfig().get<boolean>('onlyInSorobanWorkspace');
    if (onlySorobanWs && !vscode.workspace.workspaceFolders?.length) {
      collection.delete(doc.uri);
      return;
    }
    const ms = getConfig().get<number>('debounceMs') ?? 400;
    const key = doc.uri.toString();
    const prev = debouncers.get(key);
    if (prev) {
      clearTimeout(prev);
    }
    debouncers.set(
      key,
      setTimeout(async () => {
        debouncers.delete(key);
        const requireSoroban = getConfig().get<boolean>('onlyInSorobanWorkspace');
        if (requireSoroban) {
          const ok = await workspaceLooksLikeSorobanProject();
          if (!ok) {
            collection.delete(doc.uri);
            return;
          }
        }
        runAnalysis(doc);
      }, ms)
    );
  };

  context.subscriptions.push(
    collection,
    vscode.workspace.onDidChangeTextDocument((e) => schedule(e.document)),
    vscode.workspace.onDidOpenTextDocument((d) => schedule(d)),
    vscode.workspace.onDidCloseTextDocument((d) => {
      collection.delete(d.uri);
      debouncers.delete(d.uri.toString());
    }),
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration('sanctifier')) {
        sorobanWorkspaceCache = null;
        for (const doc of vscode.workspace.textDocuments) {
          schedule(doc);
        }
      }
    })
  );

  for (const doc of vscode.workspace.textDocuments) {
    schedule(doc);
  }

  context.subscriptions.push(
    vscode.commands.registerCommand('sanctifier.analyzeWorkspace', async () => {
      const exe = getConfig().get<string>('sanctifierPath')?.trim();
      if (!exe) {
        vscode.window.showWarningMessage(
          'Set sanctifier.sanctifierPath to your sanctifier CLI binary, then run again.'
        );
        return;
      }
      const folder = vscode.workspace.workspaceFolders?.[0];
      if (!folder) {
        vscode.window.showErrorMessage('Open a folder to analyze.');
        return;
      }
      const token = await new Promise<string | undefined>((resolve) => {
        const p = spawn(exe, ['analyze', folder.uri.fsPath, '--format', 'json'], {
          cwd: folder.uri.fsPath,
        });
        let out = '';
        let err = '';
        p.stdout.on('data', (b) => (out += b.toString()));
        p.stderr.on('data', (b) => (err += b.toString()));
        p.on('close', () => resolve(out || undefined));
        p.on('error', () => resolve(undefined));
      });
      if (!token) {
        vscode.window.showErrorMessage('sanctifier CLI failed or produced no output. Check sanctifierPath.');
        return;
      }
      const doc = await vscode.workspace.openTextDocument({
        content: token,
        language: 'json',
      });
      await vscode.window.showTextDocument(doc, { preview: true });
    }),

    vscode.commands.registerCommand('sanctifier.exportSarif', async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.document.languageId !== 'rust') {
        vscode.window.showWarningMessage('Open a Rust file to export its findings as SARIF.');
        return;
      }
      const text = editor.document.getText();
      if (Buffer.byteLength(text, 'utf8') > MAX_SARIF_BYTES) {
        vscode.window.showErrorMessage('File is too large to analyse for SARIF export.');
        return;
      }
      const findings: EditorFinding[] = analyzeSorobanSource(text);
      const fileUri = editor.document.uri.fsPath;
      const sarifLog = findingsToSarif(findings, vscode.Uri.file(fileUri).toString());
      const content = serialiseSarif(sarifLog);

      const defaultUri = vscode.Uri.file(path.join(path.dirname(fileUri), 'sanctifier.sarif'));
      const dest = await vscode.window.showSaveDialog({
        defaultUri,
        filters: { 'SARIF files': ['sarif', 'json'], 'All files': ['*'] },
        title: 'Export Sanctifier findings as SARIF',
      });
      if (!dest) return;

      const folder = vscode.workspace.workspaceFolders?.[0];
      if (folder && !isPathWithinWorkspace(folder.uri.fsPath, dest.fsPath)) {
        vscode.window.showErrorMessage('Cannot export outside the current workspace.');
        return;
      }

      fs.writeFileSync(dest.fsPath, content, 'utf8');
      const open = await vscode.window.showInformationMessage(
        `SARIF exported: ${path.basename(dest.fsPath)}`,
        'Open'
      );
      if (open === 'Open') {
        const doc = await vscode.workspace.openTextDocument(dest);
        await vscode.window.showTextDocument(doc, { preview: true });
      }
    }),

    vscode.commands.registerCommand('sanctifier.importSarif', async () => {
      const uris = await vscode.window.showOpenDialog({
        canSelectFiles: true,
        canSelectFolders: false,
        canSelectMany: false,
        filters: { 'SARIF files': ['sarif', 'json'], 'All files': ['*'] },
        title: 'Import SARIF file',
      });
      if (!uris || uris.length === 0) return;
      const sarifPath = uris[0].fsPath;

      const folder = vscode.workspace.workspaceFolders?.[0];
      if (folder && !isPathWithinWorkspace(folder.uri.fsPath, sarifPath)) {
        vscode.window.showErrorMessage('Selected file is outside the current workspace.');
        return;
      }

      const raw = fs.readFileSync(sarifPath, 'utf8');
      const sizeCheck = validateSarifContent(raw);
      if (!sizeCheck.ok) {
        vscode.window.showErrorMessage(`Cannot import SARIF: ${sizeCheck.error}`);
        return;
      }

      let parsed: unknown;
      try {
        parsed = parseSarif(raw);
      } catch {
        vscode.window.showErrorMessage('Failed to parse SARIF file: invalid JSON.');
        return;
      }

      if (!validateSarifShape(parsed)) {
        vscode.window.showErrorMessage('File does not look like a valid SARIF 2.1.0 log.');
        return;
      }

      const countCheck = validateSarifResultCount(parsed.runs);
      if (!countCheck.ok) {
        vscode.window.showErrorMessage(`Cannot import SARIF: ${countCheck.error}`);
        return;
      }

      const findings = sarifToFindings(parsed);
      vscode.window.showInformationMessage(
        `Imported ${findings.length} finding(s) from ${path.basename(sarifPath)}.`
      );
    })
  );
}

export function deactivate(): void {
  sorobanWorkspaceCache = null;
}
