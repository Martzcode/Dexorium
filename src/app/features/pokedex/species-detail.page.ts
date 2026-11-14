import { Component, computed, effect, inject, signal } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { ActivatedRoute, RouterLink } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';

import { AppIcon } from '../../shared/icon/app-icon';
import { PokemonSprite } from '../../shared/pokemon-sprite/pokemon-sprite';
import { CollectionService } from '../../core/collection/collection.service';
import { formatDexNumber, typeBadgeClass } from './pokedex-format';
import { SpeciesLearnset } from './species-learnset';
import type {
  EvolutionStep,
  FlavorTextEntry,
  PokemonDetail,
  PokemonForm,
  TypeEffectiveness,
} from './pokedex.models';
import { PokedexService } from './pokedex.service';

type LoadState = 'loading' | 'ready' | 'notFound' | 'error';

interface StatRow {
  readonly key: string;
  readonly labelKey: string;
  readonly value: number;
}

interface EffectivenessGroup {
  readonly key: string;
  readonly types: readonly TypeEffectiveness[];
}

interface FlavorGroup {
  readonly generationId: number;
  readonly entries: readonly FlavorTextEntry[];
}

const STAT_DEFINITIONS: readonly StatRow[] = [
  { key: 'hp', labelKey: 'pokedex.stats.hp', value: 0 },
  { key: 'attack', labelKey: 'pokedex.stats.attack', value: 0 },
  { key: 'defense', labelKey: 'pokedex.stats.defense', value: 0 },
  { key: 'spAtk', labelKey: 'pokedex.stats.spAtk', value: 0 },
  { key: 'spDef', labelKey: 'pokedex.stats.spDef', value: 0 },
  { key: 'speed', labelKey: 'pokedex.stats.speed', value: 0 },
];

const GENDER_KEYS: Readonly<Record<string, string>> = {
  male: 'pokedex.detail.breeding.male',
  female: 'pokedex.detail.breeding.female',
};

@Component({
  selector: 'app-species-detail',
  imports: [RouterLink, TranslocoPipe, AppIcon, PokemonSprite, SpeciesLearnset],
  templateUrl: './species-detail.page.html',
})
export class SpeciesDetailPage {
  private readonly route = inject(ActivatedRoute);
  private readonly pokedex = inject(PokedexService);
  private readonly transloco = inject(TranslocoService);

  protected readonly collection = inject(CollectionService);

  protected readonly species = signal<PokemonDetail | null>(null);
  protected readonly state = signal<LoadState>('loading');

  protected readonly formatDexNumber = formatDexNumber;
  protected readonly typeBadgeClass = typeBadgeClass;

  private selectedFormId: number | null = null;
  private requestId = 0;

  private readonly parameters = toSignal(this.route.paramMap, {
    initialValue: this.route.snapshot.paramMap,
  });
  private readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  protected readonly statRows = computed<readonly StatRow[]>(() => {
    const stats = this.species()?.form.stats;
    if (!stats) {
      return [];
    }

    return STAT_DEFINITIONS.map((definition) => ({
      ...definition,
      value: stats[definition.key as keyof typeof stats],
    }));
  });

  protected readonly previousId = computed(() => {
    const id = this.species()?.speciesId ?? 0;
    return id > 1 ? id - 1 : null;
  });

  protected readonly nextId = computed(() => {
    const id = this.species()?.speciesId ?? 0;
    return id > 0 ? id + 1 : null;
  });

  protected readonly genderSplit = computed(() => {
    const rate = this.species()?.genderRate ?? -1;
    if (rate < 0) {
      return null;
    }

    const female = Math.round((rate / 8) * 100);
    return { female, male: 100 - female };
  });

  protected readonly effectGroups = computed<readonly EffectivenessGroup[]>(() => {
    const entries = this.species()?.effectiveness ?? [];
    if (entries.length === 0) {
      return [];
    }

    const sortTypes = (values: readonly TypeEffectiveness[]) =>
      [...values].sort((left, right) => left.attackingType.localeCompare(right.attackingType));

    return [
      {
        key: 'weaknesses',
        types: sortTypes(entries.filter((entry) => entry.multiplierPercent > 100)).sort(
          (left, right) => right.multiplierPercent - left.multiplierPercent,
        ),
      },
      {
        key: 'resistances',
        types: sortTypes(
          entries.filter((entry) => entry.multiplierPercent > 0 && entry.multiplierPercent < 100),
        ),
      },
      {
        key: 'immunities',
        types: sortTypes(entries.filter((entry) => entry.multiplierPercent === 0)),
      },
    ].filter((group) => group.types.length > 0);
  });

  protected readonly flavorGroups = computed<readonly FlavorGroup[]>(() => {
    const entries = this.species()?.flavorTexts ?? [];
    const groups = new Map<number, FlavorTextEntry[]>();

    for (const entry of entries) {
      const bucket = groups.get(entry.generationId) ?? [];
      bucket.push(entry);
      groups.set(entry.generationId, bucket);
    }

    return [...groups.entries()]
      .sort(([left], [right]) => right - left)
      .map(([generationId, groupEntries]) => ({ generationId, entries: groupEntries }));
  });

  constructor() {
    effect(() => {
      const id = Number(this.parameters().get('id'));
      const language = this.language();

      this.selectedFormId = null;
      void this.load(id, language);
    });
  }

  protected retry(): void {
    void this.load(Number(this.parameters().get('id')), this.language());
  }

  protected selectForm(form: PokemonForm): void {
    this.selectedFormId = form.id;
    void this.load(this.species()?.speciesId ?? 0, this.language());
  }

  protected onFormChange(event: Event): void {
    const id = Number((event.target as HTMLSelectElement).value);
    const form = this.species()?.forms.find((entry) => entry.id === id);
    if (form) {
      this.selectForm(form);
    }
  }

  protected isSelectedForm(form: PokemonForm): boolean {
    return this.species()?.form.id === form.id;
  }

  protected toggleFavorite(): void {
    const id = this.species()?.speciesId;
    if (id) {
      void this.collection.toggle(id);
    }
  }

  protected favoriteLabel(): string {
    const id = this.species()?.speciesId ?? 0;
    return this.collection.isFavorite(id) ? 'pokedex.favorite.remove' : 'pokedex.favorite.add';
  }

  protected formBadges(form: PokemonForm): readonly string[] {
    const badges: string[] = [];
    if (form.isMega) {
      badges.push('pokedex.detail.form.mega');
    }
    if (form.isGigantamax) {
      badges.push('pokedex.detail.form.gigantamax');
    }
    if (form.isRegional) {
      badges.push('pokedex.detail.form.regional');
    }
    if (form.isDefault) {
      badges.push('pokedex.detail.form.default');
    }
    return badges;
  }

  protected statPercent(value: number): number {
    return Math.min(100, Math.round((value / 255) * 100));
  }

  protected statTier(value: number): string {
    if (value >= 120) {
      return 'bg-accent';
    }
    if (value >= 90) {
      return 'bg-emerald-500';
    }
    if (value >= 60) {
      return 'bg-amber-500';
    }
    return 'bg-rose-500';
  }

  protected multiplierLabel(percent: number): string {
    switch (percent) {
      case 0:
        return '0';
      case 25:
        return '×¼';
      case 50:
        return '×½';
      case 200:
        return '×2';
      case 400:
        return '×4';
      default:
        return `×${percent / 100}`;
    }
  }

  protected formatHeight(decimetres: number): string {
    return `${this.formatMeasurement(decimetres / 10, 1)} m`;
  }

  protected formatWeight(hectograms: number): string {
    return `${this.formatMeasurement(hectograms / 10, 1)} kg`;
  }

  protected formatEggCycles(hatchCounter: number | null): string {
    return hatchCounter === null ? '—' : String(hatchCounter);
  }

  protected humanizeSlug(slug: string | null): string {
    if (!slug) {
      return '—';
    }

    return slug
      .split('-')
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join(' ');
  }

  /** Builds the localized list of conditions attached to one evolution edge. */
  protected conditionParts(step: EvolutionStep): readonly string[] {
    const parts: string[] = [];
    const translate = (key: string, params?: Record<string, unknown>) =>
      this.transloco.translate(key, params);

    if (step.trigger === 'level-up' && step.minLevel !== null) {
      parts.push(translate('pokedex.detail.evolutions.trigger.level', { level: step.minLevel }));
    } else if (step.trigger === 'use-item') {
      parts.push(
        translate('pokedex.detail.evolutions.trigger.item', {
          item: this.humanizeSlug(step.triggerItem),
        }),
      );
    } else if (step.trigger === 'trade') {
      parts.push(translate('pokedex.detail.evolutions.trigger.trade'));
      if (step.tradeSpecies) {
        parts.push(
          translate('pokedex.detail.evolutions.trigger.tradeSpecies', {
            species: step.tradeSpecies,
          }),
        );
      }
    } else if (step.trigger === 'shed') {
      parts.push(translate('pokedex.detail.evolutions.trigger.shed'));
    } else if (step.trigger && !['level-up', 'use-item', 'trade', 'shed'].includes(step.trigger)) {
      parts.push(this.humanizeSlug(step.trigger));
    }

    if (step.heldItem) {
      parts.push(
        translate('pokedex.detail.evolutions.trigger.heldItem', {
          item: this.humanizeSlug(step.heldItem),
        }),
      );
    }
    if (step.minHappiness !== null) {
      parts.push(
        translate('pokedex.detail.evolutions.trigger.happiness', { value: step.minHappiness }),
      );
    }
    if (step.minAffection !== null) {
      parts.push(
        translate('pokedex.detail.evolutions.trigger.affection', { value: step.minAffection }),
      );
    }
    if (step.minBeauty !== null) {
      parts.push(translate('pokedex.detail.evolutions.trigger.beauty', { value: step.minBeauty }));
    }
    if (step.timeOfDay) {
      parts.push(translate(`pokedex.detail.evolutions.trigger.time.${step.timeOfDay}`));
    }
    if (step.gender) {
      parts.push(translate(GENDER_KEYS[step.gender] ?? 'pokedex.detail.evolutions.trigger.other'));
    }
    if (step.knownMove) {
      parts.push(translate('pokedex.detail.evolutions.trigger.move', { move: step.knownMove }));
    }
    if (step.knownMoveType) {
      parts.push(
        translate('pokedex.detail.evolutions.trigger.moveType', { type: step.knownMoveType }),
      );
    }
    if (step.location) {
      parts.push(
        translate('pokedex.detail.evolutions.trigger.location', {
          location: this.humanizeSlug(step.location),
        }),
      );
    }
    if (step.partySpecies) {
      parts.push(
        translate('pokedex.detail.evolutions.trigger.partySpecies', { species: step.partySpecies }),
      );
    }
    if (step.partyType) {
      parts.push(
        translate('pokedex.detail.evolutions.trigger.partyType', { type: step.partyType }),
      );
    }
    if (step.needsOverworldRain) {
      parts.push(translate('pokedex.detail.evolutions.trigger.rain'));
    }
    if (step.turnUpsideDown) {
      parts.push(translate('pokedex.detail.evolutions.trigger.upsideDown'));
    }
    if (step.relativePhysicalStats !== null) {
      const key =
        step.relativePhysicalStats > 0
          ? 'atkGtDef'
          : step.relativePhysicalStats < 0
            ? 'atkLtDef'
            : 'atkEqDef';
      parts.push(translate(`pokedex.detail.evolutions.trigger.${key}`));
    }
    if (parts.length === 0 && step.conditionExpression) {
      const expression = step.conditionExpression.trim();
      if (!expression.startsWith('{')) {
        parts.push(expression);
      }
    }
    if (parts.length === 0) {
      parts.push(translate('pokedex.detail.evolutions.trigger.other'));
    }

    return parts;
  }

  private formatMeasurement(value: number, digits: number): string {
    return new Intl.NumberFormat(this.language(), {
      minimumFractionDigits: digits,
      maximumFractionDigits: digits,
    }).format(value);
  }

  private async load(id: number, language: string): Promise<void> {
    if (!Number.isInteger(id) || id <= 0) {
      this.species.set(null);
      this.state.set('notFound');
      return;
    }

    const requestId = ++this.requestId;
    this.species.set(null);
    this.state.set('loading');

    try {
      const species = await this.pokedex.pokemonDetail(id, this.selectedFormId, language, null);
      if (requestId !== this.requestId) {
        return;
      }

      this.species.set(species);
      this.state.set(species ? 'ready' : 'notFound');
    } catch {
      if (requestId !== this.requestId) {
        return;
      }

      this.state.set('error');
    }
  }
}
