import { Injectable, signal } from '@angular/core';

import { invokeCommand, isTauriRuntime } from '../ipc/tauri-ipc';

export type AppInfoStatus = 'loading' | 'ready' | 'browser' | 'error';

export interface AppInfo {
  readonly name: string;
  readonly version: string;
  readonly runtime: string;
  readonly platform: string;
  readonly arch: string;
  readonly offline: boolean;
}

@Injectable({ providedIn: 'root' })
export class AppInfoService {
  readonly status = signal<AppInfoStatus>('loading');
  readonly info = signal<AppInfo | null>(null);
  readonly error = signal<string | null>(null);

  constructor() {
    void this.load();
  }

  async load(): Promise<void> {
    if (!isTauriRuntime()) {
      this.status.set('browser');
      return;
    }

    try {
      const info = await invokeCommand<AppInfo>('get_app_info');
      this.info.set(info);
      this.status.set('ready');
    } catch (error) {
      this.error.set(error instanceof Error ? error.message : String(error));
      this.status.set('error');
    }
  }
}
