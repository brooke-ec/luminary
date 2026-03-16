import { api } from "$lib";
import type { LayoutLoad } from "./$types";

export const load: LayoutLoad = async ({ fetch }) => {
	api.subscribe(fetch);
};
