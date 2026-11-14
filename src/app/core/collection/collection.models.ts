/** Favourite collection contracts, mirrored from `pdx-domain`. */

export type FavoriteKind = 'species';

export interface FavoriteEntry {
  readonly kind: FavoriteKind;
  readonly entityId: number;
  readonly createdAt: string;
}

export interface FavoriteCollection {
  readonly items: readonly FavoriteEntry[];
  readonly total: number;
}

const STORAGE_KEY = 'dexorium.favorites';

/**
 * Browser persistence used when the Tauri runtime is unavailable.
 *
 * It mirrors the SQLite behaviour closely enough to exercise the UI, and keeps
 * the "offline only" promise: nothing leaves the machine.
 */
export function readLocalFavorites(): FavoriteEntry[] {
  try {
    const raw = globalThis.localStorage?.getItem(STORAGE_KEY);
    if (!raw) {
      return [];
    }

    const parsed: unknown = JSON.parse(raw);
    if (!Array.isArray(parsed)) {
      return [];
    }

    return parsed
      .filter(
        (entry): entry is FavoriteEntry =>
          typeof entry === 'object' &&
          entry !== null &&
          'entityId' in entry &&
          typeof (entry as { entityId: unknown }).entityId === 'number',
      )
      .map((entry) => ({
        kind: 'species' as const,
        entityId: entry.entityId,
        createdAt: entry.createdAt ?? new Date().toISOString(),
      }));
  } catch {
    return [];
  }
}

export function writeLocalFavorites(entries: readonly FavoriteEntry[]): void {
  try {
    globalThis.localStorage?.setItem(STORAGE_KEY, JSON.stringify(entries));
  } catch {
    // Storage can be unavailable (private mode); the in-memory signal still works.
  }
}
