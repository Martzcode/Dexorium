import { Component, DestroyRef, inject, signal } from '@angular/core';
import { RouterLink } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { Subject, debounceTime, distinctUntilChanged } from 'rxjs';
import { takeUntilDestroyed, toSignal } from '@angular/core/rxjs-interop';

import { CatalogService } from './catalog.service';
import type { LocationSummary } from './catalog.models';

type LoadState = 'loading' | 'ready' | 'error';

@Component({
  selector: 'app-locations',
  imports: [RouterLink, TranslocoPipe],
  templateUrl: './locations.page.html',
})
export class LocationsPage {
  private readonly catalog = inject(CatalogService);
  private readonly transloco = inject(TranslocoService);
  private readonly destroyRef = inject(DestroyRef);

  protected readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  protected readonly state = signal<LoadState>('loading');
  protected readonly locations = signal<readonly LocationSummary[]>([]);

  protected readonly search = signal('');

  private readonly searchInput = new Subject<string>();

  constructor() {
    this.searchInput
      .pipe(debounceTime(220), distinctUntilChanged(), takeUntilDestroyed(this.destroyRef))
      .subscribe((value: string) => {
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

  protected async reload(): Promise<void> {
    this.state.set('loading');
    try {
      const locations = await this.catalog.listLocations(this.search(), this.language());
      this.locations.set(locations);
      this.state.set('ready');
    } catch {
      this.state.set('error');
    }
  }
}
