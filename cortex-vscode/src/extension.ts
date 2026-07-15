import * as vscode from 'vscode';
import * as http from 'http';

const CORTEX_PORT = 8787;
let statusBarItem: vscode.StatusBarItem;

export function activate(context: vscode.ExtensionContext) {
    // Status bar item
    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBarItem.command = 'cortex.status';
    context.subscriptions.push(statusBarItem);
    
    updateStatus();
    const interval = setInterval(updateStatus, 10000);
    context.subscriptions.push({ dispose: () => clearInterval(interval) });
    
    // Command: Ask with Context
    const askCmd = vscode.commands.registerCommand('cortex.ask', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showWarningMessage('Open a file first');
            return;
        }
        
        const filePath = vscode.workspace.asRelativePath(editor.document.uri, false);
        
        const query = await vscode.window.showInputBox({
            prompt: 'Ask Cortex...',
            placeHolder: 'e.g., why did I use redis here?'
        });
        
        if (!query) return;
        
        vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: "Cortex is assembling context...",
            cancellable: false
        }, async () => {
            const contextText = await fetchContext(filePath, query);
            if (contextText) {
                const doc = await vscode.workspace.openTextDocument({
                    content: contextText,
                    language: 'markdown'
                });
                await vscode.window.showTextDocument(doc, {
                    viewColumn: vscode.ViewColumn.Beside,
                    preview: false
                });
            }
        });
    });
    context.subscriptions.push(askCmd);
    
    // Command: Status
    const statusCmd = vscode.commands.registerCommand('cortex.status', async () => {
        const stats = await fetchStats();
        if (stats) {
            const data = JSON.parse(stats);
            vscode.window.showInformationMessage(
                `Cortex: ${data.total_entities} entities, ${data.total_files} files`
            );
        } else {
            vscode.window.showWarningMessage('Cortex not running. Run: cortex serve');
        }
    });
    context.subscriptions.push(statusCmd);
}

async function updateStatus() {
    const healthy = await checkHealth();
    if (healthy) {
        statusBarItem.text = "$(brain) Cortex";
        statusBarItem.tooltip = "Cortex memory active — click for status";
        statusBarItem.color = undefined;
        statusBarItem.show();
    } else {
        statusBarItem.text = "$(circle-slash) Cortex";
        statusBarItem.tooltip = "Cortex not running. Run: cortex serve";
        statusBarItem.color = new vscode.ThemeColor('statusBarItem.warningForeground');
        statusBarItem.show();
    }
}

function checkHealth(): Promise<boolean> {
    return new Promise((resolve) => {
        const req = http.get(`http://localhost:${CORTEX_PORT}/health`, (res) => {
            resolve(res.statusCode === 200);
        });
        req.on('error', () => resolve(false));
        req.setTimeout(2000, () => { req.destroy(); resolve(false); });
    });
}

function fetchContext(filePath: string, query: string): Promise<string | null> {
    return new Promise((resolve) => {
        const encodedPath = encodeURIComponent(filePath);
        const encodedQuery = encodeURIComponent(query);
        const url = `http://localhost:${CORTEX_PORT}/context?file_path=${encodedPath}&query=${encodedQuery}`;
        
        const req = http.get(url, (res) => {
            let data = '';
            res.on('data', chunk => data += chunk);
            res.on('end', () => resolve(data));
        });
        
        req.on('error', () => { 
            vscode.window.showErrorMessage('Cortex not running. Start it with: cortex serve');
            resolve(null); 
        });
        
        req.setTimeout(10000, () => { 
            req.destroy(); 
            vscode.window.showErrorMessage('Cortex request timed out');
            resolve(null); 
        });
    });
}

function fetchStats(): Promise<string | null> {
    return new Promise((resolve) => {
        const req = http.get(`http://localhost:${CORTEX_PORT}/status`, (res) => {
            let data = '';
            res.on('data', chunk => data += chunk);
            res.on('end', () => resolve(data));
        });
        req.on('error', () => resolve(null));
        req.setTimeout(2000, () => { req.destroy(); resolve(null); });
    });
}

export function deactivate() {}
