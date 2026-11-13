/** Compact species row used by the Pokédex list and search results. */
export interface SpeciesSummary {
  readonly id: number;
  readonly nationalDexNumber: number;
  readonly slug: string;
  readonly name: string;
  readonly types: readonly string[];
  readonly stats: BaseStats;
  readonly bst: number;
}

/** Type filter mode, mirrored from the Rust domain. */
export type TypeMatchMode = 'any' | 'exact' | 'mono';

/** Sortable species column. */
export type SpeciesSortField =
  | 'dexNumber'
  | 'name'
  | 'bst'
  | 'hp'
  | 'attack'
  | 'defense'
  | 'spAtk'
  | 'spDef'
  | 'speed';

export type SortDirection = 'asc' | 'desc';

export interface SortCriterion {
  readonly field: SpeciesSortField;
  readonly direction: SortDirection;
}

export type SpeciesStat = 'hp' | 'attack' | 'defense' | 'spAtk' | 'spDef' | 'speed' | 'bst';

export interface StatRange {
  readonly stat: SpeciesStat;
  readonly min: number | null;
  readonly max: number | null;
}

export interface SpeciesFlags {
  readonly legendary: boolean;
  readonly mythical: boolean;
  readonly baby: boolean;
  readonly ultraBeast: boolean;
}

/** Filtered, sorted and paginated list query. */
export interface SpeciesListQuery {
  readonly search: string | null;
  readonly types: readonly string[];
  readonly typeMatch: TypeMatchMode;
  readonly generations: readonly number[];
  readonly statRanges: readonly StatRange[];
  readonly flags: SpeciesFlags;
  readonly sort: readonly SortCriterion[];
  readonly offset: number;
  readonly limit: number;
  readonly lang: string;
}

/** One page of species plus the total number of matches. */
export interface SpeciesPage {
  readonly items: readonly SpeciesSummary[];
  readonly total: number;
  readonly offset: number;
  readonly limit: number;
}

export const EMPTY_FLAGS: SpeciesFlags = {
  legendary: false,
  mythical: false,
  baby: false,
  ultraBeast: false,
};

export const DEFAULT_QUERY: Omit<SpeciesListQuery, 'lang' | 'offset' | 'limit'> = {
  search: null,
  types: [],
  typeMatch: 'any',
  generations: [],
  statRanges: [],
  flags: EMPTY_FLAGS,
  sort: [],
};

/** Base stats for a playable form. */
export interface BaseStats {
  readonly hp: number;
  readonly attack: number;
  readonly defense: number;
  readonly spAtk: number;
  readonly spDef: number;
  readonly speed: number;
}

/** Full species payload for the detail page. */
export interface SpeciesDetail extends SpeciesSummary {
  readonly stats: BaseStats;
  readonly heightDm: number;
  readonly weightHg: number;
  readonly captureRate: number;
  readonly baseHappiness: number | null;
  readonly isLegendary: boolean;
  readonly isMythical: boolean;
}

/** Raw shape of `pdx-infra/fixtures/species.json`. */
export interface FixtureSpecies {
  readonly id: number;
  readonly slug: string;
  readonly nationalDexNumber: number;
  readonly generation: number;
  readonly names: Readonly<Record<'fr' | 'en', string>>;
  readonly types: readonly string[];
  readonly stats: BaseStats;
  readonly heightDm: number;
  readonly weightHg: number;
  readonly captureRate: number;
  readonly baseHappiness: number | null;
  readonly isLegendary: boolean;
  readonly isMythical: boolean;
}
