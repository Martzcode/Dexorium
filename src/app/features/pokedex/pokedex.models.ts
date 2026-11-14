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
  'dexNumber' | 'name' | 'bst' | 'hp' | 'attack' | 'defense' | 'spAtk' | 'spDef' | 'speed';

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
  readonly favoritesOnly: boolean;
  readonly favoriteIds: readonly number[] | null;
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
  favoritesOnly: false,
  favoriteIds: null,
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

/** One playable form of a species. */
export interface PokemonForm {
  readonly id: number;
  readonly slug: string;
  readonly name: string;
  readonly isDefault: boolean;
  readonly isMega: boolean;
  readonly isGigantamax: boolean;
  readonly isRegional: boolean;
  readonly region: string | null;
  readonly generationId: number;
  readonly types: readonly string[];
  readonly stats: BaseStats;
  readonly bst: number;
  readonly heightDm: number;
  readonly weightHg: number;
}

/** Ability attached to the selected form. */
export interface AbilityRef {
  readonly id: number;
  readonly slug: string;
  readonly name: string;
  readonly slot: number;
  readonly isHidden: boolean;
}

/** One localized Pokédex description. */
export interface FlavorTextEntry {
  readonly versionId: number;
  readonly versionSlug: string;
  readonly generationId: number;
  readonly text: string;
}

/** Defensive multiplier of one attacking type against the selected form. */
export interface TypeEffectiveness {
  readonly attackingType: string;
  readonly multiplierPercent: number;
}

/** Flat evolution edge, rebuilt into the family view by the client. */
export interface EvolutionStep {
  readonly fromSpeciesId: number | null;
  readonly fromName: string | null;
  readonly toSpeciesId: number;
  readonly toName: string;
  readonly toNationalDexNumber: number;
  readonly trigger: string;
  readonly minLevel: number | null;
  readonly triggerItem: string | null;
  readonly heldItem: string | null;
  readonly timeOfDay: string | null;
  readonly location: string | null;
  readonly knownMove: string | null;
  readonly knownMoveType: string | null;
  readonly minHappiness: number | null;
  readonly minAffection: number | null;
  readonly minBeauty: number | null;
  readonly gender: string | null;
  readonly needsOverworldRain: boolean;
  readonly turnUpsideDown: boolean;
  readonly relativePhysicalStats: number | null;
  readonly conditionExpression: string | null;
  readonly extraConditionJson: string | null;
  readonly tradeSpecies: string | null;
  readonly partySpecies: string | null;
  readonly partyType: string | null;
}

/** Aggregated payload for the species detail page. */
export interface PokemonDetail {
  readonly speciesId: number;
  readonly nationalDexNumber: number;
  readonly slug: string;
  readonly name: string;
  readonly genus: string | null;
  readonly generationId: number;
  readonly isLegendary: boolean;
  readonly isMythical: boolean;
  readonly isBaby: boolean;
  readonly isUltraBeast: boolean;
  readonly captureRate: number;
  readonly baseHappiness: number | null;
  readonly hatchCounter: number | null;
  readonly genderRate: number;
  readonly growthRate: string;
  readonly color: string | null;
  readonly shape: string | null;
  readonly habitat: string | null;
  readonly eggGroups: readonly string[];
  readonly form: PokemonForm;
  readonly forms: readonly PokemonForm[];
  readonly abilities: readonly AbilityRef[];
  readonly evolutionChainId: number | null;
  readonly evolutions: readonly EvolutionStep[];
  readonly flavorTexts: readonly FlavorTextEntry[];
  readonly effectiveness: readonly TypeEffectiveness[];
  readonly effectivenessGenerationId: number;
}
