import { useQuery } from "@tanstack/react-query";
import { log } from "evlog";
import { useEffect } from "react";

import "./App.css";
import { readBridgeStatus, readGreeting } from "./lib/bridge";

function App() {
	useEffect(() => {
		log.info({
			feature: "app-shell",
			action: "app_viewed",
			runtime: {
				surface: "renderer",
				dev: import.meta.env.DEV,
			},
		});
	}, []);

	const bridgeStatus = useQuery({
		queryKey: ["bridge-status"],
		queryFn: readBridgeStatus,
	});

	const greeting = useQuery({
		queryKey: ["hello"],
		queryFn: readGreeting,
	});

	return (
		<main className="app-shell">
			<section className="hero card">
				<p className="eyebrow">Linux-first archive workstation</p>
				<h1>LinXust</h1>
				<p className="lead">
					Electron hosts the renderer, Rust owns archive processing, and the
					bridge below shows whether the compiled N-API addon is actually
					loaded.
				</p>
			</section>

			<section className="status-grid">
				<article className="card">
					<p className="section-label">Bridge mode</p>
					<p
						className={`status-chip status-chip-${bridgeStatus.data?.mode ?? "fallback"}`}
					>
						{bridgeStatus.isLoading
							? "Checking..."
							: (bridgeStatus.data?.mode ?? "fallback")}
					</p>
					<p className="detail-copy">
						{bridgeStatus.isLoading
							? "Inspecting preload/main status..."
							: bridgeStatus.data?.detail}
					</p>
				</article>

				<article className="card">
					<p className="section-label">Hello test</p>
					<p className="detail-copy">
						{greeting.isLoading ? "Calling bridge..." : greeting.data}
					</p>
				</article>
			</section>
		</main>
	);
}

export default App;
