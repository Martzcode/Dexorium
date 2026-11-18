import { Component, DestroyRef, inject, signal } from '@angular/core';
import { ActivatedRoute, RouterLink } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { takeUntilDestroyed, toSignal } from '@angular/core/rxjs-interop';

import { CatalogService } from './catalog.service';
import type { LocationDetail } from './catalog.models';

@Component({
  selector: 'app-location-detail',
  imports: [RouterLink, TranslocoPipe],
  templateUrl: './location-detail.page.html',
})
export class LocationDetailPage {
  private readonly route = inject(ActivatedRoute);
  private readonly catalog = inject(CatalogService);
  private readonly transloco = inject(TranslocoService);
  private readonly destroyRef = inject(DestroyRef);

  protected readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  protected readonly detail = signal<LocationDetail | null>(null);
  protected readonly state = signal<'loading' | 'ready' | 'error'>('loading');

  constructor() {
    this.transloco.langChanges$
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(() => this.load());
    this.load();
  }

  private async load(): Promise<void> {
    const id = Number(this.route.snapshot.paramMap.get('id'));
    this.state.set('loading');
    try {
      const detail = await this.catalog.locationDetail(id, this.language());
      this.detail.set(detail);
      this.state.set('ready');
    } catch {
      this.state.set('error');
    }
  }
}
