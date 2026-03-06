import {
	QueryCache,
	QueryClient,
	QueryClientProvider,
} from "@tanstack/react-query";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import App from "./App.tsx";
import "./index.css";
import { initRendererObservability, logQueryError } from "./lib/observability";

initRendererObservability();

const queryClient = new QueryClient({
	queryCache: new QueryCache({
		onError: (error, query) => {
			logQueryError(error, query.queryKey);
		},
	}),
});

const rootElement = document.getElementById("root");

if (!rootElement) {
	throw new Error("Renderer root element was not found.");
}

createRoot(rootElement).render(
	<StrictMode>
		<QueryClientProvider client={queryClient}>
			<App />
		</QueryClientProvider>
	</StrictMode>,
);
