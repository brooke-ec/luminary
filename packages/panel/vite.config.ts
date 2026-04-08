import { custom, getVersion } from "./plugin";
import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

export default defineConfig({
	plugins: [sveltekit(), custom()],
	define: {
		__LUMINARY_VERSION__: getVersion(),
	},
	server: {
		proxy: {
			"^/(api)/?.*": {
				target: "http://127.0.0.1:9000",
				changeOrigin: true,
			},
		},
		allowedHosts: true,
	},
});
