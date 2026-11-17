import { Component, computed, effect, inject, input, signal } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { RouterLink } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';

import { LanguageService } from '../../core/i18n/language.service';
import { AppIcon } from '../../shared/icon/app-icon';
import { typeBadgeClass } from './pokedex-format';
import type { LearnsetEntry, LearnsetPage } from '../catalog/catalog.models';
import { CatalogService } from '../catalog/catalog.service';

type LoadState = 'loading' | 'ready' | 'error';

const METHOD_ORDER = ['level-up', 'machine', 'egg', 'tutor'] as const;

const VERSION_GROUP_NAMES_FR: Readonly<Record<string, string>> = {
  'red-green-japan': 'Rouge/Vert (Japon)',
  'blue-japan': 'Bleu (Japon)',
  'red-blue': 'Rouge/Bleu',
  yellow: 'Jaune',
  'gold-silver': 'Or/Argent',
  crystal: 'Cristal',
  'ruby-sapphire': 'Rubis/Saphir',
  emerald: 'Émeraude',
  colosseum: 'Colosseum',
  xd: 'XD',
  'firered-leafgreen': 'Rouge Feu/Vert Feuille',
  'diamond-pearl': 'Diamant/Perle',
  platinum: 'Platine',
  'heartgold-soulsilver': 'Or HeartGold/Argent SoulSilver',
  'black-white': 'Noir/Blanc',
  'black-2-white-2': 'Noir 2/Blanc 2',
  'x-y': 'X/Y',
  'omega-ruby-alpha-sapphire': 'Rubis Oméga/Saphir Alpha',
  'sun-moon': 'Soleil/Lune',
  'ultra-sun-ultra-moon': 'Ultra-Soleil/Ultra-Lune',
  'lets-go-pikachu-lets-go-eevee': "Let's Go Pikachu/Let's Go Évoli",
  'sword-shield': 'Épée/Bouclier',
  'the-isle-of-armor': "L'Île Solitaire de l'Armure",
  'the-crown-tundra': 'Les Terres Enneigées de la Couronne',
  'brilliant-diamond-shining-pearl': 'Diamant Étincelant/Perle Scintillante',
  'legends-arceus': 'Légendes : Arceus',
  'scarlet-violet': 'Écarlate/Violet',
  'the-teal-mask': 'Le Masque Turquoise',
  'the-indigo-disk': 'Le Disque Indigo',
  'legends-za': 'Légendes Z-A',
  'mega-dimension': 'Méga-Dimension',
  champions: 'Champions',
};

@Component({
  selector: 'app-species-learnset',
  imports: [RouterLink, TranslocoPipe, AppIcon],
  templateUrl: './species-learnset.html',
})
export class SpeciesLearnset {
  private readonly catalog = inject(CatalogService);
  private readonly transloco = inject(TranslocoService);
  private readonly languageService = inject(LanguageService);

  readonly formId = input.required<number>();

  protected readonly state = signal<LoadState>('loading');
  protected readonly page = signal<LearnsetPage | null>(null);
  protected readonly selectedVersionGroup = signal<string | null>(null);
  protected readonly method = signal<string>('all');
  protected readonly search = signal('');
  protected readonly typeBadgeClass = typeBadgeClass;

  protected versionGroupName(slug: string): string {
    if (this.language().startsWith('fr')) {
      return VERSION_GROUP_NAMES_FR[slug] ?? slug;
    }
    return slug;
  }

  private requestId = 0;

  protected readonly language = this.languageService.current;

  protected readonly methods = computed<readonly string[]>(() => {
    const present = new Set((this.page()?.entries ?? []).map((entry) => entry.method));
    const ordered = METHOD_ORDER.filter((method) => present.has(method));
    const extra = [...present]
      .filter((method) => !(METHOD_ORDER as readonly string[]).includes(method))
      .sort();

    return [...ordered, ...extra];
  });

  protected readonly entries = computed<readonly LearnsetEntry[]>(() => {
    const entries = this.page()?.entries ?? [];
    const method = this.method();
    const tokens = tokenize(this.search());

    return entries.filter((entry) => {
      if (method !== 'all' && entry.method !== method) {
        return false;
      }
      if (tokens.length === 0) {
        return true;
      }

      const haystack = tokenize(`${entry.name} ${entry.slug}`);
      return tokens.every((token) => haystack.some((value) => value.startsWith(token)));
    });
  });

  constructor() {
    effect(() => {
      const formId = this.formId();
      const language = this.language();
      this.selectedVersionGroup.set(null);
      this.method.set('all');
      this.search.set('');
      void this.load(formId, language, null);
    });
  }

  protected onVersionGroupChange(event: Event): void {
    const slug = (event.target as HTMLSelectElement).value;
    this.selectedVersionGroup.set(slug);
    void this.load(this.formId(), this.language(), slug);
  }

  protected setMethod(method: string): void {
    this.method.set(method);
  }

  protected onSearch(event: Event): void {
    this.search.set((event.target as HTMLInputElement).value);
  }

  protected clearSearch(input: HTMLInputElement): void {
    input.value = '';
    this.search.set('');
  }

  protected retry(): void {
    void this.load(this.formId(), this.language(), this.selectedVersionGroup());
  }

  protected isMethodSelected(method: string): boolean {
    return this.method() === method;
  }

  protected chipClass(active: boolean): string {
    const base =
      'rounded-full border px-2.5 py-1 font-mono text-[0.64rem] font-semibold uppercase tracking-[0.05em] transition-colors';
    return active
      ? `${base} border-accent bg-accent/15 text-ink`
      : `${base} border-line bg-panel-muted/40 text-ink-soft hover:border-line-strong`;
  }

  protected methodLabel(entry: LearnsetEntry): string {
    if (entry.method === 'level-up' && entry.level > 0) {
      return this.transloco.translate('pokedex.learnset.level', { level: entry.level });
    }
    if (entry.method === 'machine' && entry.machine) {
      return entry.machine.toUpperCase();
    }

    return this.transloco.translate(`moves.methods.${entry.method}`);
  }

  private async load(formId: number, language: string, versionGroup: string | null): Promise<void> {
    const requestId = ++this.requestId;
    this.state.set('loading');

    try {
      const page = await this.catalog.learnset(formId, language, versionGroup);
      if (requestId !== this.requestId) {
        return;
      }

      this.page.set(page);
      this.selectedVersionGroup.set(page.versionGroup?.slug ?? null);
      this.state.set('ready');
    } catch {
      if (requestId !== this.requestId) {
        return;
      }

      this.state.set('error');
    }
  }
}

function tokenize(value: string): string[] {
  return value
    .toLowerCase()
    .replaceAll('œ', 'oe')
    .replaceAll('æ', 'ae')
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .split(/[^a-z0-9]+/)
    .filter((token) => token.length > 0);
}
