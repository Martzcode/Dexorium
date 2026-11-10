import { Component, computed, effect, inject, signal } from '@angular/core';
import { RouterLink } from '@angular/router';
import { RouterOutlet } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';

import { ThemeService } from '../../core/theme/theme.service';
import { AppIcon, type IconName } from '../../shared/icon/app-icon';

interface NavItem {
  readonly labelKey: string;
  readonly icon: IconName;
  readonly route?: string;
  readonly ready?: boolean;
}

@Component({
  selector: 'app-shell',
  imports: [RouterOutlet, RouterLink, TranslocoPipe, AppIcon],
  templateUrl: './app-shell.html',
})
export class AppShell {
  protected readonly theme = inject(ThemeService);
  protected readonly transloco = inject(TranslocoService);
  protected readonly language = signal(this.transloco.getActiveLang());

  protected readonly navItems: readonly NavItem[] = [
    { labelKey: 'nav.overview', icon: 'layout-dashboard', route: '/', ready: true },
    { labelKey: 'nav.pokedex', icon: 'book-open' },
    { labelKey: 'nav.moves', icon: 'swords' },
    { labelKey: 'nav.abilities', icon: 'brain' },
    { labelKey: 'nav.items', icon: 'package' },
    { labelKey: 'nav.teams', icon: 'users' },
    { labelKey: 'nav.tools', icon: 'wrench' },
    { labelKey: 'nav.collection', icon: 'library-big' },
    { labelKey: 'nav.settings', icon: 'settings' },
  ];

  protected readonly themeLabelKey = computed(() =>
    this.theme.resolved() === 'dark' ? 'theme.switchToLight' : 'theme.switchToDark',
  );

  protected toggleTheme(): void {
    this.theme.toggle();
  }

  protected setLanguage(language: string): void {
    this.transloco.setActiveLang(language);
    this.language.set(language);
    localStorage.setItem('dexorium.language', language);
  }

  protected readonly year = new Date().getFullYear();
  protected readonly mobileNavOpen = signal(false);

  protected readonly sidebarClass = computed(() => {
    const base =
      'fixed inset-y-0 left-0 z-40 flex w-[260px] flex-col gap-7 border-r border-line bg-panel/80 px-4 py-5 backdrop-blur-xl transition-transform duration-200 lg:static lg:z-auto lg:w-auto lg:translate-x-0';
    return this.mobileNavOpen() ? `${base} translate-x-0 shadow-2xl` : `${base} -translate-x-full`;
  });

  constructor() {
    effect(() => {
      document.documentElement.lang = this.language();
    });
  }

  protected toggleMobileNav(): void {
    this.mobileNavOpen.update((open) => !open);
  }
}
