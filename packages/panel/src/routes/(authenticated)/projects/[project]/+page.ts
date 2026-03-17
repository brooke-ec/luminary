import type { PageLoad } from "./$types";
import { error } from "@sveltejs/kit";
import { api } from "$lib";

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
