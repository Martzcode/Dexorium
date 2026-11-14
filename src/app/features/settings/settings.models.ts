/** Contracts for the dataset provenance page (`get_dataset_info`). */

export interface DatasetRowCount {
  readonly table: string;
  readonly count: number;
}

export interface DatasetInfo {
  readonly version: string;
  readonly builtAt: string;
  readonly sourceCommit: string;
  readonly sourceLock: string;
  readonly schemaVersion: string;
  readonly languages: readonly string[];
  readonly rowCounts: readonly DatasetRowCount[];
  readonly license: string;
  readonly licenseUrl: string;
  readonly source: string;
  readonly sourceUrl: string;
}

/**
 * Translation keys for the tables counted by `pdx-dataset`.
 *
 * The keys mirror `insert_meta` in `pdx-dataset` (plus `forms`, used by the
 * in-memory fixture). Unknown tables fall back to their raw name.
 */
export const DATASET_TABLE_LABELS: Readonly<Record<string, string>> = {
  ability: 'settings.dataset.tables.ability',
  evolution_chain: 'settings.dataset.tables.evolutionChain',
  evolution_step: 'settings.dataset.tables.evolutionStep',
  flavor_text: 'settings.dataset.tables.flavorText',
  form_move: 'settings.dataset.tables.formMove',
  form_stat: 'settings.dataset.tables.formStat',
  form_stat_summary: 'settings.dataset.tables.formStatSummary',
  form_type: 'settings.dataset.tables.formType',
  forms: 'settings.dataset.tables.forms',
  move_: 'settings.dataset.tables.move',
  pokemon_form: 'settings.dataset.tables.pokemonForm',
  species: 'settings.dataset.tables.species',
  translation: 'settings.dataset.tables.translation',
  type_efficacy: 'settings.dataset.tables.typeEfficacy',
};
