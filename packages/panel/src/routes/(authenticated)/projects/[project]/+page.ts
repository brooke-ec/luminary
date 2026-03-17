import { api } from "$lib";
import { error } from "@sveltejs/kit";
import type { PageLoad } from "./$types";

export const prerender = false;

export const load: PageLoad = async ({ params }) => {
	const response = await api.client
		.GET("/api/project/{project}/compose", {
			params: { path: { project: params.project } },
		})
		.catch(() => error(404, "Project not found"));

	return {
		compose: response.data!,
	};
};
