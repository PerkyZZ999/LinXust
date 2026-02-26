const { app, BrowserWindow, ipcMain } = require('electron')
const path = require('path')

let native
try {
  native = require(path.join(__dirname, '..', 'native', 'index.js'))
} catch {
  native = { helloFromRust: (name) => `Hello, ${name}, from Rust (native module not built yet)!` }
}

function createWindow() {
  const win = new BrowserWindow({
    width: 1100,
    height: 720,
    webPreferences: {
      preload: path.join(__dirname, 'preload.cjs'),
      contextIsolation: true,
      nodeIntegration: false,
    },
  })

  const devUrl = process.env.LINXUST_DEV_URL || 'http://localhost:5173'
  win.loadURL(devUrl)
}

ipcMain.handle('linxust:hello', (_event, name) => native.helloFromRust(name))

app.whenReady().then(() => {
  createWindow()
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow()
  })
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit()
})
