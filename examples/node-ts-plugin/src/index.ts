import { Host, type Plugin } from "@env-architect/sdk";

export const validate: Plugin.Validate = (manifest) => {
	Host.log.info("Validating plugin manifest...");
	return [];
};

export const resolve: Plugin.Resolve = (context) => {
	Host.log.info("Resolving configuration...");
	return {
		planJson: JSON.stringify({
			actions: [], // Define actions here
		}),
		state: null,
	};
};

export const install: Plugin.Install = (context) => {
	Host.log.info("Installing plugin...");
};
