import { Component, computed, effect, inject, signal } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';

import { AppIcon } from '../../shared/icon/app-icon';
import { typeBadgeClass } from '../pokedex/pokedex-format';
import type { TypeMatrix } from './tools.models';
import { ToolsService } from './tools.service';

type LoadState = 'loading' | 'ready' | 'error';

interface LegendEntry {
  readonly key: string;
  readonly labelKey: string;
  readonly className: string;
}

const LEGEND: readonly LegendEntry[] = [
  { key: '0', labelKey: 'tools.types.legend.immune', className: 'bg-slate-700' },
  { key: '25', labelKey: 'tools.types.legend.quarter', className: 'bg-emerald-600/80' },
  { key: '50', labelKey: 'tools.types.legend.half', className: 'bg-emerald-400/70' },
  { key: '100', labelKey: 'tools.types.legend.neutral', className: 'bg-panel-muted' },
  { key: '200', labelKey: 'tools.types.legend.double', className: 'bg-amber-400/80' },
  { key: '400', labelKey: 'tools.types.legend.quadruple', className: 'bg-rose-500/85' },
];

@Component({
  selector: 'app-type-matrix',
  imports: [TranslocoPipe, AppIcon],
  templateUrl: './type-matrix.page.html',
})
export class TypeMatrixPage {
  private readonly tools = inject(ToolsService);
  private readonly transloco = inject(TranslocoService);

  protected readonly state = signal<LoadState>('loading');
  protected readonly matrix = signal<TypeMatrix | null>(null);
  protected readonly generation = signal<number | null>(null);
  protected readonly legend = LEGEND;
  protected readonly typeBadgeClass = typeBadgeClass;

  private readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  protected readonly generations = computed(() => this.matrix()?.generations ?? []);

  constructor() {
    effect(() => {
      void this.load(this.language(), this.generation());
    });
  }

  protected onGenerationChange(event: Event): void {
    const value = Number((event.target as HTMLSelectElement).value);
    this.generation.set(Number.isFinite(value) && value > 0 ? value : null);
  }

  protected retry(): void {
    void this.load(this.language(), this.generation());
  }

  protected multiplier(row: number, column: number): number {
    return this.matrix()?.multipliers[row]?.[column] ?? 100;
  }

  protected cellClass(value: number): string {
    switch (value) {
      case 0:
        return 'bg-slate-700 text-white';
      case 25:
        return 'bg-emerald-600/80 text-white';
      case 50:
        return 'bg-emerald-400/70 text-emerald-950';
      case 200:
        return 'bg-amber-400/80 text-amber-950';
      case 400:
        return 'bg-rose-500/85 text-white';
      default:
        return 'bg-panel-muted/60 text-ink-muted';
    }
  }

  protected cellLabel(value: number): string {
    switch (value) {
      case 0:
        return '0×';
      case 25:
        return '¼×';
      case 50:
        return '½×';
      case 200:
        return '2×';
      case 400:
        return '4×';
      default:
        return '1×';
    }
  }

  private async load(language: string, generation: number | null): Promise<void> {
    this.state.set('loading');

    try {
      const matrix = await this.tools.typeMatrix(generation, language);
      this.matrix.set(matrix);
      this.state.set('ready');
    } catch {
      this.state.set('error');
    }
  }
}
