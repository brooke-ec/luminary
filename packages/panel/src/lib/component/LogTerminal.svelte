<script lang="ts">
	import { parseServerSentEvents, type ServerSentEvent } from "parse-sse";
	import { WebLinksAddon } from "@xterm/addon-web-links";
	import type { Attachment } from "svelte/attachments";
	import { FitAddon } from "@xterm/addon-fit";
	import { Terminal, type ITheme } from "@xterm/xterm";
	import "@xterm/xterm/css/xterm.css";
	import { client } from "$lib/api";
	import { Backoff } from "$lib";

	let { project }: { project: string } = $props();
	let loading = $state(true);

	function getComputedCSSVar(name: string) {
		return getComputedStyle(document.documentElement).getPropertyValue(`--${name}`).trim();
	}

	const theme = Object.fromEntries(
		Object.entries({
			cursor: "rosewater",
			selectionForeground: "base",
			selectionBackground: "lavender",
			background: "crust",
			foreground: "text",
			black: "crust",
			red: "red",
			green: "green",
			yellow: "yellow",
			blue: "blue",
			magenta: "pink",
			cyan: "teal",
			white: "text",
		} satisfies ITheme).map(([key, varName]) => [key, getComputedCSSVar(varName)]),
	);

	const terminal: Attachment<HTMLElement> = (el) => {
		const terminal = new Terminal({ fontFamily: "DejaVu Mono", theme });
		const fitAddon = new FitAddon();

		terminal.loadAddon(new WebLinksAddon());
		terminal.loadAddon(fitAddon);
		terminal.open(el);

		fitAddon.fit();
		const observer = new ResizeObserver(() => fitAddon.fit());
		observer.observe(el);

		const abort = new AbortController();
		stream(terminal, abort.signal);

		return () => {
			abort.abort();
			observer.disconnect();
			terminal.dispose();
		};
	};

	async function stream(terminal: Terminal, signal: AbortSignal) {
		const backoff = new Backoff();

		while (true) {
			try {
				loading = true;

				const { response } = await client.GET("/api/project/{project}/logs", {
					params: { path: { project } },
					parseAs: "stream",
					signal,
				});

				backoff.reset();

				for await (const event of parseServerSentEvents(
					response,
				) as unknown as AsyncIterable<ServerSentEvent>) {
					if (signal.aborted) return;
					loading = false;

					terminal.write(Uint8Array.from(atob(event.data), (c) => c.charCodeAt(0)));
				}
			} catch (_) {
				if (signal.aborted) return;
				await backoff.wait();
			}
		}
	}
</script>

<div class="container">
	{#if loading}
		<div class="positioner"><span class="loader"></span></div>
	{/if}

	<div {@attach terminal}></div>
</div>

<style lang="scss">
	.container {
		position: relative;

		background-color: var(--crust);
		border-radius: 10px;
		padding: 10px;
	}

	.positioner {
		position: absolute;
		inset: 0;

		display: flex;
		align-items: center;
		justify-content: center;
	}

	.loader {
		z-index: 10;

		width: 48px;
		height: 48px;
		border: 5px solid var(--text);
		border-bottom-color: transparent;
		border-radius: 50%;
		display: inline-block;
		box-sizing: border-box;
		animation: rotation 1s linear infinite;
	}

	@keyframes rotation {
		0% {
			transform: rotate(0deg);
		}
		100% {
			transform: rotate(360deg);
		}
	}
</style>
