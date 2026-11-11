import { Component, computed, effect, inject, signal } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { ActivatedRoute, RouterLink } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';

import { AppIcon } from '../../shared/icon/app-icon';
import { formatDexNumber, typeBadgeClass } from './pokedex-format';
import type { SpeciesDetail } from './pokedex.models';
import { PokedexService } from './pokedex.service';

type LoadState = 'loading' | 'ready' | 'notFound' | 'error';

interface StatRow {
  readonly key: string;
  readonly labelKey: string;
  readonly value: number;
}

@Component({
  selector: 'app-species-detail',
  imports: [RouterLink, TranslocoPipe, AppIcon],
  templateUrl: './species-detail.page.html',
})
export class SpeciesDetailPage {
  private readonly route = inject(ActivatedRoute);
  private readonly pokedex = inject(PokedexService);
  private readonly transloco = inject(TranslocoService);

  protected readonly species = signal<SpeciesDetail | null>(null);
  protected readonly state = signal<LoadState>('loading');

  protected readonly formatDexNumber = formatDexNumber;
  protected readonly typeBadgeClass = typeBadgeClass;

  private readonly parameters = toSignal(this.route.paramMap, {
    initialValue: this.route.snapshot.paramMap,
  });
  private readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  protected readonly statRows = computed<readonly StatRow[]>(() => {
    const stats = this.species()?.stats;
    if (!stats) {
      return [];
    }

    return [
      { key: 'hp', labelKey: 'pokedex.stats.hp', value: stats.hp },
      { key: 'attack', labelKey: 'pokedex.stats.attack', value: stats.attack },
      { key: 'defense', labelKey: 'pokedex.stats.defense', value: stats.defense },
      { key: 'spAtk', labelKey: 'pokedex.stats.spAtk', value: stats.spAtk },
      { key: 'spDef', labelKey: 'pokedex.stats.spDef', value: stats.spDef },
      { key: 'speed', labelKey: 'pokedex.stats.speed', value: stats.speed },
    ];
  });

  private requestId = 0;

  constructor() {
    effect(() => {
      const id = Number(this.parameters().get('id'));
      const language = this.language();

      void this.load(id, language);
    });
  }

  protected retry(): void {
    void this.load(Number(this.parameters().get('id')), this.language());
  }

  protected statPercent(value: number): number {
    return Math.min(100, Math.round((value / 180) * 100));
  }

  protected formatHeight(decimetres: number): string {
    return `${this.formatMeasurement(decimetres / 10, 1)} m`;
  }

  protected formatWeight(hectograms: number): string {
    return `${this.formatMeasurement(hectograms / 10, 1)} kg`;
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
      const species = await this.pokedex.detail(id, language);
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
