import {
	createLogger,
	initLogger,
	log,
	parseError,
	type ParsedError,
	type RequestLogger,
} from "evlog";
import { createBrowserLogDrain } from "evlog/browser";

export interface RendererLogFields {
	feature: "app-shell" | "native-bridge" | "react-query";
	action: string;
	runtime?: {
		surface?: "renderer";
		dev?: boolean;
		transport?: "console" | "browser-drain";
	};
	bridge?: {
		mode?: BridgeStatus["mode"];
		detail?: string;
	};
	query?: {
		key?: string;
	};
	outcome?: {
		status?: "success" | "fallback" | "error";
		reason?: string;
	};
	error?: {
		message?: string;
		why?: string;
		fix?: string;
		link?: string;
	};
}

let initialized = false;

export function initRendererObservability(): void {
	if (initialized) {
		return;
	}

	const endpoint = import.meta.env.VITE_EVLOG_ENDPOINT?.trim();
	const drain = endpoint
		? createBrowserLogDrain({
				drain: { endpoint },
				pipeline: { batch: { size: 10, intervalMs: 2_000 } },
			})
		: undefined;

	initLogger({
		env: {
			service: "linxust-renderer",
			environment: import.meta.env.DEV ? "development" : "production",
		},
		pretty: import.meta.env.DEV,
		...(drain ? { drain } : {}),
	});

	initialized = true;

	log.info({
		feature: "app-shell",
		action: "renderer_initialized",
		runtime: {
			surface: "renderer",
			dev: import.meta.env.DEV,
			transport: endpoint ? "browser-drain" : "console",
		},
	});
}

export function createRendererLogger(
	initialContext: RendererLogFields,
): RequestLogger<RendererLogFields> {
	return createLogger<RendererLogFields>(
		initialContext as unknown as Record<string, unknown>,
	);
}

export function toParsedError(error: unknown): ParsedError {
	return parseError(error);
}

export function logQueryError(
	error: unknown,
	queryKey: readonly unknown[],
): void {
	const parsed = toParsedError(error);

	log.error({
		feature: "react-query",
		action: "query_failed",
		query: {
			key: JSON.stringify(queryKey),
		},
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
}
