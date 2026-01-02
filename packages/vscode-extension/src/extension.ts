import * as path from "path";
import { commands, type ExtensionContext, workspace } from "vscode";
import {
	LanguageClient,
	type LanguageClientOptions,
	type ServerOptions,
	TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(context: ExtensionContext) {
	// In development, we run the binary directly from target/debug
	// In production, this would point to the bundled binary
	const serverPath = context.asAbsolutePath(
		path.join("..", "..", "target", "debug", "env-lsp"),
	);

	const serverOptions: ServerOptions = {
		run: { command: serverPath, transport: TransportKind.stdio },
		debug: { command: serverPath, transport: TransportKind.stdio },
	};

	const clientOptions: LanguageClientOptions = {
		documentSelector: [
			{ scheme: "file", language: "env-toml" },
			{ scheme: "file", language: "json", pattern: "**/env.json" },
			{ scheme: "file", language: "toml", pattern: "**/{env,plugin}.toml" },
		],
	};

	client = new LanguageClient(
		"envArchitect",
		"EnvArchitect Language Server",
		serverOptions,
		clientOptions,
	);

	context.subscriptions.push(
		commands.registerCommand("env-architect.restartServer", async () => {
			if (client) {
				await client.stop();
				await client.start();
			}
		}),
	);

	client.start();
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}
