import { Injectable } from '@angular/core';

import { invokeCommand, isTauriRuntime } from '../../core/ipc/tauri-ipc';
import type { DatasetInfo } from './settings.models';

/** Facade for the embedded dataset provenance. */
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
}
