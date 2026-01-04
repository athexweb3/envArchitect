import * as RawHost from "env-architect:plugin/host";

export class Host {
	static get log() {
		return {
			debug: (msg: string) => RawHost.log("debug", msg),
			info: (msg: string) => RawHost.log("info", msg),
			warn: (msg: string) => RawHost.log("warn", msg),
			error: (msg: string) => RawHost.log("error", msg),
		};
	}

	static get ui() {
		return {
			confirm: RawHost.confirm,
			input: RawHost.input,
			secret: RawHost.secret,
			select: RawHost.select,
		};
	}

	static get fs() {
		return {
			readFile: RawHost.readFile,
			writeFile: RawHost.writeFile,
			createDir: RawHost.createDir,
			getEnv: RawHost.getEnv,
		};
	}

	static get sys() {
		return {
			exec: RawHost.exec,
		};
	}
}
