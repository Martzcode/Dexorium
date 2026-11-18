import { Injectable, inject } from '@angular/core';

import { invokeCommand, isTauriRuntime } from '../../core/ipc/tauri-ipc';
import { PokedexService } from '../pokedex/pokedex.service';
import type {
  AbilityDetail,
  AbilityListQuery,
  AbilityPage,
  GlobalSearchQuery,
  GlobalSearchResult,
  LearnsetPage,
  MoveDetail,
  MoveListQuery,
  MovePage,
  ItemDetail,
  ItemSummary,
  LocationDetail,
  LocationSummary,
  EncounterSummary,
  SearchHit,
  NatureSummary,
} from './catalog.models';
/**
 * Facade for the move and ability catalogues plus the federated search.
 *
 * Inside Tauri the SQLite-backed commands answer. In a regular browser the
 * fixture only ships species, so the catalogues stay empty and the search is
 * limited to the fixture species — enough to exercise the UI without the
 * desktop runtime.
 */
@Injectable({ providedIn: 'root' })
export class CatalogService {
  private readonly pokedex = inject(PokedexService);

  async listMoves(query: MoveListQuery): Promise<MovePage> {
    if (isTauriRuntime()) {
      return invokeCommand<MovePage>('list_moves', { query });
    }

    return { items: [], total: 0, offset: query.offset, limit: query.limit };
  }

  async moveDetail(id: number, lang: string): Promise<MoveDetail | null> {
    if (isTauriRuntime()) {
      return invokeCommand<MoveDetail | null>('get_move_detail', { id, lang });
    }

    return null;
  }

  async learnset(formId: number, lang: string, versionGroup: string | null): Promise<LearnsetPage> {
    if (isTauriRuntime()) {
      return invokeCommand<LearnsetPage>('get_pokemon_learnset', {
        formId,
        lang,
        versionGroup,
      });
    }

    return {
      formId,
      versionGroup: null,
      versionGroups: [],
      entries: [],
      total: 0,
    };
  }

  async listAbilities(query: AbilityListQuery): Promise<AbilityPage> {
    if (isTauriRuntime()) {
      return invokeCommand<AbilityPage>('list_abilities', { query });
    }

    return { items: [], total: 0, offset: query.offset, limit: query.limit };
  }

  async abilityDetail(id: number, lang: string): Promise<AbilityDetail | null> {
    if (isTauriRuntime()) {
      return invokeCommand<AbilityDetail | null>('get_ability_detail', { id, lang });
    }

    return null;
  }

  async searchGlobal(query: GlobalSearchQuery): Promise<GlobalSearchResult> {
    const term = query.query.trim();
    if (term.length === 0) {
      return { query: term, hits: [] };
    }

    if (isTauriRuntime()) {
      return invokeCommand<GlobalSearchResult>('search_global', { query });
    }

    const species = await this.pokedex.search(term, query.lang, query.limit || 20);
    const hits: SearchHit[] = species.map((entry) => ({
      entityType: 'species',
      id: entry.id,
      slug: entry.slug,
      name: entry.name,
      subtitle: `#${entry.nationalDexNumber.toString().padStart(4, '0')} · ${entry.types.join(' / ')}`,
      score: 1_000,
    }));

    return { query: term, hits };
  }

  async suggest(query: string, lang: string, limit = 8): Promise<readonly SearchHit[]> {
    const term = query.trim();
    if (term.length === 0) {
      return [];
    }

    if (isTauriRuntime()) {
      return invokeCommand<readonly SearchHit[]>('suggest', { query: term, lang, limit });
    }

    const result = await this.searchGlobal({ query: term, limit, lang });
    return result.hits;
  }

  async listItems(search: string | null, category: string | null, lang: string): Promise<ItemSummary[]> {
    if (isTauriRuntime()) {
      return invokeCommand<ItemSummary[]>('list_items', { search, category, lang, limit: 200, offset: 0 });
    }
    return [];
  }

  async itemDetail(id: number, lang: string): Promise<ItemDetail | null> {
    if (isTauriRuntime()) {
      return invokeCommand<ItemDetail | null>('get_item_detail', { id, lang });
    }
    return null;
  }

  async listLocations(search: string | null, lang: string): Promise<LocationSummary[]> {
    if (isTauriRuntime()) {
      return invokeCommand<LocationSummary[]>('list_locations', { search, lang, limit: 200, offset: 0 });
    }
    return [];
  }

  async locationDetail(id: number, lang: string): Promise<LocationDetail | null> {
    if (isTauriRuntime()) {
      return invokeCommand<LocationDetail | null>('get_location_detail', { id, lang });
    }
    return null;
  }

  async speciesEncounters(speciesId: number, lang: string): Promise<EncounterSummary[]> {
    if (isTauriRuntime()) {
      return invokeCommand<EncounterSummary[]>('get_species_encounters', { speciesId, lang });
    }
    return [];
  }

  async listNatures(lang: string): Promise<NatureSummary[]> {
    if (isTauriRuntime()) {
      return invokeCommand<NatureSummary[]>('list_natures', { lang });
    }
    return [];
  }
}
