/**
 * @tauri-apps/api v2.10.1 shim
 * 
 * Tauri v2 webview injects window.__TAURI__ with { core, event }.
 * This shim provides the invoke/listen API used by the app,
 * without requiring a bundler or npm runtime.
 * 
 * Source: @tauri-apps/api@2.10.1 (npm), adapted for vanilla JS.
 * 
 * Usage:
 *   <script src="vendor/tauri.js"></script>
 *   <script>
 *     // invoke and listen are now global:
 *     const settings = await invoke('get_settings');
 *     await listen('recording-started', (e) => console.log(e));
 *   </script>
 */

(function (global) {
    'use strict';

    // Expose a fallback if __TAURI__ isn't injected (e.g., during development preview)
    function getCore() {
        if (global.__TAURI__ && global.__TAURI__.core) {
            return global.__TAURI__.core;
        }
        throw new Error(
            'Tauri API not available. Ensure the app is running inside the Tauri webview.\n' +
            'If previewing in a browser, window.__TAURI__ is not injected.\n' +
            'Run `cargo tauri dev` or `cargo tauri build` to test properly.'
        );
    }

    function getEvent() {
        if (global.__TAURI__ && global.__TAURI__.event) {
            return global.__TAURI__.event;
        }
        throw new Error(
            'Tauri event API not available. Ensure the app is running inside the Tauri webview.'
        );
    }

    // Stable public API matching @tauri-apps/api/core
    global.invoke = function invoke(cmd, args) {
        return getCore().invoke(cmd, args);
    };

    // Stable public API matching @tauri-apps/api/event
    global.listen = function listen(event, handler) {
        return getEvent().listen(event, handler);
    };

    // Expose the raw core/event objects for advanced use
    global.__TAURI_INVOKE__ = {
        core: getCore,
        event: getEvent,
    };

})(window);