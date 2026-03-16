import { parseServerSentEvents, type ServerSentEvent } from "parse-sse";
import type { components } from "./openapi";
import { error, sleep, warn } from "$lib";
import { patch } from "ultrapatch";
import { client } from ".";

export type LuminaryStateList = components["schemas"]["luminary_node.core.model.LuminaryStateList"];
type LogMessage = components["schemas"]["luminary_node.logging.LogMessage"];

const INITIAL_RETRY_DELAY = 1000;

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
export async function subscribe(fetch?: typeof globalThis.fetch) {
	let retryDelay = INITIAL_RETRY_DELAY;

	while (true) {
		try {
			const { response } = await client.GET("/api/realtime", {
				parseAs: "stream",
				fetch,
			});

			retryDelay = INITIAL_RETRY_DELAY;
			list = {};

			try {
				for await (const event of parseServerSentEvents(response) as unknown as AsyncIterable<ServerSentEvent>)
					handleEvent(event);
			} catch (err) {
				error("Connection to server lost", [
					"Your connection to the server was lost: " + (err instanceof Error ? err.message : String(err)),
					"Luminary will attempt to reconnect automatically. If the issue persists, please check your network connection and the server status.",
				]);
			}
		} catch (_) {
			await sleep(retryDelay);
			retryDelay = Math.min(retryDelay * 2, 30000); // Exponential backoff with a max of 30 seconds
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
