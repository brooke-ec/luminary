import type { LayoutLoad } from "./$types";
import { api } from "$lib";
import { redirect } from "@sveltejs/kit";

const TOKEN_KEY = "magic_link_token";

export const load: LayoutLoad = async ({ url }) => {
	const param = url.searchParams.get("token");
	if (param) {
		sessionStorage.setItem(TOKEN_KEY, param);
		return redirect(302, url.pathname);
	}

	const token = sessionStorage.getItem(TOKEN_KEY);
	if (!token) return {};

	try {
		const response = await api.client.GET("/api/auth/reset/{token}", { params: { path: { token } } });

		return {
			magic: {
				username: response.data!,
				token,
			},
		};
	} catch (err) {
		sessionStorage.removeItem(TOKEN_KEY);
		return {};
	}
};
