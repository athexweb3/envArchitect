import { Host } from "@api/host";
import {
	type Asset,
	type Capability,
	DependencySpec,
	type InstallPlan,
	type PackageMetadata,
	type PlatformConstraints,
	type ResolutionAction,
	type ResolutionContext,
	type ResolutionOutput,
	type ServiceDef,
} from "@api/types";

export class EnvBuilder {
	private manifest: Partial<PackageMetadata> = {
		dependencies: {},
		devDependencies: {},
		conflicts: {},
		services: {},
		capabilities: [],
		assets: [],
	};

	private planInstructions: string[] = [];

	constructor() {}

	/**
	 * Automatically loads configuration from 'env.json' in the project root.
	 */
	static fromContext(ctx: ResolutionContext | string): EnvBuilder {
		const builder = new EnvBuilder();
		// Parsing context logic (omitted for brevity, same as before)
		try {
			const envContent = Host.fs.readFile("env.json");
			const json = JSON.parse(envContent);
			if (json.project) {
				builder.manifest.name = json.project.name;
				builder.manifest.version = json.project.version;
				builder.manifest.description = json.project.description;
				builder.manifest.authors = json.project.authors;
			}
		} catch (e) {
			/* ignore */
		}
		return builder;
	}

	// --- Metadata Setters (Fluent API) ---

	project(name: string, version: string, description?: string): this {
		this.manifest.name = name;
		this.manifest.version = version;
		if (description) this.manifest.description = description;
		return this;
	}

	addDependency(name: string, versionReq: string): this {
		if (!this.manifest.dependencies) this.manifest.dependencies = {};
		this.manifest.dependencies[name] = { version: versionReq };
		return this;
	}

	addDevDependency(name: string, versionReq: string): this {
		if (!this.manifest.devDependencies) this.manifest.devDependencies = {};
		this.manifest.devDependencies[name] = { version: versionReq };
		return this;
	}

	conflict(pkg: string, reason: string): this {
		if (!this.manifest.conflicts) this.manifest.conflicts = {};
		this.manifest.conflicts[pkg] = reason;
		return this;
	}

	service(name: string, service: ServiceDef): this {
		if (!this.manifest.services) this.manifest.services = {};
		this.manifest.services[name] = service;
		return this;
	}

	capability(cap: Capability): this {
		if (!this.manifest.capabilities) this.manifest.capabilities = [];
		this.manifest.capabilities.push(cap);
		return this;
	}

	asset(asset: Asset): this {
		if (!this.manifest.assets) this.manifest.assets = [];
		this.manifest.assets.push(asset);
		return this;
	}

	supportPlatform(
		os: PlatformConstraints["os"][number],
		arch: PlatformConstraints["arch"][number],
	): this {
		if (!this.manifest.platform) {
			this.manifest.platform = { os: [], arch: [] };
		}
		if (!this.manifest.platform.os.includes(os)) {
			this.manifest.platform.os.push(os);
		}
		if (!this.manifest.platform.arch.includes(arch)) {
			this.manifest.platform.arch.push(arch);
		}
		return this;
	}

	// --- Plan Builders ---

	addInstruction(instruction: string): this {
		this.planInstructions.push(instruction);
		return this;
	}

	resolutionAction(action: ResolutionAction): this {
		if (!this.manifest.intelligence) {
			this.manifest.intelligence = { proposed_actions: [] };
		}
		this.manifest.intelligence.proposed_actions.push(action);
		return this;
	}

	/**
	 * Returns the built PackageMetadata (Manifest).
	 * Matching Rust SDK's `build()` behavior which returns the struct.
	 */
	build(): PackageMetadata {
		// Deep clone or return as is?
		// Returning as casted type
		return this.manifest as PackageMetadata;
	}

	/**
	 * Generates the ResolutionOutput required by the 'resolve' export.
	 */
	toResolution(state?: string): ResolutionOutput {
		const plan: InstallPlan = {
			instructions: this.planInstructions,
		};
		return {
			planJson: JSON.stringify(plan),
			state: state,
		};
	}
}
