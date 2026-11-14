import { Injectable, computed, signal } from '@angular/core';

import { invokeCommand, isTauriRuntime } from '../ipc/tauri-ipc';
import {
  readLocalFavorites,
  writeLocalFavorites,
  type FavoriteCollection,
  type FavoriteEntry,
} from './collection.models';

/**
 * User collection backed by `user.db` inside Tauri.
 *
 * The service keeps a single source of truth for starred species so every list
 * row, table cell and detail header stays in sync. Mutations are optimistic:
 * the UI reacts immediately and the returned collection replaces the local
 * copy once SQLite confirms it.
 */
@Injectable({ providedIn: 'root' })
export class CollectionService {
  private readonly entries = signal<readonly FavoriteEntry[]>([]);
  private readonly pending = signal<readonly number[]>([]);
  private loaded = false;

  readonly favorites = this.entries.asReadonly();
  private readonly ids = computed(() => new Set(this.entries().map((entry) => entry.entityId)));
  readonly count = computed(() => this.ids().size);
  readonly busy = computed(() => this.pending().length > 0);

  /** Increments after every successful mutation, for list refreshes. */
  readonly revision = signal(0);

  isFavorite(speciesId: number): boolean {
    return this.ids().has(speciesId);
  }

  isPending(speciesId: number): boolean {
    return this.pending().includes(speciesId);
  }

  async load(force = false): Promise<void> {
    if (this.loaded && !force) {
      return;
    }

    try {
      if (isTauriRuntime()) {
        const collection = await invokeCommand<FavoriteCollection>('list_favorites');
        this.entries.set(collection.items);
      } else {
        this.entries.set(readLocalFavorites());
      }
      this.loaded = true;
    } catch {
      this.loaded = false;
    }
  }

  async toggle(speciesId: number): Promise<void> {
    if (this.isPending(speciesId)) {
      return;
    }

    const favorite = !this.isFavorite(speciesId);
    this.pending.update((ids) => [...ids, speciesId]);
    this.applyOptimistically(speciesId, favorite);

    try {
      if (isTauriRuntime()) {
        const collection = await invokeCommand<FavoriteCollection>('set_favorite', {
          speciesId,
          favorite,
        });
        this.entries.set(collection.items);
      } else {
        writeLocalFavorites(this.entries());
      }
      this.loaded = true;
      this.revision.update((value) => value + 1);
    } catch {
      this.applyOptimistically(speciesId, !favorite);
    } finally {
      this.pending.update((ids) => ids.filter((id) => id !== speciesId));
    }
  }

  private applyOptimistically(speciesId: number, favorite: boolean): void {
    if (favorite) {
      const entry: FavoriteEntry = {
        kind: 'species',
        entityId: speciesId,
        createdAt: new Date().toISOString(),
      };
      this.entries.update((entries) =>
        entries.some((item) => item.entityId === speciesId) ? entries : [entry, ...entries],
      );
    } else {
      this.entries.update((entries) => entries.filter((entry) => entry.entityId !== speciesId));
    }
  }
}
