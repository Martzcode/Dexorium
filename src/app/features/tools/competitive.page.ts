import { Component, computed, inject, signal } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';

import { CatalogService } from '../catalog/catalog.service';
import type { NatureSummary } from '../catalog/catalog.models';

type StatKey = 'hp' | 'attack' | 'defense' | 'spAtk' | 'spDef' | 'speed';

interface StatInput {
  readonly key: StatKey;
  readonly labelKey: string;
  readonly base: number;
  readonly iv: number;
  readonly ev: number;
}

const STAT_LABELS: Readonly<Record<StatKey, string>> = {
  hp: 'pokedex.stats.hp',
  attack: 'pokedex.stats.attack',
  defense: 'pokedex.stats.defense',
  spAtk: 'pokedex.stats.spAtk',
  spDef: 'pokedex.stats.spDef',
  speed: 'pokedex.stats.speed',
};

const NATURE_STATS: Readonly<Record<string, StatKey>> = {
  hp: 'hp',
  attack: 'attack',
  defense: 'defense',
  sp_atk: 'spAtk',
  sp_def: 'spDef',
  speed: 'speed',
};

const DEFAULT_STATS: readonly StatInput[] = [
  { key: 'hp', labelKey: STAT_LABELS.hp, base: 100, iv: 31, ev: 0 },
  { key: 'attack', labelKey: STAT_LABELS.attack, base: 100, iv: 31, ev: 0 },
  { key: 'defense', labelKey: STAT_LABELS.defense, base: 100, iv: 31, ev: 0 },
  { key: 'spAtk', labelKey: STAT_LABELS.spAtk, base: 100, iv: 31, ev: 0 },
  { key: 'spDef', labelKey: STAT_LABELS.spDef, base: 100, iv: 31, ev: 0 },
  { key: 'speed', labelKey: STAT_LABELS.speed, base: 100, iv: 31, ev: 0 },
];

@Component({
  selector: 'app-competitive',
  imports: [TranslocoPipe],
  templateUrl: './competitive.page.html',
})
export class CompetitivePage {
  private readonly catalog = inject(CatalogService);
  private readonly transloco = inject(TranslocoService);

  protected readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  protected readonly natures = signal<readonly NatureSummary[]>([]);
  protected readonly stats = signal<readonly StatInput[]>(DEFAULT_STATS);
  protected readonly level = signal(50);
  protected readonly nature = signal('hardy');

  protected readonly selectedNature = computed(
    () => this.natures().find((nature) => nature.slug === this.nature()) ?? null,
  );

  protected natureDetail(nature: NatureSummary | null = null): string {
    const selected = nature ?? this.selectedNature();
    if (!selected || (!selected.increasedStatSlug && !selected.decreasedStatSlug)) {
      return this.transloco.translate('tools.neutralNature');
    }
    const increased = selected.increasedStatSlug ? this.statLabel(selected.increasedStatSlug) : '-';
    const decreased = selected.decreasedStatSlug ? this.statLabel(selected.decreasedStatSlug) : '-';
    return `${this.transloco.translate('tools.increased')} ${increased} · ${this.transloco.translate('tools.decreased')} ${decreased}`;
  }

  protected readonly results = computed(() => {
    const level = Math.max(1, Math.min(100, this.level()));
    const nature = this.selectedNature();
    const increased = nature?.increasedStatSlug ? NATURE_STATS[nature.increasedStatSlug] : null;
    const decreased = nature?.decreasedStatSlug ? NATURE_STATS[nature.decreasedStatSlug] : null;

    return this.stats().map((stat) => {
      const raw = Math.floor(((2 * stat.base + stat.iv + Math.floor(stat.ev / 4)) * level) / 100);
      let value: number;
      if (stat.key === 'hp') {
        value = raw + level + 10;
      } else {
        value = raw + 5;
        if (stat.key === increased) {
          value = Math.floor(value * 1.1);
        } else if (stat.key === decreased) {
          value = Math.floor(value * 0.9);
        }
      }
      return { ...stat, value };
    });
  });

  constructor() {
    this.transloco.langChanges$.subscribe(() => this.reloadNatures());
    this.reloadNatures();
  }

  protected readonly totalEv = computed(() => this.stats().reduce((sum, stat) => sum + stat.ev, 0));

  protected evMax(index: number): number {
    const others = this.stats().reduce(
      (sum, stat, position) => sum + (position === index ? 0 : stat.ev),
      0,
    );
    return Math.max(0, Math.min(252, 510 - others));
  }

  protected setField(index: number, field: 'base' | 'iv' | 'ev', raw: string): void {
    const parsed = Math.max(0, Math.trunc(Number(raw) || 0));
    const value =
      field === 'ev'
        ? Math.min(parsed, this.evMax(index))
        : field === 'iv'
          ? Math.min(parsed, 31)
          : Math.min(parsed, 255);
    this.stats.update((stats) =>
      stats.map((stat, position) => (position === index ? { ...stat, [field]: value } : stat)),
    );
  }

  protected onLevel(raw: string): void {
    this.level.set(Math.max(1, Math.min(100, Math.trunc(Number(raw) || 50))));
  }

  private statLabel(slug: string): string {
    const key = NATURE_STATS[slug];
    return key ? this.transloco.translate(STAT_LABELS[key]) : slug;
  }

  private async reloadNatures(): Promise<void> {
    try {
      this.natures.set(await this.catalog.listNatures(this.language()));
    } catch {
      /* ignore */
    }
  }
}
