
// this file is generated — do not edit it


/// <reference types="@sveltejs/kit" />

/**
 * This module provides access to environment variables that are injected _statically_ into your bundle at build time and are limited to _private_ access.
 * 
 * |         | Runtime                                                                    | Build time                                                               |
 * | ------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
 * | Private | [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private) | [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private) |
 * | Public  | [`$env/dynamic/public`](https://svelte.dev/docs/kit/$env-dynamic-public)   | [`$env/static/public`](https://svelte.dev/docs/kit/$env-static-public)   |
 * 
 * Static environment variables are [loaded by Vite](https://vitejs.dev/guide/env-and-mode.html#env-files) from `.env` files and `process.env` at build time and then statically injected into your bundle at build time, enabling optimisations like dead code elimination.
 * 
 * **_Private_ access:**
 * 
 * - This module cannot be imported into client-side code
 * - This module only includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured)
 * 
 * For example, given the following build time environment:
 * 
 * ```env
 * ENVIRONMENT=production
 * PUBLIC_BASE_URL=http://site.com
 * ```
 * 
 * With the default `publicPrefix` and `privatePrefix`:
 * 
 * ```ts
 * import { ENVIRONMENT, PUBLIC_BASE_URL } from '$env/static/private';
 * 
 * console.log(ENVIRONMENT); // => "production"
 * console.log(PUBLIC_BASE_URL); // => throws error during build
 * ```
 * 
 * The above values will be the same _even if_ different values for `ENVIRONMENT` or `PUBLIC_BASE_URL` are set at runtime, as they are statically replaced in your code with their build time values.
 */
declare module '$env/static/private' {
	export const IMSETTINGS_INTEGRATE_DESKTOP: string;
	export const SHELL: string;
	export const npm_command: string;
	export const SESSION_MANAGER: string;
	export const __EGL_EXTERNAL_PLATFORM_CONFIG_DIRS: string;
	export const QT_SCREEN_SCALE_FACTORS: string;
	export const COLORTERM: string;
	export const XDG_CONFIG_DIRS: string;
	export const npm_config_cache: string;
	export const XDG_SESSION_PATH: string;
	export const NVM_INC: string;
	export const HISTCONTROL: string;
	export const XDG_MENU_PREFIX: string;
	export const TERM_PROGRAM_VERSION: string;
	export const GTK_IM_MODULE: string;
	export const HISTSIZE: string;
	export const HOSTNAME: string;
	export const ICEAUTHORITY: string;
	export const LANGUAGE: string;
	export const NODE: string;
	export const JAVA_HOME: string;
	export const SSH_AUTH_SOCK: string;
	export const XDG_DATA_HOME: string;
	export const GRADLE_HOME: string;
	export const XDG_CONFIG_HOME: string;
	export const XCURSOR_PATH: string;
	export const MEMORY_PRESSURE_WRITE: string;
	export const COLOR: string;
	export const npm_config_local_prefix: string;
	export const SDKMAN_CANDIDATES_DIR: string;
	export const XMODIFIERS: string;
	export const DESKTOP_SESSION: string;
	export const SSH_AGENT_PID: string;
	export const ZYPAK_LIB: string;
	export const GTK_RC_FILES: string;
	export const NO_AT_BRIDGE: string;
	export const FLATPAK_ID: string;
	export const GDK_CORE_DEVICE_EVENTS: string;
	export const npm_config_globalconfig: string;
	export const EDITOR: string;
	export const SDKMAN_BROKER_API: string;
	export const XDG_SEAT: string;
	export const PWD: string;
	export const ALSA_CONFIG_PATH: string;
	export const LOGNAME: string;
	export const XDG_SESSION_DESKTOP: string;
	export const XDG_SESSION_TYPE: string;
	export const npm_config_init_module: string;
	export const SYSTEMD_EXEC_PID: string;
	export const npm_config_tmp: string;
	export const _: string;
	export const XAUTHORITY: string;
	export const VSCODE_GIT_ASKPASS_NODE: string;
	export const container: string;
	export const GTK2_RC_FILES: string;
	export const GI_TYPELIB_PATH: string;
	export const HOME: string;
	export const SSH_ASKPASS: string;
	export const LANG: string;
	export const CARGO_HOME: string;
	export const XDG_CURRENT_DESKTOP: string;
	export const npm_package_version: string;
	export const MEMORY_PRESSURE_WATCH: string;
	export const SBX_CHROME_API_RQ: string;
	export const AT_SPI_BUS_ADDRESS: string;
	export const GIT_ASKPASS: string;
	export const XDG_SEAT_PATH: string;
	export const PULSE_CLIENTCONFIG: string;
	export const INVOCATION_ID: string;
	export const MANAGERPID: string;
	export const CODELLDB_LAUNCH_CONNECT_FILE: string;
	export const IMSETTINGS_MODULE: string;
	export const INIT_CWD: string;
	export const CHROME_DESKTOP: string;
	export const QT_QPA_PLATFORM: string;
	export const KDE_SESSION_UID: string;
	export const XDG_CACHE_HOME: string;
	export const npm_lifecycle_script: string;
	export const MOZ_GMP_PATH: string;
	export const NVM_DIR: string;
	export const VSCODE_GIT_ASKPASS_EXTRA_ARGS: string;
	export const VSCODE_PYTHON_AUTOACTIVATE_GUARD: string;
	export const npm_config_npm_version: string;
	export const XDG_SESSION_CLASS: string;
	export const TERM: string;
	export const npm_package_name: string;
	export const CHROME_WRAPPER: string;
	export const npm_config_prefix: string;
	export const ZYPAK_ZYGOTE_STRATEGY_SPAWN: string;
	export const FLATPAK_IDE_ENV: string;
	export const LESSOPEN: string;
	export const USER: string;
	export const SANDBOX_LD_LIBRARY_PATH: string;
	export const VSCODE_GIT_IPC_HANDLE: string;
	export const CARGO_INSTALL_ROOT: string;
	export const QT_WAYLAND_RECONNECT: string;
	export const KDE_SESSION_VERSION: string;
	export const PAM_KWALLET5_LOGIN: string;
	export const NPM_CONFIG_USERCONFIG: string;
	export const SDKMAN_DIR: string;
	export const DISPLAY: string;
	export const npm_lifecycle_event: string;
	export const SHLVL: string;
	export const NVM_CD_FLAGS: string;
	export const FLATPAK_SANDBOX_DIR: string;
	export const QT_IM_MODULE: string;
	export const XDG_VTNR: string;
	export const SDKMAN_CANDIDATES_API: string;
	export const XDG_SESSION_ID: string;
	export const MANAGERPIDFDID: string;
	export const npm_config_user_agent: string;
	export const ZYPAK_BIN: string;
	export const XDG_STATE_HOME: string;
	export const npm_execpath: string;
	export const FC_FONTATIONS: string;
	export const LD_LIBRARY_PATH: string;
	export const XDG_RUNTIME_DIR: string;
	export const DEBUGINFOD_URLS: string;
	export const DBUS_SYSTEM_BUS_ADDRESS: string;
	export const npm_package_json: string;
	export const GST_PLUGIN_SYSTEM_PATH: string;
	export const DEBUGINFOD_IMA_CERT_PATH: string;
	export const KDEDIRS: string;
	export const VSCODE_GIT_ASKPASS_MAIN: string;
	export const JOURNAL_STREAM: string;
	export const XDG_DATA_DIRS: string;
	export const KDE_FULL_SESSION: string;
	export const GDK_BACKEND: string;
	export const npm_config_noproxy: string;
	export const PATH: string;
	export const npm_config_node_gyp: string;
	export const ZYPAK_SPAWN_LATEST_ON_REEXEC: string;
	export const PYTHONUSERBASE: string;
	export const DBUS_SESSION_BUS_ADDRESS: string;
	export const SDKMAN_PLATFORM: string;
	export const npm_config_global_prefix: string;
	export const KDE_APPLICATIONS_AS_SCOPE: string;
	export const ALSA_CONFIG_DIR: string;
	export const MAIL: string;
	export const NVM_BIN: string;
	export const PULSE_SERVER: string;
	export const npm_node_execpath: string;
	export const npm_config_engine_strict: string;
	export const TERM_PROGRAM: string;
	export const NODE_ENV: string;
}

/**
 * This module provides access to environment variables that are injected _statically_ into your bundle at build time and are _publicly_ accessible.
 * 
 * |         | Runtime                                                                    | Build time                                                               |
 * | ------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
 * | Private | [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private) | [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private) |
 * | Public  | [`$env/dynamic/public`](https://svelte.dev/docs/kit/$env-dynamic-public)   | [`$env/static/public`](https://svelte.dev/docs/kit/$env-static-public)   |
 * 
 * Static environment variables are [loaded by Vite](https://vitejs.dev/guide/env-and-mode.html#env-files) from `.env` files and `process.env` at build time and then statically injected into your bundle at build time, enabling optimisations like dead code elimination.
 * 
 * **_Public_ access:**
 * 
 * - This module _can_ be imported into client-side code
 * - **Only** variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`) are included
 * 
 * For example, given the following build time environment:
 * 
 * ```env
 * ENVIRONMENT=production
 * PUBLIC_BASE_URL=http://site.com
 * ```
 * 
 * With the default `publicPrefix` and `privatePrefix`:
 * 
 * ```ts
 * import { ENVIRONMENT, PUBLIC_BASE_URL } from '$env/static/public';
 * 
 * console.log(ENVIRONMENT); // => throws error during build
 * console.log(PUBLIC_BASE_URL); // => "http://site.com"
 * ```
 * 
 * The above values will be the same _even if_ different values for `ENVIRONMENT` or `PUBLIC_BASE_URL` are set at runtime, as they are statically replaced in your code with their build time values.
 */
declare module '$env/static/public' {
	
}

/**
 * This module provides access to environment variables set _dynamically_ at runtime and that are limited to _private_ access.
 * 
 * |         | Runtime                                                                    | Build time                                                               |
 * | ------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
 * | Private | [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private) | [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private) |
 * | Public  | [`$env/dynamic/public`](https://svelte.dev/docs/kit/$env-dynamic-public)   | [`$env/static/public`](https://svelte.dev/docs/kit/$env-static-public)   |
 * 
 * Dynamic environment variables are defined by the platform you're running on. For example if you're using [`adapter-node`](https://github.com/sveltejs/kit/tree/main/packages/adapter-node) (or running [`vite preview`](https://svelte.dev/docs/kit/cli)), this is equivalent to `process.env`.
 * 
 * **_Private_ access:**
 * 
 * - This module cannot be imported into client-side code
 * - This module includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured)
 * 
 * > [!NOTE] In `dev`, `$env/dynamic` includes environment variables from `.env`. In `prod`, this behavior will depend on your adapter.
 * 
 * > [!NOTE] To get correct types, environment variables referenced in your code should be declared (for example in an `.env` file), even if they don't have a value until the app is deployed:
 * >
 * > ```env
 * > MY_FEATURE_FLAG=
 * > ```
 * >
 * > You can override `.env` values from the command line like so:
 * >
 * > ```sh
 * > MY_FEATURE_FLAG="enabled" npm run dev
 * > ```
 * 
 * For example, given the following runtime environment:
 * 
 * ```env
 * ENVIRONMENT=production
 * PUBLIC_BASE_URL=http://site.com
 * ```
 * 
 * With the default `publicPrefix` and `privatePrefix`:
 * 
 * ```ts
 * import { env } from '$env/dynamic/private';
 * 
 * console.log(env.ENVIRONMENT); // => "production"
 * console.log(env.PUBLIC_BASE_URL); // => undefined
 * ```
 */
declare module '$env/dynamic/private' {
	export const env: {
		IMSETTINGS_INTEGRATE_DESKTOP: string;
		SHELL: string;
		npm_command: string;
		SESSION_MANAGER: string;
		__EGL_EXTERNAL_PLATFORM_CONFIG_DIRS: string;
		QT_SCREEN_SCALE_FACTORS: string;
		COLORTERM: string;
		XDG_CONFIG_DIRS: string;
		npm_config_cache: string;
		XDG_SESSION_PATH: string;
		NVM_INC: string;
		HISTCONTROL: string;
		XDG_MENU_PREFIX: string;
		TERM_PROGRAM_VERSION: string;
		GTK_IM_MODULE: string;
		HISTSIZE: string;
		HOSTNAME: string;
		ICEAUTHORITY: string;
		LANGUAGE: string;
		NODE: string;
		JAVA_HOME: string;
		SSH_AUTH_SOCK: string;
		XDG_DATA_HOME: string;
		GRADLE_HOME: string;
		XDG_CONFIG_HOME: string;
		XCURSOR_PATH: string;
		MEMORY_PRESSURE_WRITE: string;
		COLOR: string;
		npm_config_local_prefix: string;
		SDKMAN_CANDIDATES_DIR: string;
		XMODIFIERS: string;
		DESKTOP_SESSION: string;
		SSH_AGENT_PID: string;
		ZYPAK_LIB: string;
		GTK_RC_FILES: string;
		NO_AT_BRIDGE: string;
		FLATPAK_ID: string;
		GDK_CORE_DEVICE_EVENTS: string;
		npm_config_globalconfig: string;
		EDITOR: string;
		SDKMAN_BROKER_API: string;
		XDG_SEAT: string;
		PWD: string;
		ALSA_CONFIG_PATH: string;
		LOGNAME: string;
		XDG_SESSION_DESKTOP: string;
		XDG_SESSION_TYPE: string;
		npm_config_init_module: string;
		SYSTEMD_EXEC_PID: string;
		npm_config_tmp: string;
		_: string;
		XAUTHORITY: string;
		VSCODE_GIT_ASKPASS_NODE: string;
		container: string;
		GTK2_RC_FILES: string;
		GI_TYPELIB_PATH: string;
		HOME: string;
		SSH_ASKPASS: string;
		LANG: string;
		CARGO_HOME: string;
		XDG_CURRENT_DESKTOP: string;
		npm_package_version: string;
		MEMORY_PRESSURE_WATCH: string;
		SBX_CHROME_API_RQ: string;
		AT_SPI_BUS_ADDRESS: string;
		GIT_ASKPASS: string;
		XDG_SEAT_PATH: string;
		PULSE_CLIENTCONFIG: string;
		INVOCATION_ID: string;
		MANAGERPID: string;
		CODELLDB_LAUNCH_CONNECT_FILE: string;
		IMSETTINGS_MODULE: string;
		INIT_CWD: string;
		CHROME_DESKTOP: string;
		QT_QPA_PLATFORM: string;
		KDE_SESSION_UID: string;
		XDG_CACHE_HOME: string;
		npm_lifecycle_script: string;
		MOZ_GMP_PATH: string;
		NVM_DIR: string;
		VSCODE_GIT_ASKPASS_EXTRA_ARGS: string;
		VSCODE_PYTHON_AUTOACTIVATE_GUARD: string;
		npm_config_npm_version: string;
		XDG_SESSION_CLASS: string;
		TERM: string;
		npm_package_name: string;
		CHROME_WRAPPER: string;
		npm_config_prefix: string;
		ZYPAK_ZYGOTE_STRATEGY_SPAWN: string;
		FLATPAK_IDE_ENV: string;
		LESSOPEN: string;
		USER: string;
		SANDBOX_LD_LIBRARY_PATH: string;
		VSCODE_GIT_IPC_HANDLE: string;
		CARGO_INSTALL_ROOT: string;
		QT_WAYLAND_RECONNECT: string;
		KDE_SESSION_VERSION: string;
		PAM_KWALLET5_LOGIN: string;
		NPM_CONFIG_USERCONFIG: string;
		SDKMAN_DIR: string;
		DISPLAY: string;
		npm_lifecycle_event: string;
		SHLVL: string;
		NVM_CD_FLAGS: string;
		FLATPAK_SANDBOX_DIR: string;
		QT_IM_MODULE: string;
		XDG_VTNR: string;
		SDKMAN_CANDIDATES_API: string;
		XDG_SESSION_ID: string;
		MANAGERPIDFDID: string;
		npm_config_user_agent: string;
		ZYPAK_BIN: string;
		XDG_STATE_HOME: string;
		npm_execpath: string;
		FC_FONTATIONS: string;
		LD_LIBRARY_PATH: string;
		XDG_RUNTIME_DIR: string;
		DEBUGINFOD_URLS: string;
		DBUS_SYSTEM_BUS_ADDRESS: string;
		npm_package_json: string;
		GST_PLUGIN_SYSTEM_PATH: string;
		DEBUGINFOD_IMA_CERT_PATH: string;
		KDEDIRS: string;
		VSCODE_GIT_ASKPASS_MAIN: string;
		JOURNAL_STREAM: string;
		XDG_DATA_DIRS: string;
		KDE_FULL_SESSION: string;
		GDK_BACKEND: string;
		npm_config_noproxy: string;
		PATH: string;
		npm_config_node_gyp: string;
		ZYPAK_SPAWN_LATEST_ON_REEXEC: string;
		PYTHONUSERBASE: string;
		DBUS_SESSION_BUS_ADDRESS: string;
		SDKMAN_PLATFORM: string;
		npm_config_global_prefix: string;
		KDE_APPLICATIONS_AS_SCOPE: string;
		ALSA_CONFIG_DIR: string;
		MAIL: string;
		NVM_BIN: string;
		PULSE_SERVER: string;
		npm_node_execpath: string;
		npm_config_engine_strict: string;
		TERM_PROGRAM: string;
		NODE_ENV: string;
		[key: `PUBLIC_${string}`]: undefined;
		[key: `${string}`]: string | undefined;
	}
}

/**
 * This module provides access to environment variables set _dynamically_ at runtime and that are _publicly_ accessible.
 * 
 * |         | Runtime                                                                    | Build time                                                               |
 * | ------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
 * | Private | [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private) | [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private) |
 * | Public  | [`$env/dynamic/public`](https://svelte.dev/docs/kit/$env-dynamic-public)   | [`$env/static/public`](https://svelte.dev/docs/kit/$env-static-public)   |
 * 
 * Dynamic environment variables are defined by the platform you're running on. For example if you're using [`adapter-node`](https://github.com/sveltejs/kit/tree/main/packages/adapter-node) (or running [`vite preview`](https://svelte.dev/docs/kit/cli)), this is equivalent to `process.env`.
 * 
 * **_Public_ access:**
 * 
 * - This module _can_ be imported into client-side code
 * - **Only** variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`) are included
 * 
 * > [!NOTE] In `dev`, `$env/dynamic` includes environment variables from `.env`. In `prod`, this behavior will depend on your adapter.
 * 
 * > [!NOTE] To get correct types, environment variables referenced in your code should be declared (for example in an `.env` file), even if they don't have a value until the app is deployed:
 * >
 * > ```env
 * > MY_FEATURE_FLAG=
 * > ```
 * >
 * > You can override `.env` values from the command line like so:
 * >
 * > ```sh
 * > MY_FEATURE_FLAG="enabled" npm run dev
 * > ```
 * 
 * For example, given the following runtime environment:
 * 
 * ```env
 * ENVIRONMENT=production
 * PUBLIC_BASE_URL=http://example.com
 * ```
 * 
 * With the default `publicPrefix` and `privatePrefix`:
 * 
 * ```ts
 * import { env } from '$env/dynamic/public';
 * console.log(env.ENVIRONMENT); // => undefined, not public
 * console.log(env.PUBLIC_BASE_URL); // => "http://example.com"
 * ```
 * 
 * ```
 * 
 * ```
 */
declare module '$env/dynamic/public' {
	export const env: {
		[key: `PUBLIC_${string}`]: string | undefined;
	}
}
