import { createError } from "evlog";

import { createRendererLogger, toParsedError } from "./observability";

function requireBridgeMethod<T extends (...args: never[]) => Promise<unknown>>(
	methodName: string,
	method: T | undefined,
): T {
	if (method) {
		return method;
	}

	throw createError({
		message: "Electron bridge is unavailable",
		why: `window.linxustApi.${methodName} is undefined`,
		fix: "Launch the frontend through Electron so preload.cjs can expose the bridge APIs.",
		link: "https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts",
	});
}

export async function readBridgeStatus(): Promise<BridgeStatus> {
	const logger = createRendererLogger({
		feature: "native-bridge",
		action: "read_bridge_status",
	});

	try {
		const getBridgeStatus = requireBridgeMethod(
			"getBridgeStatus",
			window.linxustApi?.getBridgeStatus,
		);
		const status = await getBridgeStatus();

		logger.set({
			bridge: {
				mode: status.mode,
				detail: status.detail,
			},
			outcome: {
				status: status.mode === "fallback" ? "fallback" : "success",
			},
		});

		if (status.mode === "fallback") {
			logger.warn("bridge status resolved through the fallback path");
			logger.emit({ _forceKeep: true });
		} else {
			logger.info("native addon is active");
			logger.emit();
		}

		return status;
	} catch (error) {
		const parsed = toParsedError(error);

		logger.error(parsed.message, {
			outcome: {
				status: "error",
				reason: parsed.message,
			},
			error: {
				message: parsed.message,
				why: parsed.why,
				fix: parsed.fix,
				link: parsed.link,
			},
		});
		logger.emit({ _forceKeep: true });

		throw error;
	}
}

export async function readGreeting(): Promise<string> {
	const logger = createRendererLogger({
		feature: "native-bridge",
		action: "hello_from_rust",
	});

	try {
		const helloFromRust = requireBridgeMethod(
			"helloFromRust",
			window.linxustApi?.helloFromRust,
		) as (name: string) => Promise<string>;
		const greeting = await helloFromRust("LinXust");

		logger.info("renderer greeting request completed", {
			outcome: {
				status: "success",
			},
		});
		logger.emit();

		return greeting;
	} catch (error) {
		const parsed = toParsedError(error);

		logger.error(parsed.message, {
			outcome: {
				status: "error",
				reason: parsed.message,
			},
			error: {
				message: parsed.message,
				why: parsed.why,
				fix: parsed.fix,
				link: parsed.link,
			},
		});
		logger.emit({ _forceKeep: true });

		throw error;
	}
}
