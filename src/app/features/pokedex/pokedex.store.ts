import { Injectable, computed, inject, signal } from '@angular/core';

import {
  DEFAULT_QUERY,
  EMPTY_FLAGS,
  type SortCriterion,
  type SpeciesFlags,
  type SpeciesListQuery,
  type SpeciesSortField,
  type SpeciesStat,
  type SpeciesSummary,
  type StatRange,
  type TypeMatchMode,
} from './pokedex.models';
import { PokedexService } from './pokedex.service';

export type PokedexView = 'grid' | 'list' | 'table';

const PAGE_SIZE = 120;
const SEARCH_DEBOUNCE_MS = 120;

/**
 * Signal store backing the Pokédex list.
 *
 * It owns the filters, the current page and the pagination cursor. Every
 * mutation schedules a debounced reload so rapid typing does not flood the
 * IPC boundary.
 */
@Injectable({ providedIn: 'root' })
export class PokedexStore {
  private readonly pokedex = inject(PokedexService);

  readonly search = signal('');
  readonly types = signal<readonly string[]>([]);
  readonly typeMatch = signal<TypeMatchMode>('any');
  readonly generations = signal<readonly number[]>([]);
  readonly statRanges = signal<readonly StatRange[]>([]);
  readonly flags = signal<SpeciesFlags>(EMPTY_FLAGS);
  readonly favoritesOnly = signal(false);
  readonly sort = signal<readonly SortCriterion[]>([]);
  readonly view = signal<PokedexView>('grid');

  readonly items = signal<readonly SpeciesSummary[]>([]);
  readonly total = signal(0);
  readonly loading = signal(false);
  readonly error = signal(false);

  readonly hasMore = computed(() => this.items().length < this.total());
  readonly hasFilters = computed(
    () =>
      this.search().trim().length > 0 ||
      this.types().length > 0 ||
      this.generations().length > 0 ||
      this.statRanges().length > 0 ||
      this.flags().legendary ||
      this.flags().mythical ||
      this.flags().baby ||
      this.flags().ultraBeast ||
      this.favoritesOnly(),
  );

  private language = 'en';
  private initialized = false;
  private requestId = 0;
  private loadingMore = false;
  private timer?: ReturnType<typeof setTimeout>;

  setLanguage(language: string): void {
    if (this.initialized && language === this.language) {
      return;
    }

    this.language = language;
    this.initialized = true;
    this.scheduleReload(0);
  }

  setSearch(value: string): void {
    this.search.set(value);
    this.scheduleReload(SEARCH_DEBOUNCE_MS);
  }

  toggleType(type: string): void {
    const selected = this.types();
    this.types.set(
      selected.includes(type) ? selected.filter((entry) => entry !== type) : [...selected, type],
    );
    this.scheduleReload(0);
  }

  setTypeMatch(mode: TypeMatchMode): void {
    this.typeMatch.set(mode);
    this.scheduleReload(0);
  }

  toggleGeneration(generation: number): void {
    const selected = this.generations();
    this.generations.set(
      selected.includes(generation)
        ? selected.filter((entry) => entry !== generation)
        : [...selected, generation].sort((left, right) => left - right),
    );
    this.scheduleReload(0);
  }

  toggleFlag(flag: keyof SpeciesFlags): void {
    this.flags.update((flags) => ({ ...flags, [flag]: !flags[flag] }));
    this.scheduleReload(0);
  }

  /** Restricts the list to the user's favourite species. */
  toggleFavoritesOnly(): void {
    this.favoritesOnly.update((value) => !value);
    this.scheduleReload(0);
  }

  /** Reloads the first page, e.g. after the collection changed. */
  refresh(): void {
    this.scheduleReload(0);
  }

  setStatRange(stat: SpeciesStat, min: number | null, max: number | null): void {
    const remaining = this.statRanges().filter((range) => range.stat !== stat);
    const hasBounds = min !== null || max !== null;
    this.statRanges.set(hasBounds ? [...remaining, { stat, min, max }] : remaining);
    this.scheduleReload(0);
  }

  clearStatRanges(): void {
    this.statRanges.set([]);
    this.scheduleReload(0);
  }

  /** Cycles one column through ascending, descending and unsorted. */
  cycleSort(field: SpeciesSortField): void {
    const current = this.sort().find((criterion) => criterion.field === field);

    if (!current) {
      this.sort.set([{ field, direction: 'asc' }]);
    } else if (current.direction === 'asc') {
      this.sort.set([{ field, direction: 'desc' }]);
    } else {
      this.sort.set([]);
    }

    this.scheduleReload(0);
  }

  /** Adds a secondary sort criterion (shift-click on a table header). */
  addSort(field: SpeciesSortField): void {
    const selected = this.sort().filter((criterion) => criterion.field !== field);
    const existing = this.sort().find((criterion) => criterion.field === field);
    const direction = existing?.direction === 'asc' ? 'desc' : 'asc';

    this.sort.set([...selected, { field, direction }]);
    this.scheduleReload(0);
  }

  setView(view: PokedexView): void {
    this.view.set(view);
  }

  setSort(field: SpeciesSortField | null, direction: 'asc' | 'desc' = 'asc'): void {
    this.sort.set(field ? [{ field, direction }] : []);
    this.scheduleReload(0);
  }

  clearFilters(): void {
    this.search.set('');
    this.types.set([]);
    this.typeMatch.set('any');
    this.generations.set([]);
    this.statRanges.set([]);
    this.flags.set(EMPTY_FLAGS);
    this.favoritesOnly.set(false);
    this.scheduleReload(0);
  }

  retry(): void {
    this.scheduleReload(0);
  }

  async loadMore(): Promise<void> {
    if (this.loading() || this.loadingMore || !this.hasMore()) {
      return;
    }

    this.loadingMore = true;
    await this.fetch(this.items().length);
    this.loadingMore = false;
  }

  private scheduleReload(delay: number): void {
    if (this.timer) {
      clearTimeout(this.timer);
    }

    this.timer = setTimeout(() => {
      this.timer = undefined;
      void this.reload();
    }, delay);
  }

  private async reload(): Promise<void> {
    await this.fetch(0);
  }

  private async fetch(offset: number): Promise<void> {
    const requestId = ++this.requestId;
    if (offset === 0) {
      this.loading.set(true);
      this.error.set(false);
    }

    try {
      const page = await this.pokedex.list(this.buildQuery(offset));
      if (requestId !== this.requestId) {
        return;
      }

      this.total.set(page.total);
      this.items.update((items) => (offset === 0 ? page.items : [...items, ...page.items]));
      this.error.set(false);
    } catch {
      if (requestId !== this.requestId) {
        return;
      }

      this.error.set(true);
      if (offset === 0) {
        this.items.set([]);
        this.total.set(0);
      }
    } finally {
      if (requestId === this.requestId) {
        this.loading.set(false);
      }
    }
  }

  private buildQuery(offset: number): SpeciesListQuery {
    return {
      search: this.search().trim() || null,
      types: this.types(),
      typeMatch: this.typeMatch(),
      generations: this.generations(),
      statRanges: this.statRanges(),
      flags: this.flags(),
      favoritesOnly: this.favoritesOnly(),
      favoriteIds: null,
      sort: this.sort(),
      offset,
      limit: PAGE_SIZE,
      lang: this.language,
    };
  }
}
