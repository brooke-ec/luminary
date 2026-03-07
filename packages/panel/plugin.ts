import openapiTS, { astToString } from "openapi-typescript";
import { writeFileSync } from "node:fs";
import type { Plugin } from "vite";

async function generateTypes() {
	const output = await openapiTS(new URL("src/lib/openapi.json", import.meta.url));
	writeFileSync(new URL("src/lib/openapi.ts", import.meta.url), astToString(output));
}

export function custom(): Plugin {
	return {
		name: "custom",
		enforce: "pre",
		buildStart: generateTypes,
		configureServer(server) {
			const path = new URL("src/lib/openapi.ts", import.meta.url).toString();
			server.watcher.add(path);
			server.watcher.on("change", async (path) => {
				if (path === path) await generateTypes();
			});
		},
	};
}
