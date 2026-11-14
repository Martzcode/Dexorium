import { Component, DestroyRef, computed, inject, signal } from '@angular/core';
import { takeUntilDestroyed, toSignal } from '@angular/core/rxjs-interop';
import { RouterLink } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { Subject, debounceTime, distinctUntilChanged } from 'rxjs';

import { AppIcon, type IconName } from '../../shared/icon/app-icon';
import { typeBadgeClass } from '../pokedex/pokedex-format';
import {
  MOVE_PAGE_SIZE,
  defaultMoveQuery,
  type MoveListQuery,
  type MoveSortField,
  type MoveSummary,
  type SortDirection,
} from '../catalog/catalog.models';
import { CatalogService } from '../catalog/catalog.service';

type LoadState = 'loading' | 'ready' | 'error';

const TYPES = [
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

const GENERATIONS = [1, 2, 3, 4, 5, 6, 7, 8, 9] as const;
const DAMAGE_CLASSES = ['physical', 'special', 'status'] as const;

@Component({
  selector: 'app-moves',
  imports: [RouterLink, TranslocoPipe, AppIcon],
  templateUrl: './moves.page.html',
})
export class MovesPage {
  private readonly catalog = inject(CatalogService);
  private readonly transloco = inject(TranslocoService);
  private readonly destroyRef = inject(DestroyRef);

  protected readonly types = TYPES;
  protected readonly generations = GENERATIONS;
  protected readonly damageClasses = DAMAGE_CLASSES;
  protected readonly typeBadgeClass = typeBadgeClass;
  protected readonly pageSize = MOVE_PAGE_SIZE;

  protected readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  protected readonly state = signal<LoadState>('loading');
  protected readonly items = signal<readonly MoveSummary[]>([]);
  protected readonly total = signal(0);
  protected readonly loadingMore = signal(false);

  protected readonly search = signal('');
  protected readonly selectedTypes = signal<readonly string[]>([]);
  protected readonly selectedClasses = signal<readonly string[]>([]);
  protected readonly selectedGenerations = signal<readonly number[]>([]);
  protected readonly minPower = signal<number | null>(null);
  protected readonly maxPower = signal<number | null>(null);
  protected readonly sortField = signal<MoveSortField>('id');
  protected readonly sortDirection = signal<SortDirection>('asc');

  protected readonly hasMore = computed(() => this.items().length < this.total());
  protected readonly activeFilterCount = computed(
    () =>
      this.selectedTypes().length +
      this.selectedClasses().length +
      this.selectedGenerations().length +
      (this.minPower() === null ? 0 : 1) +
      (this.maxPower() === null ? 0 : 1),
  );

  private readonly searchInput = new Subject<string>();
  private requestId = 0;

  constructor() {
    this.searchInput
      .pipe(debounceTime(220), distinctUntilChanged(), takeUntilDestroyed(this.destroyRef))
      .subscribe((value) => {
        this.search.set(value);
        this.reload();
      });

    this.transloco.langChanges$
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(() => this.reload());

    this.reload();
  }

  protected onSearch(event: Event): void {
    this.searchInput.next((event.target as HTMLInputElement).value);
  }

  protected clearSearch(input: HTMLInputElement): void {
    input.value = '';
    this.searchInput.next('');
  }

  protected toggleType(type: string): void {
    this.selectedTypes.update((types) =>
      types.includes(type) ? types.filter((entry) => entry !== type) : [...types, type],
    );
    this.reload();
  }

  protected toggleClass(damageClass: string): void {
    this.selectedClasses.update((classes) =>
      classes.includes(damageClass)
        ? classes.filter((entry) => entry !== damageClass)
        : [...classes, damageClass],
    );
    this.reload();
  }

  protected toggleGeneration(generation: number): void {
    this.selectedGenerations.update((generations) =>
      generations.includes(generation)
        ? generations.filter((entry) => entry !== generation)
        : [...generations, generation],
    );
    this.reload();
  }

  protected setPower(bound: 'min' | 'max', event: Event): void {
    const raw = (event.target as HTMLInputElement).value.trim();
    const value = raw.length === 0 ? null : Number(raw);
    const normalized =
      value === null || !Number.isFinite(value)
        ? null
        : Math.min(300, Math.max(0, Math.round(value)));

    if (bound === 'min') {
      this.minPower.set(normalized);
    } else {
      this.maxPower.set(normalized);
    }
    this.reload();
  }

  protected setSort(field: MoveSortField, direction: SortDirection): void {
    this.sortField.set(field);
    this.sortDirection.set(direction);
    this.reload();
  }

  protected onSortChange(event: Event): void {
    const [field, direction] = (event.target as HTMLSelectElement).value.split(':') as [
      MoveSortField,
      SortDirection,
    ];
    this.setSort(field, direction);
  }

  protected reset(): void {
    this.search.set('');
    this.selectedTypes.set([]);
    this.selectedClasses.set([]);
    this.selectedGenerations.set([]);
    this.minPower.set(null);
    this.maxPower.set(null);
    this.sortField.set('id');
    this.sortDirection.set('asc');
    this.reload();
  }

  protected loadMore(): void {
    if (this.loadingMore() || !this.hasMore() || this.state() !== 'ready') {
      return;
    }

    void this.load(this.items().length, true);
  }

  protected retry(): void {
    this.reload();
  }

  protected isTypeSelected(type: string): boolean {
    return this.selectedTypes().includes(type);
  }

  protected isClassSelected(damageClass: string): boolean {
    return this.selectedClasses().includes(damageClass);
  }

  protected isGenerationSelected(generation: number): boolean {
    return this.selectedGenerations().includes(generation);
  }

  protected chipClass(active: boolean): string {
    const base =
      'rounded-full border px-2.5 py-1 font-mono text-[0.66rem] font-semibold uppercase tracking-[0.05em] transition-colors';
    return active
      ? `${base} border-accent bg-accent/15 text-ink`
      : `${base} border-line bg-panel-muted/40 text-ink-soft hover:border-line-strong`;
  }

  protected damageClassIcon(damageClass: string): IconName {
    switch (damageClass) {
      case 'physical':
        return 'swords';
      case 'special':
        return 'brain';
      default:
        return 'settings';
    }
  }

  private reload(): void {
    void this.load(0, false);
  }

  private query(offset: number): MoveListQuery {
    return {
      ...defaultMoveQuery(this.language()),
      search: this.search().trim() || null,
      types: this.selectedTypes(),
      damageClasses: this.selectedClasses(),
      generations: this.selectedGenerations(),
      minPower: this.minPower(),
      maxPower: this.maxPower(),
      sort: [{ field: this.sortField(), direction: this.sortDirection() }],
      offset,
      limit: MOVE_PAGE_SIZE,
    };
  }

  private async load(offset: number, append: boolean): Promise<void> {
    const requestId = ++this.requestId;

    if (append) {
      this.loadingMore.set(true);
    } else {
      this.state.set('loading');
    }

    try {
      const page = await this.catalog.listMoves(this.query(offset));
      if (requestId !== this.requestId) {
        return;
      }

      this.items.update((current) => (append ? [...current, ...page.items] : page.items));
      this.total.set(page.total);
      this.state.set('ready');
    } catch {
      if (requestId !== this.requestId) {
        return;
      }

      this.state.set('error');
    } finally {
      if (requestId === this.requestId) {
        this.loadingMore.set(false);
      }
    }
  }
}
