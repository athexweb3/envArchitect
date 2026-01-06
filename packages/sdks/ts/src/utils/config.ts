import type { ResolutionContext } from "../api/types";

export class ConfigUtils {
	/**
	 * Helper to retrieve a plugin-specific configuration section.
	 * Checks root level, then `plugin.[key]`.
	 */
	static getConfig<T>(context: ResolutionContext, key: string): T | undefined {
		const config = context.configuration;
		if (!config) return undefined;

		// 1. Root level
		if (config[key]) return config[key] as T;

		// 2. Plugin namespace
		if (config.plugin && config.plugin[key]) return config.plugin[key] as T;

		// 3. Tool namespace
		if (config.tool && config.tool[key]) return config.tool[key] as T;

		return undefined;
	}
}
