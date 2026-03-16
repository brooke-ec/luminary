import { isAuthenticated } from "$lib/api";
import type { PageLoad } from "./$types";
import { goto } from "$app/navigation";

export const load: PageLoad = async () => {
	if (isAuthenticated()) goto("/");
};
