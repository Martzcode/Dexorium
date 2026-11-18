import { Component, computed, inject, signal } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';

import { CatalogService } from '../catalog/catalog.service';
import type { NatureSummary } from '../catalog/catalog.models';

@Component({
  selector: 'app-competitive',
  imports: [TranslocoPipe],
  templateUrl: './competitive.page.html',
})
export class CompetitivePage {
  private readonly catalog = inject(CatalogService);
  private readonly transloco = inject(TranslocoService);

  protected readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  protected readonly natures = signal<readonly NatureSummary[]>([]);

  // Stat calculator state
  protected readonly base = signal(100);
  protected readonly iv = signal(31);
  protected readonly ev = signal(0);
  protected readonly level = signal(50);
  protected readonly nature = signal('hardy');

  protected readonly finalStat = computed(() => {
    const n = this.natures().find((x) => x.slug === this.nature()) ?? null;
    const mod = n ? (n.increasedStatSlug === n.decreasedStatSlug ? 1 : 1) : 1;
    // simplified: apply 1.1 / 0.9 based on increased/decreased
    const natureMod =
      n && n.increasedStatSlug && n.decreasedStatSlug && n.increasedStatSlug !== n.decreasedStatSlug
        ? 1.1
        : n && n.increasedStatSlug && !n.decreasedStatSlug
          ? 1
          : 1;
    const raw = Math.floor(
      ((2 * this.base() + this.iv() + Math.floor(this.ev() / 4)) * this.level()) / 100 + 5,
    );
    return Math.floor(raw * natureMod);
  });

  constructor() {
    this.catalog
      .listNatures(this.language())
      .then((n) => this.natures.set(n))
      .catch(() => {});
  }

  protected async reloadNatures(): Promise<void> {
    try {
      this.natures.set(await this.catalog.listNatures(this.language()));
    } catch {
      /* ignore */
    }
  }
}
