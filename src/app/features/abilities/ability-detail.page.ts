import { Component, computed, effect, inject, signal } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { ActivatedRoute, RouterLink } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';

import { AppIcon } from '../../shared/icon/app-icon';
import { formatDexNumber } from '../pokedex/pokedex-format';
import type { AbilityDetail, AbilityHolder } from '../catalog/catalog.models';
import { CatalogService } from '../catalog/catalog.service';

type LoadState = 'loading' | 'ready' | 'notFound' | 'error';

@Component({
  selector: 'app-ability-detail',
  imports: [RouterLink, TranslocoPipe, AppIcon],
  templateUrl: './ability-detail.page.html',
})
export class AbilityDetailPage {
  private readonly route = inject(ActivatedRoute);
  private readonly catalog = inject(CatalogService);
  private readonly transloco = inject(TranslocoService);

  protected readonly ability = signal<AbilityDetail | null>(null);
  protected readonly state = signal<LoadState>('loading');
  protected readonly formatDexNumber = formatDexNumber;

  private requestId = 0;

  private readonly parameters = toSignal(this.route.paramMap, {
    initialValue: this.route.snapshot.paramMap,
  });
  private readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  protected readonly holders = computed<readonly AbilityHolder[]>(
    () => this.ability()?.holders ?? [],
  );

  protected readonly effectText = computed(
    () => this.ability()?.effectLong ?? this.ability()?.description ?? null,
  );

  constructor() {
    effect(() => {
      const id = Number(this.parameters().get('id'));
      const language = this.language();
      void this.load(id, language);
    });
  }

  protected retry(): void {
    void this.load(Number(this.parameters().get('id')), this.language());
  }

  private async load(id: number, language: string): Promise<void> {
    if (!Number.isInteger(id) || id <= 0) {
      this.ability.set(null);
      this.state.set('notFound');
      return;
    }

    const requestId = ++this.requestId;
    this.ability.set(null);
    this.state.set('loading');

    try {
      const ability = await this.catalog.abilityDetail(id, language);
      if (requestId !== this.requestId) {
        return;
      }

      this.ability.set(ability);
      this.state.set(ability ? 'ready' : 'notFound');
    } catch {
      if (requestId !== this.requestId) {
        return;
      }

      this.state.set('error');
    }
  }
}
