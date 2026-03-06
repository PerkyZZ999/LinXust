const { app, BrowserWindow, ipcMain } = require("electron");
const path = require("path");

const { loadNativeBridge } = require("./native-bridge.cjs");

const frontendEntry = path.join(
	__dirname,
	"..",
	"frontend",
	"dist",
	"index.html",
);

const { binding: native, bindingInfo } = loadNativeBridge();

function createWindow() {
	const win = new BrowserWindow({
		width: 1100,
		height: 720,
		webPreferences: {
			preload: path.join(__dirname, "preload.cjs"),
			contextIsolation: true,
			nodeIntegration: false,
		},
	});

	const devUrl = process.env.LINXUST_DEV_URL;
	if (devUrl) {
		win.loadURL(devUrl);
		return;
	}

	win.loadFile(frontendEntry);
}

ipcMain.handle("linxust:hello", (_event, name) => native.helloFromRust(name));
ipcMain.handle("linxust:bridge-status", () => bindingInfo);

app.whenReady().then(() => {
	createWindow();
	app.on("activate", () => {
		if (BrowserWindow.getAllWindows().length === 0) createWindow();
	});
});

app.on("window-all-closed", () => {
	if (process.platform !== "darwin") app.quit();
});
