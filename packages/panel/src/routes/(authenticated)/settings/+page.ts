import type { PageLoad } from "../$types";
import { api } from "$lib";

export const load: PageLoad = async () => {
	const response = await api.client.GET("/api/auth/self");

	return {
		user: response.data!,
	};
};
