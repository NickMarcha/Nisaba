# Desktop App Troubleshooting

## Window never opens

### 1. Port 1420 in use

If you see `Error: Port 1420 is already in use`, a previous Vite process is still running.

**Fix:** Kill the process, then run again:

```powershell
# Find process using port 1420
netstat -ano | findstr :1420

# Kill it (replace PID with the number from the output)
taskkill /PID <PID> /F
```

### 2. Try running from the desktop app directory

```bash
npm run dev:desktop:direct
```

This runs Tauri from `apps/desktop` directly instead of via the workspace.

### 3. WebView2 (Windows)

Tauri on Windows requires the WebView2 runtime. If the window never appears, install or repair it:

- Download: https://developer.microsoft.com/en-us/microsoft-edge/webview2/
- Or run: `winget install Microsoft.EdgeWebView2Runtime`

### 4. Run the built app manually

1. Start Vite in one terminal: `cd apps/desktop && npm run dev`
2. In another terminal, run the built exe: `apps\desktop\src-tauri\target\debug\nisaba-desktop.exe`

If the window opens, the issue is with the dev script flow. If it still doesn't open, the issue may be WebView2 or the app itself.
