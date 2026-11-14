import { Component, DestroyRef, computed, inject, signal } from '@angular/core';
import { takeUntilDestroyed, toSignal } from '@angular/core/rxjs-interop';
import { RouterLink } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { Subject, debounceTime, distinctUntilChanged } from 'rxjs';

import { AppIcon } from '../../shared/icon/app-icon';
import {
  ABILITY_PAGE_SIZE,
  defaultAbilityQuery,
  type AbilityListQuery,
  type AbilitySummary,
} from '../catalog/catalog.models';
import { CatalogService } from '../catalog/catalog.service';

type LoadState = 'loading' | 'ready' | 'error';

const GENERATIONS = [1, 2, 3, 4, 5, 6, 7, 8, 9] as const;

@Component({
  selector: 'app-abilities',
  imports: [RouterLink, TranslocoPipe, AppIcon],
  templateUrl: './abilities.page.html',
})
export class AbilitiesPage {
  private readonly catalog = inject(CatalogService);
  private readonly transloco = inject(TranslocoService);
  private readonly destroyRef = inject(DestroyRef);

  protected readonly generations = GENERATIONS;

  protected readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  protected readonly state = signal<LoadState>('loading');
  protected readonly items = signal<readonly AbilitySummary[]>([]);
  protected readonly total = signal(0);
  protected readonly loadingMore = signal(false);

  protected readonly search = signal('');
  protected readonly selectedGenerations = signal<readonly number[]>([]);

  protected readonly hasMore = computed(() => this.items().length < this.total());
  protected readonly activeFilterCount = computed(() => this.selectedGenerations().length);

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

  protected toggleGeneration(generation: number): void {
    this.selectedGenerations.update((generations) =>
      generations.includes(generation)
        ? generations.filter((entry) => entry !== generation)
        : [...generations, generation],
    );
    this.reload();
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

  protected reset(): void {
    this.search.set('');
    this.selectedGenerations.set([]);
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

  private reload(): void {
    void this.load(0, false);
  }

  private query(offset: number): AbilityListQuery {
    return {
      ...defaultAbilityQuery(this.language()),
      search: this.search().trim() || null,
      generations: this.selectedGenerations(),
      offset,
      limit: ABILITY_PAGE_SIZE,
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
      const page = await this.catalog.listAbilities(this.query(offset));
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
