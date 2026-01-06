import type { InstallationContext, ResolutionOutput } from "@bindings/plugin";

export type {
	InstallationContext,
	ResolutionOutput,
} from "@bindings/plugin";

export type { LogLevel } from "@bindings/interfaces/env-architect-plugin-host";

// User-facing Plugin Interface
// User-facing Plugin Interface
export namespace Plugin {
	export type Validate = (manifestJson: string) => string[];
	export type Resolve = (contextJson: string) => ResolutionOutput;
	export type Install = (context: InstallationContext) => void;
}

export interface InstallPlan {
	instructions: string[];
}

export interface ResolutionContext {
	projectRoot: string;
	targetOs?: "linux" | "macos" | "windows";
	targetArch?: "x86_64" | "aarch64";
	configuration?: any; // The full parsed content of env.toml/env.json
}

export type Capability =
	| "fs-read"
	| "fs-write"
	| "net-outbound"
	| "sys-exec"
	| "env-read";

export interface Asset {
	name: string;
	url: string;
	hash?: string;
	targetPath?: string;
}

export interface ServiceDef {
	command: string;
	args?: string[];
	env?: Record<string, string>;
	ports?: number[];
	readinessProbe?: {
		httpGet?: { path: string; port: number };
		tcpSocket?: { port: number };
	};
}

export interface PlatformConstraints {
	os: ("linux" | "macos" | "windows")[];
	arch: ("x86_64" | "aarch64")[];
}

export interface DependencySpec {
	version: string;
}

export interface PackageMetadata {
	name: string;
	version: string;
	description?: string;
	authors?: string[];
	dependencies?: Record<string, DependencySpec | string>;
	devDependencies?: Record<string, DependencySpec | string>;
	conflicts?: Record<string, string>;
	services?: Record<string, ServiceDef>;
	capabilities?: Capability[];
	assets?: Asset[];
	platform?: PlatformConstraints;
	intelligence?: IntelligenceData;
}

export type ResolutionAction =
	| { "managed-install": { manager: string; command: string } }
	| { "auto-shim": { url: string; binary_name: string } }
	| { "config-update": { path: string; patch: string } }
	| { "manual-prompt": { message: string; instructions: string } };

export interface IntelligenceData {
	proposed_actions: ResolutionAction[];
}
