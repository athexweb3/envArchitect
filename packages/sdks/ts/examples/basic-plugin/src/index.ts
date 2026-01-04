import {
	EnvBuilder,
	Host,
	type InstallationContext,
	type ResolutionOutput,
} from "@env-architect/sdk";

// 1. Metadata Validation
export function validate(manifestJson: string): string[] {
	Host.log.info("Validating manifest...");
	return []; // No errors
}

// 2. Dependency Resolution
export function resolve(contextJson: string): ResolutionOutput {
	Host.log.info("Resolving dependencies...");

	// Use the builder to construct our plan
	const builder = EnvBuilder.fromContext(contextJson)
		.addDependency("node", "20.x")
		.addInstruction("echo 'Hello from TypeScript Plugin! 🚀'");

	// Build the resolution output
	return builder.toResolution();
}

// 3. Installation (Side Effects)
export function install(context: InstallationContext): void {
	Host.log.info("Installing...");
	Host.fs.writeFile("ts-plugin-install.txt", "Installed via Wasm Component!");
}
