import { Component, computed, effect, inject, signal } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { ActivatedRoute, RouterLink } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';

import { AppIcon } from '../../shared/icon/app-icon';
import { formatDexNumber, typeBadgeClass } from '../pokedex/pokedex-format';
import type { MoveDetail, MoveLearner } from '../catalog/catalog.models';
import { CatalogService } from '../catalog/catalog.service';

type LoadState = 'loading' | 'ready' | 'notFound' | 'error';

@Component({
  selector: 'app-move-detail',
  imports: [RouterLink, TranslocoPipe, AppIcon],
  templateUrl: './move-detail.page.html',
})
export class MoveDetailPage {
  private readonly route = inject(ActivatedRoute);
  private readonly catalog = inject(CatalogService);
  private readonly transloco = inject(TranslocoService);

  protected readonly move = signal<MoveDetail | null>(null);
  protected readonly state = signal<LoadState>('loading');
  protected readonly formatDexNumber = formatDexNumber;
  protected readonly typeBadgeClass = typeBadgeClass;

  private requestId = 0;

  private readonly parameters = toSignal(this.route.paramMap, {
    initialValue: this.route.snapshot.paramMap,
  });
  private readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  protected readonly learners = computed<readonly MoveLearner[]>(() => this.move()?.learners ?? []);

  protected readonly statCards = computed(() => {
    const move = this.move();
    if (!move) {
      return [];
    }

    return [
      { key: 'power', labelKey: 'moves.detail.power', value: move.power ?? null },
      {
        key: 'accuracy',
        labelKey: 'moves.detail.accuracy',
        value: move.accuracy === null ? null : `${move.accuracy} %`,
      },
      { key: 'pp', labelKey: 'moves.detail.pp', value: move.pp ?? null },
      { key: 'priority', labelKey: 'moves.detail.priority', value: move.priority },
      {
        key: 'effectChance',
        labelKey: 'moves.detail.effectChance',
        value: move.effectChance === null ? null : `${move.effectChance} %`,
      },
      {
        key: 'generation',
        labelKey: 'moves.detail.generation',
        value: move.generationId,
      },
    ];
  });

  protected readonly effectText = computed(
    () => this.move()?.effectLong ?? this.move()?.description ?? this.move()?.effectShort ?? null,
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

  protected humanizeSlug(slug: string | null): string {
    if (!slug) {
      return '—';
    }

    return slug
      .split('-')
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join(' ');
  }

  private async load(id: number, language: string): Promise<void> {
    if (!Number.isInteger(id) || id <= 0) {
      this.move.set(null);
      this.state.set('notFound');
      return;
    }

    const requestId = ++this.requestId;
    this.move.set(null);
    this.state.set('loading');

    try {
      const move = await this.catalog.moveDetail(id, language);
      if (requestId !== this.requestId) {
        return;
      }

      this.move.set(move);
      this.state.set(move ? 'ready' : 'notFound');
    } catch {
      if (requestId !== this.requestId) {
        return;
      }

      this.state.set('error');
    }
  }
}
