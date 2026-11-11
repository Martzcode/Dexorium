import { HttpClient } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { firstValueFrom } from 'rxjs';

import { invokeCommand, isTauriRuntime } from '../../core/ipc/tauri-ipc';
import type { FixtureSpecies, SpeciesDetail, SpeciesSummary } from './pokedex.models';

const FIXTURE_URL = '/fixtures/species.json';
const DEFAULT_LIMIT = 60;

/**
 * Facade consumed by the Pokédex pages.
 *
 * Inside Tauri the SQLite-backed commands answer; in a regular browser the
 * shared JSON fixture is loaded so UI work does not require the desktop
 * runtime.
 */
@Injectable({ providedIn: 'root' })
export class PokedexService {
  private readonly http = inject(HttpClient);
  private fixture?: Promise<readonly FixtureSpecies[]>;

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
