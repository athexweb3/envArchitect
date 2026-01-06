import { ConfigUtils } from "../utils/config";
import type { Plugin, ResolutionContext, ResolutionOutput } from "./types";

/**
 * A typed handler for the resolve phase.
 * @param context The parsed resolution context.
 * @param config The injected configuration (if found).
 */
export type UserResolveHandler<C> = (
	context: ResolutionContext,
	config: C,
) => ResolutionOutput;

/**
 * Creates a raw Plugin.Resolve function that handles JSON parsing and configuration injection.
 *
 * @param configKey The configuration key to look for (e.g. "node").
 * @param handler The user-defined handler function.
 */
export function createResolveHandler<C>(
	configKey: string,
	handler: UserResolveHandler<Partial<C>>,
): Plugin.Resolve {
	return (contextJson: string) => {
		const context: ResolutionContext = JSON.parse(contextJson);
		const config = ConfigUtils.getConfig<C>(context, configKey) || {};
		return handler(context, config);
	};
}

/**
 * A typed handler for the validate phase.
 */
export type UserValidateHandler = (manifest: any) => string[];

/**
 * Creates a raw Plugin.Validate function that handles JSON parsing.
 */
export function createValidateHandler(
	handler: UserValidateHandler,
): Plugin.Validate {
	return (manifestJson: string) => {
		const manifest = JSON.parse(manifestJson);
		return handler(manifest);
	};
}
