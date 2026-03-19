import { error } from "@sveltejs/kit";
import type { PageLoad } from "./$types";
import { api } from "$lib";

export const load: PageLoad = async ({ url }) => {
	const token = url.searchParams.get("token");
	if (!token) error(400, "Token is required");

	const response = await api.client
		.GET("/api/auth/reset/{token}", { params: { path: { token } } })
		.catch(() => error(401, "Invalid token"));

	return {
		username: response.data!,
		token,
	};
};
