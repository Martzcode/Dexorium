import {
  Component,
  ElementRef,
  HostListener,
  computed,
  effect,
  inject,
  signal,
  viewChild,
} from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { Router } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';

import { CatalogService } from '../../features/catalog/catalog.service';
import type { SearchHit, SearchEntityType } from '../../features/catalog/catalog.models';
import { AppIcon, type IconName } from '../../shared/icon/app-icon';
import { CommandPaletteService } from './command-palette.service';

interface PaletteAction {
  readonly labelKey: string;
  readonly icon: IconName;
  readonly route: readonly string[];
}

interface PaletteItem {
  readonly id: string;
  readonly kind: 'action' | SearchEntityType;
  readonly label: string;
  readonly subtitle: string;
  readonly icon: IconName;
  readonly route: readonly string[];
}

const ACTIONS: readonly PaletteAction[] = [
  { labelKey: 'nav.overview', icon: 'layout-dashboard', route: ['/'] },
  { labelKey: 'nav.pokedex', icon: 'book-open', route: ['/pokedex'] },
  { labelKey: 'nav.moves', icon: 'swords', route: ['/moves'] },
  { labelKey: 'nav.abilities', icon: 'brain', route: ['/abilities'] },
  { labelKey: 'nav.tools', icon: 'wrench', route: ['/tools/types'] },
  { labelKey: 'nav.settings', icon: 'settings', route: ['/settings'] },
];

const ENTITY_ICONS: Readonly<Record<SearchEntityType, IconName>> = {
  species: 'book-open',
  move: 'swords',
  ability: 'brain',
};

const ENTITY_LABELS: Readonly<Record<SearchEntityType, string>> = {
  species: 'palette.kinds.species',
  move: 'palette.kinds.move',
  ability: 'palette.kinds.ability',
};

const ENTITY_ROUTES: Readonly<Record<SearchEntityType, string>> = {
  species: '/pokedex',
  move: '/moves',
  ability: '/abilities',
};

const SEARCH_DEBOUNCE_MS = 140;

@Component({
  selector: 'app-command-palette',
  imports: [TranslocoPipe, AppIcon],
  templateUrl: './command-palette.html',
})
export class CommandPalette {
  private readonly palette = inject(CommandPaletteService);
  private readonly catalog = inject(CatalogService);
  private readonly router = inject(Router);
  private readonly transloco = inject(TranslocoService);
  private readonly input = viewChild<ElementRef<HTMLInputElement>>('paletteInput');

  protected readonly isOpen = this.palette.open;
  protected readonly query = signal('');
  protected readonly hints = signal<readonly SearchHit[]>([]);
  protected readonly loading = signal(false);
  protected readonly error = signal(false);
  protected readonly selected = signal(0);

  private readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  protected readonly actions = computed<readonly PaletteItem[]>(() => {
    const term = this.query().trim().toLowerCase();

    return ACTIONS.map((action) => {
      const label = this.transloco.translate(action.labelKey);
      return {
        id: `action:${action.route.join('/')}`,
        kind: 'action' as const,
        label,
        subtitle: this.transloco.translate('palette.action'),
        icon: action.icon,
        route: action.route,
      };
    }).filter((action) => term.length === 0 || action.label.toLowerCase().includes(term));
  });

  protected readonly items = computed<readonly PaletteItem[]>(() => {
    // Read the language signal so labels refresh when the user switches.
    this.language();

    const hints = this.hints().map<PaletteItem>((hit) => ({
      id: `${hit.entityType}:${hit.id}`,
      kind: hit.entityType,
      label: hit.name,
      subtitle: hit.subtitle,
      icon: ENTITY_ICONS[hit.entityType],
      route: [ENTITY_ROUTES[hit.entityType], String(hit.id)],
    }));

    return [...this.actions(), ...hints];
  });

  protected readonly activeDescendant = computed(() => {
    const index = this.selected();
    return index >= 0 && index < this.items().length ? `palette-item-${index}` : null;
  });

  private timer?: ReturnType<typeof setTimeout>;
  private requestId = 0;

  constructor() {
    effect(() => {
      if (!this.isOpen()) {
        return;
      }

      this.query.set('');
      this.hints.set([]);
      this.selected.set(0);
      this.error.set(false);
      setTimeout(() => this.input()?.nativeElement.focus());
    });

    effect(() => {
      const length = this.items().length;
      if (length === 0 && this.selected() !== 0) {
        this.selected.set(0);
      } else if (this.selected() >= length) {
        this.selected.set(Math.max(0, length - 1));
      }
    });
  }

  protected onInput(event: Event): void {
    const value = (event.target as HTMLInputElement).value;
    this.query.set(value);
    this.selected.set(0);
    this.error.set(false);

    if (this.timer) {
      clearTimeout(this.timer);
    }

    const term = value.trim();
    if (term.length === 0) {
      this.hints.set([]);
      this.loading.set(false);
      return;
    }

    this.loading.set(true);
    this.timer = setTimeout(() => void this.search(term), SEARCH_DEBOUNCE_MS);
  }

  protected onKeydown(event: KeyboardEvent): void {
    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        this.move(1);
        break;
      case 'ArrowUp':
        event.preventDefault();
        this.move(-1);
        break;
      case 'Enter':
        event.preventDefault();
        this.activate(this.items()[this.selected()]);
        break;
      case 'Escape':
        event.preventDefault();
        this.close();
        break;
      default:
        break;
    }
  }

  protected select(index: number): void {
    this.selected.set(index);
  }

  protected activate(item: PaletteItem | undefined): void {
    if (!item) {
      return;
    }

    this.close();
    void this.router.navigate(item.route);
  }

  protected close(): void {
    this.palette.hide();
  }

  protected kindLabel(kind: PaletteItem['kind']): string {
    return kind === 'action' ? 'palette.action' : ENTITY_LABELS[kind];
  }

  @HostListener('document:keydown', ['$event'])
  protected onDocumentKeydown(event: KeyboardEvent): void {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') {
      event.preventDefault();
      this.palette.toggle();
      return;
    }

    if (event.key === 'Escape' && this.isOpen()) {
      event.preventDefault();
      this.close();
    }
  }

  private move(delta: number): void {
    const length = this.items().length;
    if (length === 0) {
      return;
    }

    const next = (this.selected() + delta + length) % length;
    this.selected.set(next);
    setTimeout(() =>
      document.getElementById(`palette-item-${next}`)?.scrollIntoView({ block: 'nearest' }),
    );
  }

  private async search(term: string): Promise<void> {
    const requestId = ++this.requestId;

    try {
      const hits = await this.catalog.suggest(term, this.language(), 8);
      if (requestId !== this.requestId) {
        return;
      }

      this.hints.set(hits);
      this.selected.set(0);
    } catch {
      if (requestId === this.requestId) {
        this.error.set(true);
        this.hints.set([]);
      }
    } finally {
      if (requestId === this.requestId) {
        this.loading.set(false);
      }
    }
  }
}
