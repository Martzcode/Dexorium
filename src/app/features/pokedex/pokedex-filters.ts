import { Component, computed, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';

import { AppIcon } from '../../shared/icon/app-icon';
import { typeBadgeClass } from './pokedex-format';
import { PokedexStore } from './pokedex.store';
import type { SpeciesFlags, SpeciesSortField, SpeciesStat } from './pokedex.models';

const TYPES = [
  'normal',
  'fire',
  'water',
  'electric',
  'grass',
  'ice',
  'fighting',
  'poison',
  'ground',
  'flying',
  'psychic',
  'bug',
  'rock',
  'ghost',
  'dragon',
  'dark',
  'steel',
  'fairy',
] as const;

const GENERATIONS = [1, 2, 3, 4, 5, 6, 7, 8, 9] as const;

interface StatFilter {
  readonly stat: SpeciesStat;
  readonly labelKey: string;
  readonly max: number;
}

const STAT_FILTERS: readonly StatFilter[] = [
  { stat: 'hp', labelKey: 'pokedex.stats.hp', max: 255 },
  { stat: 'attack', labelKey: 'pokedex.stats.attack', max: 255 },
  { stat: 'defense', labelKey: 'pokedex.stats.defense', max: 255 },
  { stat: 'spAtk', labelKey: 'pokedex.stats.spAtk', max: 255 },
  { stat: 'spDef', labelKey: 'pokedex.stats.spDef', max: 255 },
  { stat: 'speed', labelKey: 'pokedex.stats.speed', max: 255 },
  { stat: 'bst', labelKey: 'pokedex.stats.bst', max: 1_200 },
];

@Component({
  selector: 'app-pokedex-filters',
  imports: [TranslocoPipe, AppIcon],
  templateUrl: './pokedex-filters.html',
})
export class PokedexFilters {
  protected readonly store = inject(PokedexStore);

  protected readonly types = TYPES;
  protected readonly generations = GENERATIONS;
  protected readonly statFilters = STAT_FILTERS;
  protected readonly typeBadgeClass = typeBadgeClass;
  protected readonly flagFilters: readonly { key: keyof SpeciesFlags; label: string }[] = [
    { key: 'legendary', label: 'pokedex.detail.legendary' },
    { key: 'mythical', label: 'pokedex.detail.mythical' },
    { key: 'baby', label: 'pokedex.filters.baby' },
    { key: 'ultraBeast', label: 'pokedex.filters.ultraBeast' },
  ];

  protected readonly activeCount = computed(
    () =>
      this.store.types().length +
      this.store.generations().length +
      this.store.statRanges().length +
      (this.store.flags().legendary ? 1 : 0) +
      (this.store.flags().mythical ? 1 : 0) +
      (this.store.flags().baby ? 1 : 0) +
      (this.store.flags().ultraBeast ? 1 : 0) +
      (this.store.favoritesOnly() ? 1 : 0),
  );

  protected bounds(stat: SpeciesStat): { min: number | null; max: number | null } {
    return this.store.statRanges().find((range) => range.stat === stat) ?? { min: null, max: null };
  }

  protected flagActive(flag: keyof SpeciesFlags): boolean {
    return this.store.flags()[flag];
  }

  protected toggleFlag(flag: keyof SpeciesFlags): void {
    this.store.toggleFlag(flag);
  }

  protected onTypeMatchChange(value: string): void {
    this.store.setTypeMatch(value as 'any' | 'exact' | 'mono');
  }

  protected updateBound(stat: SpeciesStat, bound: 'min' | 'max', raw: string): void {
    const current = this.bounds(stat);
    const parsed = raw.trim() === '' ? null : Number(raw);
    const value =
      parsed !== null && Number.isFinite(parsed) ? Math.max(0, Math.trunc(parsed)) : null;

    this.store.setStatRange(
      stat,
      bound === 'min' ? value : current.min,
      bound === 'max' ? value : current.max,
    );
  }

  protected onSortChange(raw: string): void {
    if (!raw) {
      this.store.setSort(null);
      return;
    }

    const [field, direction] = raw.split(':') as [SpeciesSortField, 'asc' | 'desc'];
    this.store.setSort(field, direction);
  }

  protected sortValue(): string {
    const [criterion] = this.store.sort();
    return criterion ? `${criterion.field}:${criterion.direction}` : '';
  }
}
