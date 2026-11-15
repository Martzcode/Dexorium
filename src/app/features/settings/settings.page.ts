import { Component, computed, effect, inject, signal } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';

import { AppInfoService } from '../../core/app-info/app-info.service';
import { LanguageService } from '../../core/i18n/language.service';
import { ThemeService, type ThemePreference } from '../../core/theme/theme.service';
import { AppIcon } from '../../shared/icon/app-icon';
import { DATASET_TABLE_LABELS, type DatasetInfo } from './settings.models';
import { SettingsService } from './settings.service';

type LoadState = 'loading' | 'ready' | 'error';

@Component({
  selector: 'app-settings',
  imports: [TranslocoPipe, AppIcon],
  templateUrl: './settings.page.html',
})
export class SettingsPage {
  private readonly settings = inject(SettingsService);
  private readonly transloco = inject(TranslocoService);

  protected readonly appInfo = inject(AppInfoService);
  protected readonly theme = inject(ThemeService);
  protected readonly languagePreference = inject(LanguageService);
  protected readonly state = signal<LoadState>('loading');
  protected readonly dataset = signal<DatasetInfo | null>(null);

  protected readonly themeOptions: readonly {
    key: ThemePreference;
    labelKey: string;
  }[] = [
    { key: 'system', labelKey: 'settings.preferences.themeSystem' },
    { key: 'light', labelKey: 'settings.preferences.themeLight' },
    { key: 'dark', labelKey: 'settings.preferences.themeDark' },
  ];

  private readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });

  protected readonly languages = computed(() => {
    const codes = this.dataset()?.languages ?? [];
    const names = new Intl.DisplayNames([this.language()], { type: 'language' });
    return codes.map((code) => names.of(code) ?? code.toUpperCase());
  });

  constructor() {
    effect(() => {
      const language = this.language();
      void this.load(language);
    });
  }

  protected setTheme(preference: ThemePreference): void {
    this.theme.set(preference);
  }

  protected onLanguageChange(event: Event): void {
    this.languagePreference.setLanguage((event.target as HTMLSelectElement).value);
  }

  protected tableLabel(table: string): string {
    return DATASET_TABLE_LABELS[table] ?? table;
  }

  protected formatCount(value: number): string {
    return new Intl.NumberFormat(this.language()).format(value);
  }

  protected retry(): void {
    void this.load(this.language());
  }

  private async load(_language: string): Promise<void> {
    this.state.set('loading');

    try {
      this.dataset.set(await this.settings.datasetInfo());
      this.state.set('ready');
    } catch {
      this.state.set('error');
    }
  }
}
