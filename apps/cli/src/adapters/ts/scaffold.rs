use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub async fn scaffold_ts_project(dir: &Path, name: &str) -> Result<()> {
    let spinner = cliclack::spinner();
    spinner.start("Initializing TypeScript project...");

    let project_dir = dir.join(name);
    fs::create_dir_all(&project_dir)?;

    fs::create_dir_all(project_dir.join("src"))?;
    fs::create_dir_all(project_dir.join("scripts"))?;

    // Note: Using relative path for SDK for dev/testing within monorepo.
    // In production, this should likely be a version number.
    let package_json = format!(
        r#"{{
  "name": "{}",
  "version": "0.1.0",
  "type": "module",
  "scripts": {{
    "build": "node scripts/build.mjs",
    "prebuild": "esbuild src/index.ts --bundle --format=esm --outfile=dist/index.js --platform=node --external:env-architect:plugin/*"
  }},
  "dependencies": {{
      "@env-architect/sdk": "file:../../packages/sdks/ts"
  }},
  "devDependencies": {{
    "@bytecodealliance/componentize-js": "^0.13.0",
    "@bytecodealliance/jco": "^1.0.0",
    "@types/node": "^20.0.0",
    "esbuild": "^0.19.0",
    "typescript": "^5.0.0"
  }}
}}"#,
        name
    );
    fs::write(project_dir.join("package.json"), package_json)
        .context("Failed to write package.json")?;

    let tsconfig = r#"{
    "compilerOptions": {
        "target": "ES2022",
        "module": "ESNext",
        "moduleResolution": "bundler",
        "outDir": "./dist",
        "rootDir": "./src",
        "strict": true,
        "esModuleInterop": true,
        "skipLibCheck": true,
        "forceConsistentCasingInFileNames": true
    },
    "include": ["src/**/*"]
}"#;
    fs::write(project_dir.join("tsconfig.json"), tsconfig)
        .context("Failed to write tsconfig.json")?;

    let index_ts = r#"import { Host, Plugin } from '@env-architect/sdk';

export const validate: Plugin.Validate = (manifest) => {
    Host.log.info('Validating plugin manifest...');
    return [];
};

export const resolve: Plugin.Resolve = (context) => {
    Host.log.info('Resolving configuration...');
    return {
        planJson: JSON.stringify({
            actions: [] // Define actions here
        }),
        state: null
    };
};

export const install: Plugin.Install = (context) => {
    Host.log.info('Installing plugin...');
};
"#;
    fs::write(project_dir.join("src/index.ts"), index_ts)
        .context("Failed to write src/index.ts")?;

    let build_script = r#"import { componentize } from '@bytecodealliance/componentize-js';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { resolve } from 'node:path';
import { existsSync } from 'node:fs';

async function build() {
    const input = 'dist/index.js';
    const output = 'plugin.wasm';
    const wit = resolve('node_modules/@env-architect/sdk/wit');

    if (!existsSync('dist')) {
        await mkdir('dist');
    }

    console.log('Componentizing...');
    const source = await readFile(input, 'utf8');
    const { component } = await componentize(source, {
        witPath: resolve(wit),
        worldName: 'plugin',
    });

    await writeFile(output, component);
    console.log('Build complete: ' + output);
}

build().catch(err => {
    console.error(err);
    process.exit(1);
});
"#;
    fs::write(project_dir.join("scripts/build.mjs"), build_script)
        .context("Failed to write scripts/build.mjs")?;

    spinner.start("Installing dependencies...");
    if std::process::Command::new("npm")
        .arg("install")
        .current_dir(&project_dir)
        .output()
        .is_err()
    {
        cliclack::log::warning(
            "Failed to install dependencies. Please run 'npm install' manually.",
        )?;
    }

    spinner.stop("TypeScript project initialized");
    Ok(())
}
