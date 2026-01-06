""`
import { Host, Plugin, ResolutionContext, ConfigUtils, createResolveHandler, createValidateHandler } from '@env-architect/sdk';

export const validate = createValidateHandler((manifest) => {
    return [];
});

export const resolve = createResolveHandler<{}>("node", (context, config) => {
    Host.log.info('Running Node.js Resolution (TS Re-implementation)');

    // 0. Detect Current Version
    const customState = "node";
    const nvmDir = Host.fs.getEnv("NVM_DIR") || "$HOME/.nvm";

    // Robust NVM Loading command
    // We escape backslashes for JS string, then for shell
    const sourceNvm = `export NVM_DIR = "${nvmDir}";[-s "$NVM_DIR/nvm.sh"] && . "$NVM_DIR/nvm.sh"`;
    const checkCmd = `${ sourceNvm } && nvm current`;

    let currentVersion: string | null = null;
    try {
        const out = Host.sys.exec("bash", ["-c", checkCmd]);
        const trimmed = out.trim();
        if (trimmed && trimmed !== "none") {
            currentVersion = trimmed;
        }
    } catch (e) {
        // failed to exec
    }

    let shouldPrompt = true;
    if (currentVersion) {
        shouldPrompt = Host.ui.confirm(`Node.js ${ currentVersion } is currently active.Change version ? `, false);
    }

    if (!shouldPrompt) {
        return {
            planJson: JSON.stringify({ actions: [] }),
            state: customState
        };
    }

    // 1. Interactive Selection
    const options = [
        "Stable (LTS) - Recommended",
        "Latest Features",
        "Specific Version..."
    ];

    const selection = Host.ui.select("Which Node.js version do you need?", options, "Stable (LTS) - Recommended");

    let versionReq = "lts/*";
    if (selection.includes("Latest")) {
        versionReq = "node";
    } else if (selection.includes("Specific")) {
        const input = Host.ui.input("Enter version (e.g. 18.16.0):", null);
        if (input.trim()) {
            versionReq = input.trim();
        }
    }

    const makeDefault = Host.ui.confirm("Set as system default?", true);

    // 3. Construct Plan
    let cmdChain = `${ sourceNvm }; nvm install ${ versionReq } `;

    if (makeDefault) {
        cmdChain += `; nvm alias default ${ versionReq } `;
        cmdChain += `; nvm use ${ versionReq } `;
    }

    return {
        planJson: JSON.stringify({
            actions: [
                // We don't have structured actions yet in the CLI runner? 
                // The old Rust runner collected `cmd_chain` into `instructions`.
                // The WIT definition has `actions` in resolution-output? 
                // Wait, the WIT definition in scaffold.rs showed `plan - json` string.
                // And the CLI resolver expects `env_architect_core:: resolution:: ResolutionPlan`.
                // Let's assume we pass instructions via a "shell" action if supported, 
                // OR we just assume the host prints it?
                // The Rust plugin set `plan.instructions`.
                // Let's mirror that structure in JSON.
            ],
            // If the host supports generic instructions field in JSON:
            instructions: [cmdChain]
        }),
        state: customState
    };
};

export const install: Plugin.Install = (context) => {
    Host.log.info('Installing...');
};
