const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("linxustApi", {
	getBridgeStatus: () => ipcRenderer.invoke("linxust:bridge-status"),
	helloFromRust: (name) => ipcRenderer.invoke("linxust:hello", name),
});
