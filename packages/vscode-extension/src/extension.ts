import * as path from "path";
import { type ExtensionContext, workspace } from "vscode";
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
		path.join("..", "..", "target", "debug", "lsp-server"),
	);

	const serverOptions: ServerOptions = {
		run: { command: serverPath, transport: TransportKind.stdio },
		debug: { command: serverPath, transport: TransportKind.stdio },
	};

	const clientOptions: LanguageClientOptions = {
		documentSelector: [{ scheme: "file", language: "env-toml" }],
	};

	client = new LanguageClient(
		"envArchitect",
		"EnvArchitect Language Server",
		serverOptions,
		clientOptions,
	);

	client.start();
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}
