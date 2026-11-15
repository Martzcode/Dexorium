import { Injectable } from '@angular/core';

import { invokeCommand, isTauriRuntime } from '../../core/ipc/tauri-ipc';
import type { DatasetInfo } from './settings.models';

export const DEVELOPER_PROFILE_URL = 'https://github.com/Martzcode';

/** Facade for the embedded dataset provenance and external links. */
@Injectable({ providedIn: 'root' })
export class SettingsService {
  async datasetInfo(): Promise<DatasetInfo> {
    if (isTauriRuntime()) {
      return invokeCommand<DatasetInfo>('get_dataset_info');
    }

    return {
      version: 'browser-preview',
      builtAt: '',
      sourceCommit: '',
      sourceLock: '',
      schemaVersion: '2',
      languages: ['en', 'fr'],
      rowCounts: [],
      license: 'BSD-3-Clause',
      licenseUrl: 'https://github.com/PokeAPI/api-data/blob/master/LICENSE.txt',
      source: 'PokeAPI/api-data',
      sourceUrl: 'https://github.com/PokeAPI/api-data',
    };
  }

  /** Opens the developer GitHub profile in the user's default browser. */
  async openDeveloperProfile(): Promise<void> {
    if (isTauriRuntime()) {
      await invokeCommand('open_external', { url: DEVELOPER_PROFILE_URL });
      return;
    }

    window.open(DEVELOPER_PROFILE_URL, '_blank', 'noopener,noreferrer');
  }
}
