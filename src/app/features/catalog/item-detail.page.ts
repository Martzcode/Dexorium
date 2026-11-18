import { Component, DestroyRef, inject, signal } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { toSignal } from '@angular/core/rxjs-interop';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';

import { CatalogService } from './catalog.service';
import type { ItemDetail } from './catalog.models';

@Component({
  selector: 'app-item-detail',
  imports: [TranslocoPipe],
  templateUrl: './item-detail.page.html',
})
export class ItemDetailPage {
  private readonly route = inject(ActivatedRoute);
  private readonly catalog = inject(CatalogService);
  private readonly transloco = inject(TranslocoService);
  private readonly destroyRef = inject(DestroyRef);

  protected readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  protected readonly item = signal<ItemDetail | null>(null);
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
      const item = await this.catalog.itemDetail(id, this.language());
      this.item.set(item);
      this.state.set('ready');
    } catch {
      this.state.set('error');
    }
  }
}
