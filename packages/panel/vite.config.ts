import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
import { custom } from "./plugin";

export default defineConfig({
	plugins: [sveltekit(), custom()],
	server: {
		proxy: {
			"^/(api)/?.*": {
				target: "http://127.0.0.1:9000",
				changeOrigin: true,
			},
		},
	},
});
