import { parseServerSentEvents, type ServerSentEvent } from "parse-sse";
import type { components } from "./openapi";
import { Backoff, error, warn } from "$lib";
import { client, isAuthenticated } from ".";
import { goto } from "$app/navigation";
import { patch } from "ultrapatch";

export type LuminaryStateList = components["schemas"]["luminary_node.core.model.LuminaryStateList"];
export type LuminaryProject = components["schemas"]["luminary_node.core.model.LuminaryProject"];
type LogMessage = components["schemas"]["luminary_node.logging.LogMessage"];

/**
 * The current internal list of projects and their states.
 */
let list: LuminaryStateList = $state({});

/**
 * A getter for the current project list.
 * @returns A reactive store containing the current state list.
 */
export const getList = () => list;

/**
 * Subscribes to real-time updates from the server.
 */
export function subscribe(fetch?: typeof globalThis.fetch) {
	if (!isAuthenticated()) goto("/login");

	const controller = new AbortController();
	listen(controller.signal, fetch);
	return () => controller.abort();
}

/**
 * The main listning loop that connects to the server and processes incoming events.
 */
async function listen(signal: AbortSignal, fetch?: typeof globalThis.fetch) {
	const backoff = new Backoff();

	while (isAuthenticated()) {
		try {
			const { response } = await client.GET("/api/realtime", {
				parseAs: "stream",
				signal,
				fetch,
			});

			backoff.reset();
			list = {};

			try {
				for await (const event of parseServerSentEvents(
					response,
				) as unknown as AsyncIterable<ServerSentEvent>) {
					if (signal.aborted) return;
					handleEvent(event);
				}
			} catch (err) {
				error("Connection to server lost", [
					"Your connection to the server was lost: " + (err instanceof Error ? err.message : String(err)),
					"Luminary will attempt to reconnect automatically. If the issue persists, please check your network connection and the server status.",
				]);
			}
		} catch (_) {
			if (signal.aborted) return;
			await backoff.wait();
		}
	}
}

/**
 * Handle incoming server-sent events from the server.
 * @param event The event received from the server.
 */
function handleEvent(event: ServerSentEvent) {
	switch (event.type) {
		case "list":
			patch(list, JSON.parse(event.data));
		case "log":
			const message = JSON.parse(event.data) as LogMessage;
			const details = message.message?.split(",").map((l) => l.trim()) ?? undefined;
			const title = details?.[0] ?? `Unknown ${message.level}`;
			switch (message.level) {
				case "ERROR":
					error(title, details);
				case "WARN":
					warn(title, details);
			}
	}
}
