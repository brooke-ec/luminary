import type { paths, components } from "./openapi";
import type { Middleware } from "openapi-fetch";
import createClient from "openapi-fetch";
import { goto } from "$app/navigation";
import { error } from "$lib";

export * from "./state.svelte";

const TOKEN_KEY = "luminary-token";

type LuminaryFailResponse = components["schemas"]["luminary_node.api.response.LuminaryFailResponse"];

type LuminaryResponse =
	| components["schemas"]["luminary_node.api.response.LuminarySuccessResponse<()>"]
	| LuminaryFailResponse;

type ExcludeFail<T> = T extends LuminaryFailResponse ? never : T;
type UnwrapSuccess<T> = T extends { success: true; data: infer D } ? D : T;
type CaughtPaths = {
	[P in keyof paths]: {
		[M in keyof paths[P]]: paths[P][M] extends { responses: infer R }
			? {
					[K in keyof paths[P][M]]: K extends "responses"
						? {
								[S in keyof R]: R[S] extends { content: infer C }
									? {
											[H in keyof R[S]]: H extends "content"
												? { [CT in keyof C]: UnwrapSuccess<ExcludeFail<C[CT]>> }
												: R[S][H];
										}
									: R[S];
							}
						: paths[P][M][K];
				}
			: paths[P][M];
	};
};

/**
 * The token currently used for authentication.
 */
let token: string | null = localStorage.getItem(TOKEN_KEY);

/**
 * Check if the client is currently authenticated.
 * @returns `true` if the client is authenticated, `false` otherwise.
 */
export const isAuthenticated = () => !!token;

/**
 * @returns The current token used for authentication.
 */
export const getToken = () => token;

/**
 * Updates the token used for authentication and stores it in localStorage.
 * @param newToken The new token to use for authentication, or null to remove the token.
 */
function putToken(newToken: string | null) {
	if (newToken) localStorage.setItem(TOKEN_KEY, newToken);
	else localStorage.removeItem(TOKEN_KEY);

	token = newToken;
}

/**
 * Middleware that adds authentication to requests.
 */
const middleware: Middleware = {
	onRequest({ request }) {
		if (token) request.headers.set("Authorization", `Bearer ${token}`);
		return request;
	},
	async onResponse({ response, options }) {
		if (response.status === 401) {
			putToken(null);
			goto("/login");
			throw new Error("Unauthorized Request");
		} else if (options.parseAs == "json" && response.status === 200) {
			const payload: LuminaryResponse = await response.json();
			if (payload.success) return new Response(JSON.stringify(payload.data), response);
			else {
				error(payload.error[0], payload.error);
				throw new Error(payload.error[0]);
			}
		}

		return response;
	},
	onError({ error: e, options }) {
		if (options.parseAs !== "stream")
			error("Network Error", [
				"An error occurred while making a request to the backend." +
					" Please check your network connection and try again.",
				String(e),
			]);
	},
};

/**
 * The client used to make API requests to the backend.
 */
export const client = createClient<CaughtPaths>({ baseUrl: "./" });
client.use(middleware);

/**
 * Authenticate with the backend using the provided credentials.
 *
 * If successful, the token will be stored and used for future requests.
 * @param credentials The credentials to use for authentication.
 */
export async function login(credentials: components["schemas"]["luminary_node.api.auth.LuminaryUserCredentials"]) {
	const response = await client.POST("/api/auth/login", { body: credentials });
	putToken(response.data!);
}
