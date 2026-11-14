import { Injectable } from '@angular/core';

import { invokeCommand, isTauriRuntime } from '../../core/ipc/tauri-ipc';
import { neutralTypeMatrix, type TypeMatrix } from './tools.models';

/** Facade for the type effectiveness chart. */
@Injectable({ providedIn: 'root' })
export class ToolsService {
  async typeMatrix(generation: number | null, lang: string): Promise<TypeMatrix> {
    if (isTauriRuntime()) {
      return invokeCommand<TypeMatrix>('get_type_matrix', { generation, lang });
    }

    return neutralTypeMatrix(generation ?? 9);
  }
}
