import { Component, computed, effect, inject, signal } from '@angular/core';
import { RouterLink } from '@angular/router';
import { RouterOutlet } from '@angular/router';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';

import { AppInfoService } from '../../core/app-info/app-info.service';
import { ThemeService } from '../../core/theme/theme.service';

interface NavItem {
  readonly labelKey: string;
  readonly icon: string;
  readonly route?: string;
  readonly ready?: boolean;
}

@Component({
  selector: 'app-shell',
  imports: [RouterOutlet, RouterLink, TranslocoPipe],
  templateUrl: './app-shell.html',
  styleUrl: './app-shell.scss',
})
export class AppShell {
  protected readonly theme = inject(ThemeService);
  protected readonly appInfo = inject(AppInfoService);
  protected readonly transloco = inject(TranslocoService);
  protected readonly language = signal(this.transloco.getActiveLang());

  protected readonly navItems: readonly NavItem[] = [
    { labelKey: 'nav.overview', icon: '◈', route: '/', ready: true },
    { labelKey: 'nav.pokedex', icon: '▦' },
    { labelKey: 'nav.moves', icon: '✦' },
    { labelKey: 'nav.abilities', icon: '◉' },
    { labelKey: 'nav.items', icon: '◆' },
    { labelKey: 'nav.teams', icon: '⬡' },
    { labelKey: 'nav.tools', icon: '⌁' },
    { labelKey: 'nav.collection', icon: '▤' },
    { labelKey: 'nav.settings', icon: '⚙' },
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

  constructor() {
    effect(() => {
      document.documentElement.lang = this.language();
    });
  }

  protected toggleMobileNav(): void {
    this.mobileNavOpen.update((open) => !open);
  }
}
