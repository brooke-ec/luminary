<script lang="ts">
	import { parseServerSentEvents, type ServerSentEvent } from "parse-sse";
	import { WebLinksAddon } from "@xterm/addon-web-links";
	import type { Attachment } from "svelte/attachments";
	import { FitAddon } from "@xterm/addon-fit";
	import { Terminal } from "@xterm/xterm";
	import "@xterm/xterm/css/xterm.css";
	import { client } from "$lib/api";

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

		stream(terminal);

		return () => {
			observer.disconnect();
			terminal.dispose();
		};
	};

	async function stream(terminal: Terminal) {
		const { response } = await client.GET("/api/project/{project}/logs", {
			params: { path: { project } },
			parseAs: "stream",
		});

		for await (const event of parseServerSentEvents(response) as unknown as AsyncIterable<ServerSentEvent>) {
			terminal.write(Uint8Array.from(atob(event.data), (c) => c.charCodeAt(0)));
		}
	}
</script>

<div {@attach terminal}></div>
