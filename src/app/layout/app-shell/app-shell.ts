import { Component, HostListener, computed, inject, signal } from '@angular/core';
import { RouterLink, RouterLinkActive } from '@angular/router';
import { RouterOutlet } from '@angular/router';
import { TranslocoPipe } from '@jsverse/transloco';

import { AppIcon, type IconName } from '../../shared/icon/app-icon';
import { CollectionService } from '../../core/collection/collection.service';
import { CommandPalette } from '../command-palette/command-palette';
import { CommandPaletteService } from '../command-palette/command-palette.service';
import { AppTitlebar } from '../titlebar/app-titlebar';

interface NavItem {
  readonly labelKey: string;
  readonly icon: IconName;
  readonly route?: string;
  readonly ready?: boolean;
}

@Component({
  selector: 'app-shell',
  imports: [
    RouterOutlet,
    RouterLink,
    RouterLinkActive,
    TranslocoPipe,
    AppIcon,
    AppTitlebar,
    CommandPalette,
  ],
  templateUrl: './app-shell.html',
})
export class AppShell {
  protected readonly navItems: readonly NavItem[] = [
    { labelKey: 'nav.overview', icon: 'layout-dashboard', route: '/', ready: true },
    { labelKey: 'nav.pokedex', icon: 'book-open', route: '/pokedex', ready: true },
    { labelKey: 'nav.moves', icon: 'swords', route: '/moves', ready: true },
    { labelKey: 'nav.abilities', icon: 'brain', route: '/abilities', ready: true },
    { labelKey: 'nav.items', icon: 'package', route: '/items', ready: true },
    { labelKey: 'nav.locations', icon: 'layout-grid', route: '/locations', ready: true },
    { labelKey: 'nav.natures', icon: 'sparkles', route: '/natures', ready: true },
    { labelKey: 'nav.teams', icon: 'users' },
    { labelKey: 'nav.tools', icon: 'wrench', route: '/tools/types', ready: true },
    { labelKey: 'nav.collection', icon: 'library-big' },
    { labelKey: 'nav.settings', icon: 'settings', route: '/settings', ready: true },
  ];

  private readonly palette = inject(CommandPaletteService);
  private readonly collection = inject(CollectionService);

  protected readonly year = new Date().getFullYear();
  protected readonly mobileNavOpen = signal(false);
  protected readonly shortcutsOpen = signal(false);

  constructor() {
    void this.collection.load();
  }

  protected openPalette(): void {
    this.palette.show();
  }

  protected readonly sidebarClass = computed(() => {
    const base =
      'fixed bottom-0 left-0 top-9 z-40 flex w-[260px] flex-col gap-7 border-r border-line bg-panel/80 px-4 py-5 backdrop-blur-xl transition-transform duration-200 lg:static lg:z-auto lg:w-auto lg:translate-x-0';
    return this.mobileNavOpen() ? `${base} translate-x-0 shadow-2xl` : `${base} -translate-x-full`;
  });

  protected toggleMobileNav(): void {
    this.mobileNavOpen.update((open) => !open);
  }

  protected closeMobileNav(): void {
    this.mobileNavOpen.set(false);
  }

  @HostListener('window:keydown', ['$event'])
  protected onKeydown(event: KeyboardEvent): void {
    const target = event.target as HTMLElement | null;
    const typing =
      !!target &&
      (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable);

    if (event.key === 'Escape') {
      if (this.shortcutsOpen()) {
        this.shortcutsOpen.set(false);
        event.preventDefault();
      }
      return;
    }

    if (typing) {
      return;
    }

    if (event.key === '?') {
      this.shortcutsOpen.update((value) => !value);
      event.preventDefault();
    }
  }

  protected navClass(item: NavItem, active: boolean): string {
    const base =
      'flex items-center gap-2.5 rounded-lg px-2.5 py-2.5 text-sm no-underline transition-colors';

    return active
      ? `${base} bg-accent/10 font-semibold text-ink`
      : `${base} text-ink-soft hover:bg-panel-muted hover:text-ink`;
  }
}
