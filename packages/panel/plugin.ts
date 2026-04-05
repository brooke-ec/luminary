import openapiTS, { astToString } from "openapi-typescript";
import { writeFileSync } from "node:fs";
import { existsSync } from "node:fs";
import type { Plugin } from "vite";

const OPENAPI_PATH = new URL("../node/openapi.json", import.meta.url);

async function generateTypes() {
	if (!existsSync(OPENAPI_PATH)) return;
	const output = await openapiTS(OPENAPI_PATH);
	writeFileSync(new URL("src/lib/api/openapi.ts", import.meta.url), astToString(output));
}

export function custom(): Plugin {
	return {
		name: "custom",
		enforce: "pre",
		buildStart: generateTypes,
		configureServer(server) {
			const openapiPath = OPENAPI_PATH.pathname;
			server.watcher.add(openapiPath);
			server.watcher.on("change", async (changedPath) => {
				if (changedPath === openapiPath) await generateTypes();
			});
		},
	};
}
