import { HttpClient } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { firstValueFrom } from 'rxjs';

import { invokeCommand, isTauriRuntime } from '../../core/ipc/tauri-ipc';
import type {
  FixtureSpecies,
  SpeciesDetail,
  SpeciesListQuery,
  SpeciesPage,
  SpeciesSummary,
} from './pokedex.models';

const FIXTURE_URL = '/fixtures/species.json';
const DEFAULT_LIMIT = 1300;

/**
 * Facade consumed by the Pokédex pages.
 *
 * Inside Tauri the SQLite-backed commands answer; in a regular browser the
 * shared JSON fixture is loaded so UI work does not require the desktop
 * runtime. The fixture fallback implements the same filters, sort and
 * pagination contract as the Rust repository.
 */
@Injectable({ providedIn: 'root' })
export class PokedexService {
  private readonly http = inject(HttpClient);
  private fixture?: Promise<readonly FixtureSpecies[]>;

  async list(query: SpeciesListQuery): Promise<SpeciesPage> {
    if (isTauriRuntime()) {
      return invokeCommand<SpeciesPage>('list_pokemon', { query });
    }

    return listFixture(await this.fixtureSpecies(), query);
  }

  async search(query: string, lang: string, limit = DEFAULT_LIMIT): Promise<SpeciesSummary[]> {
    if (isTauriRuntime()) {
      return invokeCommand<SpeciesSummary[]>('search_species', { query, lang, limit });
    }

    return searchFixture(await this.fixtureSpecies(), query, lang, limit);
  }

  async detail(id: number, lang: string): Promise<SpeciesDetail | null> {
    if (isTauriRuntime()) {
      return invokeCommand<SpeciesDetail | null>('get_species', { id, lang });
    }

    const species = await this.fixtureSpecies();
    const match = species.find((entry) => entry.id === id);

    return match ? toDetail(match, lang) : null;
  }

  private fixtureSpecies(): Promise<readonly FixtureSpecies[]> {
    if (!this.fixture) {
      this.fixture = firstValueFrom(this.http.get<readonly FixtureSpecies[]>(FIXTURE_URL)).catch(
        (error: unknown) => {
          this.fixture = undefined;
          throw error;
        },
      );
    }

    return this.fixture;
  }
}

function listFixture(species: readonly FixtureSpecies[], query: SpeciesListQuery): SpeciesPage {
  const tokens = tokenize(query.search ?? '');
  const types = query.types.map((type) => type.toLowerCase());

  const filtered = species.filter((entry) => {
    if (!tokens.every((token) => matches(entry, token))) {
      return false;
    }

    if (types.length > 0 && !matchesTypes(entry, types, query.typeMatch)) {
      return false;
    }

    if (query.generations.length > 0 && !query.generations.includes(entry.generation)) {
      return false;
    }

    if (!query.statRanges.every((range) => matchesStatRange(entry, range))) {
      return false;
    }

    if (!matchesFlags(entry, query.flags)) {
      return false;
    }

    return true;
  });

  const sorted = [...filtered].sort((left, right) => compare(left, right, query));
  const start = Math.max(0, query.offset);
  const end = start + Math.max(0, query.limit);

  return {
    items: sorted.slice(start, end).map((entry) => toSummary(entry, query.lang)),
    total: sorted.length,
    offset: start,
    limit: query.limit,
  };
}

function matchesTypes(
  species: FixtureSpecies,
  selected: readonly string[],
  mode: SpeciesListQuery['typeMatch'],
): boolean {
  const types = species.types.map((type) => type.toLowerCase());

  switch (mode) {
    case 'exact':
      return types.length === selected.length && selected.every((type) => types.includes(type));
    case 'mono':
      return types.length === 1 && selected.includes(types[0]);
    case 'any':
    default:
      return selected.some((type) => types.includes(type));
  }
}

function matchesStatRange(
  species: FixtureSpecies,
  range: SpeciesListQuery['statRanges'][number],
): boolean {
  const value = range.stat === 'bst' ? statsTotal(species) : species.stats[range.stat];

  return (range.min === null || value >= range.min) && (range.max === null || value <= range.max);
}

function matchesFlags(
  species: FixtureSpecies,
  flags: SpeciesListQuery['flags'],
): boolean {
  const selected = [
    flags.legendary && species.isLegendary,
    flags.mythical && species.isMythical,
    flags.baby && false,
    flags.ultraBeast && false,
  ];

  if (!flags.legendary && !flags.mythical && !flags.baby && !flags.ultraBeast) {
    return true;
  }

  return selected.some(Boolean);
}

function compare(left: FixtureSpecies, right: FixtureSpecies, query: SpeciesListQuery): number {
  for (const criterion of query.sort) {
    const direction = criterion.direction === 'desc' ? -1 : 1;
    const result = compareField(left, right, criterion.field);

    if (result !== 0) {
      return result * direction;
    }
  }

  return left.nationalDexNumber - right.nationalDexNumber;
}

function compareField(
  left: FixtureSpecies,
  right: FixtureSpecies,
  field: SpeciesListQuery['sort'][number]['field'],
): number {
  switch (field) {
    case 'name':
      return left.names.en.localeCompare(right.names.en, undefined, { sensitivity: 'base' });
    case 'dexNumber':
      return left.nationalDexNumber - right.nationalDexNumber;
    case 'bst':
      return statsTotal(left) - statsTotal(right);
    default:
      return left.stats[field] - right.stats[field];
  }
}

function searchFixture(
  species: readonly FixtureSpecies[],
  query: string,
  lang: string,
  limit: number,
): SpeciesSummary[] {
  const tokens = tokenize(query);

  return species
    .filter((entry) => tokens.every((token) => matches(entry, token)))
    .sort((left, right) => left.nationalDexNumber - right.nationalDexNumber)
    .slice(0, limit)
    .map((entry) => toSummary(entry, lang));
}

function matches(species: FixtureSpecies, token: string): boolean {
  return searchableTokens(species).some((candidate) => candidate.startsWith(token));
}

function searchableTokens(species: FixtureSpecies): string[] {
  return [
    ...tokenize(species.names.fr),
    ...tokenize(species.names.en),
    ...tokenize(species.slug),
    species.nationalDexNumber.toString(),
  ];
}

function tokenize(value: string): string[] {
  return normalize(value)
    .split(/[^a-z0-9]+/)
    .filter((token) => token.length > 0);
}

function normalize(value: string): string {
  return value
    .toLowerCase()
    .replaceAll('œ', 'oe')
    .replaceAll('æ', 'ae')
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '');
}

function toSummary(species: FixtureSpecies, lang: string): SpeciesSummary {
  return {
    id: species.id,
    nationalDexNumber: species.nationalDexNumber,
    slug: species.slug,
    name: lang === 'fr' ? species.names.fr : species.names.en,
    types: species.types,
    stats: species.stats,
    bst: statsTotal(species),
  };
}

function toDetail(species: FixtureSpecies, lang: string): SpeciesDetail {
  return {
    ...toSummary(species, lang),
    stats: species.stats,
    heightDm: species.heightDm,
    weightHg: species.weightHg,
    captureRate: species.captureRate,
    baseHappiness: species.baseHappiness,
    isLegendary: species.isLegendary,
    isMythical: species.isMythical,
  };
}

function statsTotal(species: FixtureSpecies): number {
  const stats = species.stats;
  return stats.hp + stats.attack + stats.defense + stats.spAtk + stats.spDef + stats.speed;
}
