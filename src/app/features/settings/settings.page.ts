import { Component, computed, effect, inject, signal } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { TranslocoPipe, TranslocoService } from '@jsverse/transloco';

import { AppInfoService } from '../../core/app-info/app-info.service';
import { LanguageService } from '../../core/i18n/language.service';
import { SpriteFolderService } from '../../core/media/sprite-folder.service';
import { ThemeService } from '../../core/theme/theme.service';
import { AppIcon } from '../../shared/icon/app-icon';
import { DATASET_TABLE_LABELS, type DatasetInfo } from './settings.models';
import { DEVELOPER_PROFILE_URL, SettingsService } from './settings.service';

type LoadState = 'loading' | 'ready' | 'error';

@Component({
  selector: 'app-settings',
  imports: [TranslocoPipe, AppIcon],
  templateUrl: './settings.page.html',
})
export class SettingsPage {
  protected readonly settings = inject(SettingsService);
  private readonly transloco = inject(TranslocoService);

  protected readonly appInfo = inject(AppInfoService);
  protected readonly theme = inject(ThemeService);
  protected readonly languagePreference = inject(LanguageService);
  protected readonly sprites = inject(SpriteFolderService);
  protected readonly state = signal<LoadState>('loading');
  protected readonly dataset = signal<DatasetInfo | null>(null);
  protected readonly developerProfileUrl = DEVELOPER_PROFILE_URL;

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

  protected onLanguageChange(event: Event): void {
    this.languagePreference.setLanguage((event.target as HTMLSelectElement).value);
  }

  protected async pickSpritesFolder(): Promise<void> {
    await this.sprites.pickFolder();
  }

  protected clearSpritesFolder(): void {
    this.sprites.clear();
  }

  protected exportData(): void {
    const dump: Record<string, string | null> = {};
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i);
      if (key) {
        dump[key] = localStorage.getItem(key);
      }
    }
    const blob = new Blob([JSON.stringify(dump, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = 'dexorium-backup.json';
    anchor.click();
    URL.revokeObjectURL(url);
  }

  protected async importData(event: Event): Promise<void> {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) {
      return;
    }
    const text = await file.text();
    try {
      const dump = JSON.parse(text) as Record<string, string | null>;
      for (const [key, value] of Object.entries(dump)) {
        if (value === null) {
          localStorage.removeItem(key);
        } else {
          localStorage.setItem(key, value);
        }
      }
      window.location.reload();
    } catch {
      /* ignore */
    }
  }

  protected openDeveloperProfile(event: Event): void {
    event.preventDefault();
    void this.settings.openDeveloperProfile();
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
