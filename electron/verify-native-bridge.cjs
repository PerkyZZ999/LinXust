const assert = require("node:assert/strict");

const { loadNativeBridge } = require("./native-bridge.cjs");

function main() {
	const { binding, bindingInfo } = loadNativeBridge();

	assert.equal(
		bindingInfo.mode,
		"native",
		`expected compiled addon, received ${bindingInfo.mode}: ${bindingInfo.detail}`,
	);

	const greeting = binding.helloFromRust("LinXust");
	assert.equal(greeting, "Hello, LinXust, from Rust!");

	console.log("Native bridge verified:", greeting);
}

main();
