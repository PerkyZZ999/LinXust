/// <reference types="vite/client" />

declare global {
	interface ImportMetaEnv {
		readonly VITE_EVLOG_ENDPOINT?: string;
	}

	interface BridgeStatus {
		mode: "native" | "fallback";
		detail: string;
	}

	interface Window {
		linxustApi?: {
			getBridgeStatus: () => Promise<BridgeStatus>;
			helloFromRust: (name: string) => Promise<string>;
		};
	}
}

export {};
