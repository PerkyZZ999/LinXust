const path = require("path");

const nativeEntrypoint = path.join(__dirname, "..", "native", "index.js");

function createFallbackBinding(reason) {
	return {
		helloFromRust: (name) =>
			`Fallback path active for ${name}. Build the native addon to execute Rust code.`,
		__bindingInfo: {
			mode: "fallback",
			detail: reason,
		},
	};
}

function loadNativeBridge() {
	try {
		const binding = require(nativeEntrypoint);
		if (typeof binding.helloFromRust !== "function") {
			throw new TypeError("native addon did not export helloFromRust");
		}

		return {
			binding,
			bindingInfo: {
				mode: "native",
				detail: "compiled addon loaded through native/index.js",
			},
		};
	} catch (error) {
		const reason =
			error instanceof Error ? error.message : "native module not built yet";
		const binding = createFallbackBinding(reason);

		return {
			binding,
			bindingInfo: binding.__bindingInfo,
		};
	}
}

module.exports = {
	loadNativeBridge,
};
