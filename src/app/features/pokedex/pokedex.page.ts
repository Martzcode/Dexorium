import { Component, computed, effect, inject } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';

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

  protected readonly resultCount = computed(() => {
    const shown = this.store.items().length;
    const total = this.store.total();
    return shown < total ? `${shown} / ${total}` : String(total);
  });

  private readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  constructor() {
    effect(() => {
      this.store.setLanguage(this.language());
    });
  }

  protected onSearch(event: Event): void {
    this.store.setSearch((event.target as HTMLInputElement).value);
  }

  protected retry(): void {
    this.store.retry();
  }

  protected loadMore(): void {
    void this.store.loadMore();
  }

  protected sortField(field: Parameters<PokedexStore['cycleSort']>[0]): void {
    this.store.cycleSort(field);
  }
}
