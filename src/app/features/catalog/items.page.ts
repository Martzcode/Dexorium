import { Component, DestroyRef, computed, inject, signal } from '@angular/core';
import { takeUntilDestroyed, toSignal } from '@angular/core/rxjs-interop';
import { RouterLink } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { Subject, debounceTime, distinctUntilChanged } from 'rxjs';

import { CatalogService } from '../catalog/catalog.service';
import type { ItemSummary } from '../catalog/catalog.models';

type LoadState = 'loading' | 'ready' | 'error';

@Component({
  selector: 'app-items',
  imports: [RouterLink, TranslocoPipe],
  templateUrl: './items.page.html',
})
export class ItemsPage {
  private readonly catalog = inject(CatalogService);
  private readonly transloco = inject(TranslocoService);
  private readonly destroyRef = inject(DestroyRef);

  protected readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  protected readonly state = signal<LoadState>('loading');
  protected readonly items = signal<readonly ItemSummary[]>([]);
  protected readonly total = signal(0);

  protected readonly search = signal('');
  protected readonly category = signal<string | null>(null);

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

  protected async reload(): Promise<void> {
    const id = ++this.requestId;
    this.state.set('loading');
    try {
      const items = await this.catalog.listItems(this.search(), this.category(), this.language());
      if (id !== this.requestId) {
        return;
      }
      this.items.set(items);
      this.total.set(items.length);
      this.state.set('ready');
    } catch {
      if (id !== this.requestId) {
        return;
      }
      this.state.set('error');
    }
  }
}
