/**
 * Shared models for the move and ability catalogues.
 *
 * The shapes mirror the Rust domain structs (`pdx-domain`) which serialize in
 * camelCase. Keeping the query types here means the IPC payloads and the
 * browser fallback share a single contract.
 */

export type SortDirection = 'asc' | 'desc';

export type MoveSortField = 'id' | 'name' | 'power' | 'accuracy' | 'pp';

export interface MoveSortCriterion {
  readonly field: MoveSortField;
  readonly direction: SortDirection;
}

export interface MoveListQuery {
  readonly search: string | null;
  readonly types: readonly string[];
  readonly damageClasses: readonly string[];
  readonly generations: readonly number[];
  readonly minPower: number | null;
  readonly maxPower: number | null;
  readonly sort: readonly MoveSortCriterion[];
  readonly offset: number;
  readonly limit: number;
  readonly lang: string;
}

export interface MoveSummary {
  readonly id: number;
  readonly slug: string;
  readonly name: string;
  readonly typeSlug: string;
  readonly damageClass: string;
  readonly power: number | null;
  readonly accuracy: number | null;
  readonly pp: number | null;
  readonly generationId: number;
}

export interface MovePage {
  readonly items: readonly MoveSummary[];
  readonly total: number;
  readonly offset: number;
  readonly limit: number;
}

export interface MoveLearner {
  readonly formId: number;
  readonly speciesId: number;
  readonly nationalDexNumber: number;
  readonly speciesName: string;
  readonly formName: string;
  readonly methods: readonly string[];
  readonly level: number | null;
}

export interface MoveDetail {
  readonly id: number;
  readonly slug: string;
  readonly name: string;
  readonly generationId: number;
  readonly typeSlug: string;
  readonly damageClass: string;
  readonly power: number | null;
  readonly accuracy: number | null;
  readonly pp: number | null;
  readonly priority: number;
  readonly effectChance: number | null;
  readonly target: string;
  readonly ailment: string | null;
  readonly critRateBonus: number;
  readonly drainPct: number;
  readonly healingPct: number;
  readonly minHits: number | null;
  readonly maxHits: number | null;
  readonly minTurns: number | null;
  readonly maxTurns: number | null;
  readonly flinchChance: number;
  readonly statChance: number;
  readonly flagsBitmask: number;
  readonly effectShort: string | null;
  readonly effectLong: string | null;
  readonly description: string | null;
  readonly learnerVersionGroup: string | null;
  readonly learners: readonly MoveLearner[];
}

export interface LearnsetEntry {
  readonly moveId: number;
  readonly slug: string;
  readonly name: string;
  readonly typeSlug: string;
  readonly damageClass: string;
  readonly power: number | null;
  readonly accuracy: number | null;
  readonly pp: number | null;
  readonly method: string;
  readonly level: number;
  readonly machine: string | null;
}

export interface VersionGroupRef {
  readonly id: number;
  readonly slug: string;
  readonly generationId: number;
  readonly orderIndex: number;
}

export interface LearnsetPage {
  readonly formId: number;
  readonly versionGroup: VersionGroupRef | null;
  readonly versionGroups: readonly VersionGroupRef[];
  readonly entries: readonly LearnsetEntry[];
  readonly total: number;
}

export interface AbilityListQuery {
  readonly search: string | null;
  readonly generations: readonly number[];
  readonly offset: number;
  readonly limit: number;
  readonly lang: string;
}

export interface AbilitySummary {
  readonly id: number;
  readonly slug: string;
  readonly name: string;
  readonly generationId: number;
  readonly isMainSeries: boolean;
  readonly holderCount: number;
}

export interface AbilityPage {
  readonly items: readonly AbilitySummary[];
  readonly total: number;
  readonly offset: number;
  readonly limit: number;
}

export interface AbilityHolder {
  readonly formId: number;
  readonly speciesId: number;
  readonly nationalDexNumber: number;
  readonly speciesName: string;
  readonly formName: string;
  readonly slot: number;
  readonly isHidden: boolean;
}

export interface AbilityDetail {
  readonly id: number;
  readonly slug: string;
  readonly name: string;
  readonly generationId: number;
  readonly isMainSeries: boolean;
  readonly effectShort: string | null;
  readonly effectLong: string | null;
  readonly description: string | null;
  readonly holders: readonly AbilityHolder[];
}

export type SearchEntityType = 'species' | 'move' | 'ability';

export interface SearchHit {
  readonly entityType: SearchEntityType;
  readonly id: number;
  readonly slug: string;
  readonly name: string;
  readonly subtitle: string;
  readonly score: number;
}

export interface GlobalSearchQuery {
  readonly query: string;
  readonly limit: number;
  readonly lang: string;
}

export interface GlobalSearchResult {
  readonly query: string;
  readonly hits: readonly SearchHit[];
}

/** Default page size shared with the Rust use cases. */
export const MOVE_PAGE_SIZE = 120;
export const ABILITY_PAGE_SIZE = 120;

/** Empty move query, used as the initial UI state. */
export function defaultMoveQuery(lang: string): MoveListQuery {
  return {
    search: null,
    types: [],
    damageClasses: [],
    generations: [],
    minPower: null,
    maxPower: null,
    sort: [{ field: 'id', direction: 'asc' }],
    offset: 0,
    limit: MOVE_PAGE_SIZE,
    lang,
  };
}

/** Empty ability query, used as the initial UI state. */
export function defaultAbilityQuery(lang: string): AbilityListQuery {
  return {
    search: null,
    generations: [],
    offset: 0,
    limit: ABILITY_PAGE_SIZE,
    lang,
  };
}
