import openapiTS, { astToString } from "openapi-typescript";
import { readFileSync, writeFileSync } from "node:fs";
import { existsSync } from "node:fs";
import type { Plugin } from "vite";

const OPENAPI_PATH = new URL("../node/openapi.json", import.meta.url);
const CARGO_TOML_PATH = new URL("../../Cargo.toml", import.meta.url);

export function getVersion() {
	const cargo = readFileSync(CARGO_TOML_PATH, "utf8");
	const version = cargo.match(/\[workspace\.package\][\s\S]*?\nversion\s*=\s*(.+)/)?.[1];
	return version ?? "unknown";
}

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
