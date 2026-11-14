/** Contracts for the type effectiveness chart (`get_type_matrix`). */

export interface TypeRef {
  readonly slug: string;
  readonly name: string;
  readonly introducedIn: number;
}

export interface TypeMatrix {
  readonly generation: number;
  readonly generations: readonly number[];
  readonly types: readonly TypeRef[];
  /** Row-major percentages: `multipliers[attacker][defender]`. */
  readonly multipliers: readonly (readonly number[])[];
}

export const TYPE_SLUGS = [
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

/** Neutral chart used by the browser fallback. */
export function neutralTypeMatrix(generation: number): TypeMatrix {
  const types = TYPE_SLUGS.map((slug) => ({
    slug,
    name: slug,
    introducedIn: 1,
  }));

  return {
    generation,
    generations: [9, 8, 7, 6, 5, 4, 3, 2, 1],
    types,
    multipliers: types.map(() => types.map(() => 100)),
  };
}
