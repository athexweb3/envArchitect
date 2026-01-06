import {
	EnvBuilder,
	Host,
	type InstallationContext,
	type ResolutionOutput,
} from "@env-architect/sdk";


export function validate(manifestJson: string): string[] {
	Host.log.info("Validating manifest...");
	return []; // No errors
}


export function resolve(contextJson: string): ResolutionOutput {
	Host.log.info("Resolving dependencies...");


	const builder = EnvBuilder.fromContext(contextJson)
		.addDependency("node", "20.x")
		.addInstruction("echo 'Hello from TypeScript Plugin! 🚀'");


	return builder.toResolution();
}


export function install(context: InstallationContext): void {
	Host.log.info("Installing...");
	Host.fs.writeFile("ts-plugin-install.txt", "Installed via Wasm Component!");
}
