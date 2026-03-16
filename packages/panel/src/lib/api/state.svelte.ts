import { parseServerSentEvents, type ServerSentEvent } from "parse-sse";
import type { components } from "./openapi";
import { error, warn } from "$lib";
import { patch } from "ultrapatch";
import { client } from ".";

export type LuminaryStateList = components["schemas"]["luminary_node.core.model.LuminaryStateList"];
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
export async function subscribe(fetch?: typeof globalThis.fetch) {
	list = {};

	const { response } = await client.GET("/api/realtime", {
		parseAs: "stream",
		fetch,
	});

	for await (const event of parseServerSentEvents(response) as unknown as AsyncIterable<ServerSentEvent>) {
		switch (event.type) {
			case "list":
				patch(list, JSON.parse(event.data));
			case "log":
				const message = JSON.parse(event.data) as LogMessage;
				const title = message.message?.split("\n")[0] ?? "Log Message";
				const details = message.message ?? "No additional information provided.";
				switch (message.level) {
					case "ERROR":
						error(title, details);
					case "WARN":
						warn(title, details);
				}
		}
	}
}
