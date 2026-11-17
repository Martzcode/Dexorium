import { Component, computed, effect, inject, untracked } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';

import { CollectionService } from '../../core/collection/collection.service';
import { AppIcon } from '../../shared/icon/app-icon';
import { PokedexFilters } from './pokedex-filters';
import { PokedexResults } from './pokedex-results';
import { PokedexStore, type PokedexView } from './pokedex.store';

@Component({
  selector: 'app-pokedex',
  imports: [TranslocoPipe, AppIcon, PokedexFilters, PokedexResults],
  templateUrl: './pokedex.page.html',
})
export class PokedexPage {
  private readonly transloco = inject(TranslocoService);
  private readonly collection = inject(CollectionService);

  protected readonly store = inject(PokedexStore);
  protected readonly placeholders = [1, 2, 3, 4, 5, 6];
  protected readonly views: readonly {
    key: PokedexView;
    icon: 'layout-grid' | 'list' | 'table';
  }[] = [
    { key: 'grid', icon: 'layout-grid' },
    { key: 'list', icon: 'list' },
    { key: 'table', icon: 'table' },
  ];

  protected readonly resultRange = computed(() => {
    const total = this.store.total();
    const shown = this.store.items().length;
    const from = shown === 0 ? 0 : (this.store.page() - 1) * this.store.pageSize + 1;

    return { from, to: shown === 0 ? 0 : from + shown - 1, total };
  });

  private readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  constructor() {
    effect(() => {
      this.store.setLanguage(this.language());
    });

    let seenRevision = -1;
    effect(() => {
      const revision = this.collection.revision();
      const changed = seenRevision !== -1 && seenRevision !== revision;
      seenRevision = revision;

      // Track the revision only: reading `favoritesOnly` untracked avoids a
      // duplicate reload when the filter itself toggles.
      if (changed && untracked(() => this.store.favoritesOnly())) {
        this.store.refresh();
      }
    });
  }

  protected onSearch(event: Event): void {
    this.store.setSearch((event.target as HTMLInputElement).value);
  }

  protected retry(): void {
    this.store.retry();
  }

  protected exportCsv(): void {
    const items = this.store.items();
    if (items.length === 0) {
      return;
    }

    const header = [
      'nationalDexNumber',
      'name',
      'types',
      'bst',
      'hp',
      'attack',
      'defense',
      'spAtk',
      'spDef',
      'speed',
    ];
    const lines = [header.join(',')];
    for (const item of items) {
      const stats = item.stats;
      lines.push(
        [
          item.nationalDexNumber,
          `"${item.name.replace(/"/g, '""')}"`,
          `"${item.types.join(';')}"`,
          item.bst,
          stats.hp,
          stats.attack,
          stats.defense,
          stats.spAtk,
          stats.spDef,
          stats.speed,
        ].join(','),
      );
    }

    const blob = new Blob([lines.join('\n')], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = 'dexorium-pokedex.csv';
    anchor.click();
    URL.revokeObjectURL(url);
  }

  protected formatNumber(value: number): string {
    return new Intl.NumberFormat(this.language()).format(value);
  }

  protected sortField(field: Parameters<PokedexStore['cycleSort']>[0]): void {
    this.store.cycleSort(field);
  }
}
