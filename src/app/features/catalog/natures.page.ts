import { Component, DestroyRef, inject, signal } from '@angular/core';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';
import { takeUntilDestroyed, toSignal } from '@angular/core/rxjs-interop';

import { CatalogService } from './catalog.service';
import type { NatureSummary } from './catalog.models';

@Component({
  selector: 'app-natures',
  imports: [TranslocoPipe],
  templateUrl: './natures.page.html',
})
export class NaturesPage {
  private readonly catalog = inject(CatalogService);
  private readonly transloco = inject(TranslocoService);
  private readonly destroyRef = inject(DestroyRef);

  protected readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  protected readonly state = signal<'loading' | 'ready' | 'error'>('loading');
  protected readonly natures = signal<readonly NatureSummary[]>([]);

  constructor() {
    this.transloco.langChanges$.pipe(takeUntilDestroyed(this.destroyRef)).subscribe(() => this.reload());
    this.reload();
  }

  protected async reload(): Promise<void> {
    this.state.set('loading');
    try {
      const natures = await this.catalog.listNatures(this.language());
      this.natures.set(natures);
      this.state.set('ready');
    } catch {
      this.state.set('error');
    }
  }
}
