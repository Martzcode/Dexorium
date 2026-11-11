/** Compact species row used by the Pokédex list and search results. */
export interface SpeciesSummary {
  readonly id: number;
  readonly nationalDexNumber: number;
  readonly slug: string;
  readonly name: string;
  readonly types: readonly string[];
  readonly bst: number;
}

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
