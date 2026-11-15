import { computed, effect, Injectable, signal } from '@angular/core';

export type ThemePreference = 'light' | 'dark' | 'system';
export type ResolvedTheme = 'light' | 'dark';
export type ThemeIcon = 'monitor' | 'sun' | 'moon';

const STORAGE_KEY = 'dexorium.theme';
/** Clicking the theme button walks through the preferences in this order. */
const CYCLE: readonly ThemePreference[] = ['system', 'light', 'dark'];

const ICONS: Record<ThemePreference, ThemeIcon> = {
  system: 'monitor',
  light: 'sun',
  dark: 'moon',
};

const LABEL_KEYS: Record<ThemePreference, string> = {
  system: 'theme.auto',
  light: 'theme.light',
  dark: 'theme.dark',
};

@Injectable({ providedIn: 'root' })
export class ThemeService {
  private readonly media = window.matchMedia('(prefers-color-scheme: dark)');
  private readonly systemTheme = signal<ResolvedTheme>(this.media.matches ? 'dark' : 'light');

  readonly preference = signal<ThemePreference>(readStoredPreference());
  readonly resolved = computed<ResolvedTheme>(() => {
    const preference = this.preference();
    return preference === 'system' ? this.systemTheme() : preference;
  });
  /** Icon matching the current preference: monitor for system, sun, moon. */
  readonly icon = computed<ThemeIcon>(() => ICONS[this.preference()]);
  readonly labelKey = computed(() => LABEL_KEYS[this.preference()]);

  constructor() {
    this.media.addEventListener('change', (event) => {
      this.systemTheme.set(event.matches ? 'dark' : 'light');
    });

    effect(() => {
      const theme = this.resolved();
      document.documentElement.dataset['theme'] = theme;
      document.documentElement.style.colorScheme = theme;
    });
  }

  set(preference: ThemePreference): void {
    this.preference.set(preference);
    localStorage.setItem(STORAGE_KEY, preference);
  }

  /** Switches to the next preference: system → light → dark → system. */
  cycle(): void {
    const index = CYCLE.indexOf(this.preference());
    this.set(CYCLE[(index + 1) % CYCLE.length] ?? 'system');
  }

  toggle(): void {
    this.set(this.resolved() === 'dark' ? 'light' : 'dark');
  }
}

function readStoredPreference(): ThemePreference {
  const stored = localStorage.getItem(STORAGE_KEY);
  return stored === 'light' || stored === 'dark' || stored === 'system' ? stored : 'system';
}
