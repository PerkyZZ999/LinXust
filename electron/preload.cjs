const { contextBridge, ipcRenderer } = require('electron')

contextBridge.exposeInMainWorld('linxustApi', {
  helloFromRust: (name) => ipcRenderer.invoke('linxust:hello', name),
})
