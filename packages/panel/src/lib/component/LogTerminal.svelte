<script lang="ts">
	import { parseServerSentEvents, type ServerSentEvent } from "parse-sse";
	import { WebLinksAddon } from "@xterm/addon-web-links";
	import type { Attachment } from "svelte/attachments";
	import { Terminal, type ITheme } from "@xterm/xterm";
	import { SvelteMap } from "svelte/reactivity";
	import { FitAddon } from "@xterm/addon-fit";
	import "@xterm/xterm/css/xterm.css";
	import { client } from "$lib/api";
	import { onMount } from "svelte";
	import { Backoff } from "$lib";

	let { project }: { project: string } = $props();
	let loading = $state(true);

	let terminals = $state(new SvelteMap<string, Terminal>());

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

	const terminal: (terminal: Terminal) => Attachment<HTMLElement> = (terminal) => (el) => {
		const fitAddon = new FitAddon();

		terminal.loadAddon(new WebLinksAddon());
		terminal.loadAddon(fitAddon);
		terminal.open(el);

		// Remove the textarea from tab order
		const textarea = el.querySelector("textarea");
		if (textarea) textarea.tabIndex = -1;

		fitAddon.fit();
		const observer = new ResizeObserver(() => fitAddon.fit());
		observer.observe(el);

		return () => {
			observer.disconnect();
		};
	};

	async function stream(signal: AbortSignal) {
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

					if (event.type === "close") {
						const terminal = terminals.get(event.data);
						if (terminal) terminal.dispose();
						terminals.delete(event.data);
					} else {
						const id = event.type;
						if (!terminals.has(id)) terminals.set(id, new Terminal({ theme, disableStdin: true }));
						const terminal = terminals.get(event.type)!;

						terminal.write(Uint8Array.from(atob(event.data), (c) => c.charCodeAt(0)));
					}
				}
			} catch (e) {
				if (signal.aborted) return;
				await backoff.wait();
			}
		}
	}

	onMount(() => {
		const abort = new AbortController();
		stream(abort.signal);

		return () => {
			terminals.forEach((t) => t.dispose());
			abort.abort();
		};
	});
</script>

<div class="container">
	{#if loading}
		<div class="positioner"><span class="loader"></span></div>
	{/if}

	<div class="terminals">
		{#each terminals.entries() as [id, t] (id)}
			<div {@attach terminal(t)} data-uuid={id}></div>
		{/each}
	</div>
</div>

<style lang="scss">
	.container {
		position: relative;

		background-color: var(--crust);
		border-radius: 10px;
		padding: 10px;
	}

	.terminals {
		display: grid;
		grid-auto-flow: row;
		grid-auto-rows: minmax(0, 1fr);

		max-height: 452px;
		min-height: 452px;
	}

	.terminals > div {
		min-width: 0;
		width: 100%;
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
