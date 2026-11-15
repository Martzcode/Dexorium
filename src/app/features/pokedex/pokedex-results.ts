import { ScrollingModule, CdkVirtualScrollViewport } from '@angular/cdk/scrolling';
import { BreakpointObserver } from '@angular/cdk/layout';
import {
  Component,
  computed,
  effect,
  inject,
  input,
  output,
  untracked,
  viewChild,
} from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { RouterLink } from '@angular/router';
import { TranslocoPipe } from '@jsverse/transloco';
import { map } from 'rxjs';

import { PokemonSprite } from '../../shared/pokemon-sprite/pokemon-sprite';
import { CollectionService } from '../../core/collection/collection.service';
import { AppIcon } from '../../shared/icon/app-icon';
import { formatDexNumber, typeBadgeClass } from './pokedex-format';
import type { SortCriterion, SpeciesSortField, SpeciesSummary } from './pokedex.models';
import type { PokedexView } from './pokedex.store';

const GRID_ROW_HEIGHT = 156;
const LIST_ROW_HEIGHT = 76;
const TABLE_ROW_HEIGHT = 44;

@Component({
  selector: 'app-pokedex-results',
  imports: [ScrollingModule, RouterLink, TranslocoPipe, PokemonSprite, AppIcon],
  templateUrl: './pokedex-results.html',
})
export class PokedexResults {
  private readonly breakpoints = inject(BreakpointObserver);
  protected readonly collection = inject(CollectionService);

  readonly view = input.required<PokedexView>();
  readonly items = input.required<readonly SpeciesSummary[]>();
  readonly sort = input<readonly SortCriterion[]>([]);

  readonly sortField = output<SpeciesSortField>();

  private readonly viewport = viewChild(CdkVirtualScrollViewport);

  protected readonly formatDexNumber = formatDexNumber;
  protected readonly typeBadgeClass = typeBadgeClass;

  protected readonly gridRowHeight = GRID_ROW_HEIGHT;
  protected readonly listRowHeight = LIST_ROW_HEIGHT;
  protected readonly tableRowHeight = TABLE_ROW_HEIGHT;

  private readonly columns = toSignal(
    this.breakpoints
      .observe(['(min-width: 640px)', '(min-width: 1280px)', '(min-width: 1536px)'])
      .pipe(
        map((state) => {
          if (state.breakpoints['(min-width: 1536px)']) {
            return 4;
          }
          if (state.breakpoints['(min-width: 1280px)']) {
            return 3;
          }
          if (state.breakpoints['(min-width: 640px)']) {
            return 2;
          }
          return 1;
        }),
      ),
    { initialValue: 1 },
  );

  protected readonly gridColumns = computed(() => this.columns());

  protected readonly gridRows = computed(() => {
    const columns = this.columns();
    const rows: SpeciesSummary[][] = [];

    for (let index = 0; index < this.items().length; index += columns) {
      rows.push(this.items().slice(index, index + columns));
    }

    return rows;
  });

  constructor() {
    // A new page replaces the whole list: bring the virtual viewport back to
    // the first row so pagination never leaves the user mid-scroll.
    effect(() => {
      this.items();
      untracked(() => this.viewport()?.scrollToIndex(0));
    });
  }

  protected readonly singleRows = computed(() => this.items().map((species) => [species] as const));

  protected trackRow(index: number, row: readonly SpeciesSummary[]): number {
    return row[0]?.id ?? index;
  }

  protected sortIndicator(field: SpeciesSortField): string {
    const criterion = this.sort().find((entry) => entry.field === field);
    if (!criterion) {
      return '';
    }

    return criterion.direction === 'asc' ? '↑' : '↓';
  }

  protected onSort(field: SpeciesSortField): void {
    this.sortField.emit(field);
  }

  protected toggleFavorite(event: Event, speciesId: number): void {
    event.preventDefault();
    event.stopPropagation();
    void this.collection.toggle(speciesId);
  }

  protected favoriteLabel(speciesId: number): string {
    return this.collection.isFavorite(speciesId)
      ? 'pokedex.favorite.remove'
      : 'pokedex.favorite.add';
  }
}
