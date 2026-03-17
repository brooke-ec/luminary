import type { components } from "$lib/api/openapi";
import type { LuminaryProject } from "$lib/api";
import type { PageLoad } from "./$types";
import { error } from "@sveltejs/kit";
import { api } from "$lib";

export const prerender = false;

type LuminaryProjectWithCompose = components["schemas"]["luminary_node.api.project.LuminaryProjectWithCompose"];

export const load: PageLoad = async ({ params }) => {
	const response = await api.client
		.GET("/api/project/{project}", {
			params: { path: { project: params.project } },
		})
		.catch(() => error(404, "Project not found"));

	const data = response.data! as LuminaryProject & Partial<LuminaryProjectWithCompose>;
	const compose = data.compose!;
	delete data.compose;

	// Update global project list immediately in case realtime patches haven't arrived yet
	api.putProject(data);

	return { compose };
};
