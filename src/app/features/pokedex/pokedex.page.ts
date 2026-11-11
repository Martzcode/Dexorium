import { Component, effect, inject, signal } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { RouterLink } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';

import { AppIcon } from '../../shared/icon/app-icon';
import { formatDexNumber, typeBadgeClass } from './pokedex-format';
import type { SpeciesSummary } from './pokedex.models';
import { PokedexService } from './pokedex.service';

type LoadState = 'loading' | 'ready' | 'error';

@Component({
  selector: 'app-pokedex',
  imports: [RouterLink, TranslocoPipe, AppIcon],
  templateUrl: './pokedex.page.html',
})
export class PokedexPage {
  private readonly pokedex = inject(PokedexService);
  private readonly transloco = inject(TranslocoService);

  protected readonly query = signal('');
  protected readonly results = signal<readonly SpeciesSummary[]>([]);
  protected readonly state = signal<LoadState>('loading');
  protected readonly placeholders = [1, 2, 3, 4, 5, 6];

  protected readonly formatDexNumber = formatDexNumber;
  protected readonly typeBadgeClass = typeBadgeClass;

  private readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  private requestId = 0;

  constructor() {
    effect(() => {
      const query = this.query();
      const language = this.language();

      void this.load(query, language);
    });
  }

  protected retry(): void {
    void this.load(this.query(), this.language());
  }

  private async load(query: string, language: string): Promise<void> {
    const requestId = ++this.requestId;
    this.state.set('loading');

    try {
      const results = await this.pokedex.search(query, language);
      if (requestId !== this.requestId) {
        return;
      }

      this.results.set(results);
      this.state.set('ready');
    } catch {
      if (requestId !== this.requestId) {
        return;
      }

      this.state.set('error');
    }
  }
}
