import { Injectable, signal } from '@angular/core';
import { open } from '@tauri-apps/plugin-dialog';
import { isTauriRuntime } from '../ipc/tauri-ipc';

const STORAGE_KEY = 'dexorium.sprites.folder';

@Injectable({ providedIn: 'root' })
export class SpriteFolderService {
  readonly folder = signal<string | null>(this.load());

  private load(): string | null {
    try {
      return localStorage.getItem(STORAGE_KEY);
    } catch {
      return null;
    }
  }

  private persist(value: string | null): void {
    try {
      if (value) {
        localStorage.setItem(STORAGE_KEY, value);
      } else {
        localStorage.removeItem(STORAGE_KEY);
      }
    } catch {
      // Ignore storage failures (private mode, quota, ...).
    }
  }

  async pickFolder(): Promise<void> {
    if (!isTauriRuntime()) {
      return;
    }

    const selected = await open({ directory: true, multiple: false, title: 'Sprites folder' });
    if (typeof selected === 'string' && selected.length > 0) {
      this.folder.set(selected);
      this.persist(selected);
    }
  }

  clear(): void {
    this.folder.set(null);
    this.persist(null);
  }
}
