import type { components } from "./openapi";

export type LuminaryStateList = components["schemas"]["luminary_node.core.model.LuminaryStateList"];
export type LuminaryStatus = components["schemas"]["luminary_node.core.model.LuminaryStatus"];
export type LuminaryAction = components["schemas"]["luminary_node.core.model.LuminaryAction"];

/**
 * The current internal list of projects and their states.
 */
let list: LuminaryStateList = $state({});

/**
 * A getter for the current project list.
 * @returns A reactive store containing the current state list.
 */
export const getList = () => list;
