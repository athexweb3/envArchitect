import { existsSync } from "node:fs";
import { readFile, stat, writeFile } from "node:fs/promises";
import { dirname, isAbsolute, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { componentize } from "@bytecodealliance/componentize-js";

// console.log("DEBUG: build.ts loaded"); // Removed debug logs to be clean, now that we know it runs

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

export interface ComponentizeOptions {
	inputPath: string;
	outputPath: string;
	witPath?: string;
	worldName?: string;
	enableStdout?: boolean;
	optimize?: "0" | "s" | "3";
	sourcePath?: string;
}

export class Builder {
	private static readonly DEFAULT_WIT_PATH = resolve(__dirname, "../../../wit");

	static async build(opts: ComponentizeOptions): Promise<void> {
		const input = isAbsolute(opts.inputPath)
			? opts.inputPath
			: resolve(process.cwd(), opts.inputPath);
		const output = isAbsolute(opts.outputPath)
			? opts.outputPath
			: resolve(process.cwd(), opts.outputPath);
		const wit = opts.witPath
			? isAbsolute(opts.witPath)
				? opts.witPath
				: resolve(process.cwd(), opts.witPath)
			: Builder.DEFAULT_WIT_PATH;

		if (!existsSync(input)) {
			throw new Error(`Input file not found: ${input}`);
		}
		if (!existsSync(wit)) {
			throw new Error(`WIT directory not found: ${wit}`);
		}

		console.info("\n🏗️  Building Component...");
		console.info(`   XML Source: ${input}`);
		console.info(`   WIT World:  ${wit} [${opts.worldName || "plugin"}]`);

		const startTime = performance.now();

		try {
			const source = await readFile(input, "utf8");
			// @ts-expect-error
			const { component } = await componentize(source, {
				witPath: wit,
				worldName: opts.worldName || "plugin",
			});

			await writeFile(output, component);

			const duration = ((performance.now() - startTime) / 1000).toFixed(2);
			const stats = await stat(output);
			const sizeMb = (stats.size / 1024 / 1024).toFixed(2);

			console.info(`✅ Success! [${duration}s]`);
			console.info(`   Output:     ${output}`);
			console.info(`   Size:       ${sizeMb} MB\n`);
		} catch (error) {
			console.error("❌ Build Failed:", error);
			throw error;
		}
	}
}

const isMain =
	import.meta.url === `file://${process.argv[1]}` ||
	process.argv[1].endsWith("build.ts") ||
	process.argv[1].endsWith("build.js");

if (isMain) {
	const args = process.argv.slice(2);
	if (args.length < 2) {
		console.error(
			"Usage: bun scripts/build.ts <input.js> <output.wasm> [--debug]",
		);
		process.exit(1);
	}
	const input = args[0];
	const output = args[1];
	const enableStdout = args.includes("--debug");

	Builder.build({
		inputPath: input,
		outputPath: output,
		enableStdout,
	}).catch((e) => {
		console.error("Fatal Error:", e);
		process.exit(1);
	});
}
