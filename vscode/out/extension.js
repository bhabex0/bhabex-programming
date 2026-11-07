"use strict";
// Bhabex Language Support Extension for VS Code
// Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const child_process_1 = require("child_process");
function activate(context) {
    console.log('Bhabex Language Support activated');
    // Register commands
    const commands = [
        vscode.commands.registerCommand('bhabex.build', buildProject),
        vscode.commands.registerCommand('bhabex.run', runFile),
        vscode.commands.registerCommand('bhabex.check', checkSyntax),
        vscode.commands.registerCommand('bhabex.fmt', formatDocument),
        vscode.commands.registerCommand('bhabex.test', runTests),
        vscode.commands.registerCommand('bhabex.newProject', newProject),
        vscode.commands.registerCommand('bhabex.addDependency', addDependency),
        vscode.commands.registerCommand('bhabex.showOutput', showOutput),
    ];
    commands.forEach(cmd => context.subscriptions.push(cmd));
    // Register formatter
    context.subscriptions.push(vscode.languages.registerDocumentFormattingEditProvider('bhabex', {
        provideDocumentFormattingEdits(document) {
            return formatDocument(document);
        }
    }));
    // Register hover provider
    context.subscriptions.push(vscode.languages.registerHoverProvider('bhabex', {
        provideHover(document, position, token) {
            return provideHover(document, position, token);
        }
    }));
    // Register completion provider
    context.subscriptions.push(vscode.languages.registerCompletionItemProvider('bhabex', {
        provideCompletionItems(document, position, token, context) {
            return provideCompletions(document, position, token, context);
        }
    }, '.', ':', '(', '<'));
    // Register definition provider
    context.subscriptions.push(vscode.languages.registerDefinitionProvider('bhabex', {
        provideDefinition(document, position, token) {
            return provideDefinition(document, position, token);
        }
    }));
    // Register diagnostic collection
    const diagnosticCollection = vscode.languages.createDiagnosticCollection('bhabex');
    context.subscriptions.push(diagnosticCollection);
    // Lint on save / change
    if (vscode.workspace.getConfiguration('bhabex').get('lintOnSave')) {
        context.subscriptions.push(vscode.workspace.onDidSaveTextDocument(document => {
            if (document.languageId === 'bhabex') {
                lintDocument(document, diagnosticCollection);
            }
        }));
    }
    // Format on save
    if (vscode.workspace.getConfiguration('bhabex').get('formatOnSave')) {
        context.subscriptions.push(vscode.workspace.onDidSaveTextDocument(document => {
            if (document.languageId === 'bhabex') {
                formatDocument(document);
            }
        }));
    }
    // Status bar item
    const statusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBar.text = "$(gear) Bhabex";
    statusBar.tooltip = "Bhabex Language Support";
    statusBar.command = 'bhabex.showOutput';
    statusBar.show();
    context.subscriptions.push(statusBar);
    // Output channel
    const outputChannel = vscode.window.createOutputChannel('Bhabex');
    context.subscriptions.push(outputChannel);
    // Detect project
    detectProject(diagnosticCollection, outputChannel);
}
async function buildProject() {
    const workspace = vscode.workspace.workspaceFolders?.[0];
    if (!workspace) {
        vscode.window.showErrorMessage('No workspace folder open');
        return;
    }
    const terminal = vscode.window.createTerminal('Bhabex Build');
    terminal.sendText('bx build');
    terminal.show();
}
async function runFile(uri) {
    const editor = vscode.window.activeTextEditor;
    const file = uri || editor?.document.uri;
    if (!file) {
        vscode.window.showErrorMessage('No file to run');
        return;
    }
    const terminal = vscode.window.createTerminal('Bhabex Run');
    terminal.sendText(`bx run ${file.fsPath}`);
    terminal.show();
}
async function checkSyntax(uri) {
    const editor = vscode.window.activeTextEditor;
    const file = uri || editor?.document.uri;
    if (!file) {
        vscode.window.showErrorMessage('No file to check');
        return;
    }
    const result = await runCommand('bx', ['check', file.fsPath]);
    if (result.code !== 0) {
        vscode.window.showErrorMessage(`Syntax check failed: ${result.stderr}`);
    }
    else {
        vscode.window.showInformationMessage('Syntax OK');
    }
}
async function formatDocument(document) {
    const text = document.getText();
    const result = await runCommand('bx', ['fmt', '--stdin'], text);
    if (result.code === 0) {
        const fullRange = new vscode.Range(document.positionAt(0), document.positionAt(text.length));
        return [vscode.TextEdit.replace(fullRange, result.stdout)];
    }
    return [];
}
async function runTests() {
    const workspace = vscode.workspace.workspaceFolders?.[0];
    if (!workspace) {
        vscode.window.showErrorMessage('No workspace folder open');
        return;
    }
    const terminal = vscode.window.createTerminal('Bhabex Test');
    terminal.sendText('bx test');
    terminal.show();
}
async function newProject() {
    const name = await vscode.window.showInputBox({
        prompt: 'Project name',
        placeHolder: 'my-project'
    });
    if (!name)
        return;
    const isLib = await vscode.window.showQuickPick(['Binary', 'Library'], {
        placeHolder: 'Project type'
    }) === 'Library';
    const uri = await vscode.window.showOpenDialog({
        canSelectFolders: true,
        canSelectFiles: false,
        canSelectMany: false,
        openLabel: 'Create project here'
    });
    if (!uri || uri.length === 0)
        return;
    const projectPath = path.join(uri[0].fsPath, name);
    try {
        await runCommand('bxpkg', ['new', name, isLib ? '--lib' : ''], undefined, projectPath);
        vscode.window.showInformationMessage(`Created project ${name}`);
        // Open project
        const folder = vscode.Uri.file(projectPath);
        await vscode.commands.executeCommand('vscode.openFolder', folder);
    }
    catch (error) {
        vscode.window.showErrorMessage(`Failed to create project: ${error}`);
    }
}
async function addDependency() {
    const spec = await vscode.window.showInputBox({
        prompt: 'Package to add (name@version)',
        placeHolder: 'http@0.5'
    });
    if (!spec)
        return;
    const result = await runCommand('bxpkg', ['add', spec]);
    if (result.code === 0) {
        vscode.window.showInformationMessage(`Added ${spec}`);
    }
    else {
        vscode.window.showErrorMessage(`Failed to add: ${result.stderr}`);
    }
}
async function showOutput() {
    const channel = vscode.window.createOutputChannel('Bhabex');
    channel.show();
}
async function provideHover(document, position, token) {
    const wordRange = document.getWordRangeAtPosition(position);
    if (!wordRange)
        return undefined;
    const word = document.getText(wordRange);
    const docs = getDocumentation(word);
    if (!docs)
        return undefined;
    return new vscode.Hover(new vscode.MarkdownString(docs), wordRange);
}
function getDocumentation(word) {
    const docs = {
        'print': '**print(...values)**\n\nPrints values to stdout.',
        'println': '**println(...values)**\n\nPrints values to stdout with newline.',
        'fn': '**fn name(params) -> Type**\n\nFunction declaration.',
        'let': '**let mut name: Type = value**\n\nVariable declaration.',
        'mut': '**mut**\n\nMutable variable.',
        'const': '**const NAME: Type = value**\n\nConstant declaration.',
        'struct': '**struct Name { field: Type }**\n\nStruct definition.',
        'enum': '**enum Name { Variant(Type), Other }**\n\nEnum definition.',
        'trait': '**trait Name { fn method() -> Type; }**\n\nTrait definition.',
        'impl': '**impl Trait for Type { ... }**\n\nImplementation block.',
        'match': '**match expr { pattern => result, _ => default }**\n\nPattern matching.',
        'if': '**if condition { } else { }**\n\nConditional statement.',
        'for': '**for item in iterable { }**\n\nLoop over iterable.',
        'while': '**while condition { }**\n\nLoop while condition is true.',
        'async': '**async fn name() -> Type**\n\nAsync function.',
        'await': '**await future**\n\nAwait async operation.',
        'spawn': '**spawn || { }**\n\nSpawn async task.',
        'try': '**try { } catch Error as e { }**\n\nError handling.',
        'Channel': '**Channel::new() -> (Sender, Receiver)**\n\nMPSC channel.',
        'Mutex': '**Mutex::new(value) -> Mutex<T>**\n\nMutual exclusion lock.',
        'RwLock': '**RwLock::new(value) -> RwLock<T>**\n\nRead-write lock.',
        'Result': '**Result<T, E> = Ok(T) | Err(E)**\n\nError handling type.',
        'Option': '**Option<T> = Some(T) | None**\n\nOptional value type.',
    };
    return docs[word];
}
async function provideCompletions(document, position, token, context) {
    const completions = [];
    // Keywords
    const keywords = [
        'fn', 'let', 'mut', 'const', 'struct', 'enum', 'trait', 'impl',
        'type', 'mod', 'import', 'export', 'use', 'as', 'if', 'else',
        'match', 'for', 'in', 'while', 'loop', 'break', 'continue',
        'return', 'async', 'await', 'spawn', 'try', 'catch', 'throw',
        'pub', 'unsafe', 'extern', 'where', 'Self', 'self', 'super', 'crate'
    ];
    keywords.forEach(kw => {
        const item = new vscode.CompletionItem(kw, vscode.CompletionItemKind.Keyword);
        item.detail = 'Keyword';
        completions.push(item);
    });
    // Types
    const types = [
        'i8', 'i16', 'i32', 'i64', 'i128', 'isize',
        'u8', 'u16', 'u32', 'u64', 'u128', 'usize',
        'f32', 'f64', 'bool', 'char', 'str', 'string', 'never', '()'
    ];
    types.forEach(t => {
        const item = new vscode.CompletionItem(t, vscode.CompletionItemKind.Class);
        item.detail = 'Primitive type';
        completions.push(item);
    });
    // Built-ins
    const builtins = [
        'print', 'println', 'len', 'String', 'Vec', 'HashMap', 'HashSet',
        'Option', 'Result', 'Box', 'Rc', 'Arc', 'Ref', 'RefMut',
        'Channel', 'Mutex', 'RwLock', 'Semaphore', 'Condvar'
    ];
    builtins.forEach(b => {
        const item = new vscode.CompletionItem(b, vscode.CompletionItemKind.Function);
        item.detail = 'Built-in';
        completions.push(item);
    });
    return completions;
}
async function provideDefinition(document, position, token) {
    // Would implement go-to-definition using symbol table
    return undefined;
}
async function lintDocument(document, collection) {
    const result = await runCommand('bx', ['check', document.uri.fsPath]);
    if (result.code !== 0) {
        const diagnostics = [];
        const lines = result.stderr.split('\n');
        // Parse error output (simplified)
        // Real implementation would parse structured error output
        collection.set(document.uri, diagnostics);
    }
    else {
        collection.delete(document.uri);
    }
}
async function detectProject(diagnosticCollection, outputChannel) {
    const workspace = vscode.workspace.workspaceFolders?.[0];
    if (!workspace)
        return;
    const pkgPath = path.join(workspace.uri.fsPath, 'bxpkg.toml');
    if (fs.existsSync(pkgPath)) {
        outputChannel.appendLine('Bhabex project detected');
        // Run initial lint
        const files = await vscode.workspace.findFiles('**/*.bx');
        for (const file of files) {
            const doc = await vscode.workspace.openTextDocument(file);
            await lintDocument(doc, diagnosticCollection);
        }
    }
}
function runCommand(command, args, input, cwd) {
    return new Promise((resolve) => {
        const proc = (0, child_process_1.spawn)(command, args, { cwd, shell: true });
        let stdout = '';
        let stderr = '';
        proc.stdout.on('data', data => stdout += data.toString());
        proc.stderr.on('data', data => stderr += data.toString());
        if (input) {
            proc.stdin.write(input);
            proc.stdin.end();
        }
        proc.on('close', (code) => {
            resolve({ code: code || 0, stdout, stderr });
        });
        proc.on('error', (err) => {
            resolve({ code: -1, stdout, stderr: err.message });
        });
    });
}
function deactivate() {
    console.log('Bhabex Language Support deactivated');
}
//# sourceMappingURL=extension.js.map