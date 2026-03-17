<script lang="ts">
	import { parseServerSentEvents, type ServerSentEvent } from "parse-sse";
	import { WebLinksAddon } from "@xterm/addon-web-links";
	import type { Attachment } from "svelte/attachments";
	import { FitAddon } from "@xterm/addon-fit";
	import { Terminal } from "@xterm/xterm";
	import "@xterm/xterm/css/xterm.css";
	import { client } from "$lib/api";
	import { Backoff } from "$lib";

	let { project }: { project: string } = $props();

	const terminal: Attachment<HTMLElement> = (el) => {
		const terminal = new Terminal();
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

					terminal.write(Uint8Array.from(atob(event.data), (c) => c.charCodeAt(0)));
				}
			} catch (_) {
				if (signal.aborted) return;
				await backoff.wait();
			}
		}
	}
</script>

<div {@attach terminal}></div>
