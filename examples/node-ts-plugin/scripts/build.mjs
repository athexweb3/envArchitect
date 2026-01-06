import { existsSync } from "node:fs";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { componentize } from "@bytecodealliance/componentize-js";

async function build() {
	const input = "dist/index.js";
	const output = "plugin.wasm";
	const wit = resolve("node_modules/@env-architect/sdk/wit");

	if (!existsSync("dist")) {
		await mkdir("dist");
	}

	console.log("Componentizing...");
	const source = await readFile(input, "utf8");
	const { component } = await componentize(source, {
		witPath: resolve(wit),
		worldName: "plugin",
	});

	await writeFile(output, component);
	console.log("Build complete: " + output);
}

build().catch((err) => {
	console.error(err);
	process.exit(1);
});
