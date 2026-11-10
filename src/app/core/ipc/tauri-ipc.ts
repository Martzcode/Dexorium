import { invoke } from '@tauri-apps/api/core';

/**
 * True when the Angular bundle runs inside the Tauri WebView.
 *
 * `ng serve` in a regular browser is still useful for UI work, so the app must
 * degrade gracefully instead of calling the IPC bridge unconditionally.
 */
export function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/**
 * Typed wrapper around Tauri's `invoke`.
 *
 * Components never call `invoke` directly: they go through a feature facade,
 * which in turn uses this single entry point.
 */
export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!isTauriRuntime()) {
    throw new Error(`IPC command "${command}" is unavailable outside the desktop runtime.`);
  }

  return invoke<T>(command, args);
}
