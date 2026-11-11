import { Component, DestroyRef, inject, signal } from '@angular/core';
import { getCurrentWindow, type Window as TauriWindow } from '@tauri-apps/api/window';
import { TranslocoPipe } from '@jsverse/transloco';

import { LanguageService } from '../../core/i18n/language.service';
import { isTauriRuntime } from '../../core/ipc/tauri-ipc';
import { ThemeService, type ThemePreference } from '../../core/theme/theme.service';
import { AppIcon, type IconName } from '../../shared/icon/app-icon';

interface ThemeOption {
  readonly value: ThemePreference;
  readonly icon: IconName;
  readonly labelKey: string;
}

interface LanguageOption {
  readonly code: string;
  readonly label: string;
}

@Component({
  selector: 'app-titlebar',
  imports: [TranslocoPipe, AppIcon],
  templateUrl: './app-titlebar.html',
})
export class AppTitlebar {
  protected readonly theme = inject(ThemeService);
  protected readonly language = inject(LanguageService);

  private readonly destroyRef = inject(DestroyRef);
  private readonly appWindow: TauriWindow | null = isTauriRuntime() ? getCurrentWindow() : null;

  protected readonly isDesktop = this.appWindow !== null;
  protected readonly maximized = signal(false);

  protected readonly themeOptions: readonly ThemeOption[] = [
    { value: 'system', icon: 'monitor', labelKey: 'theme.auto' },
    { value: 'light', icon: 'sun', labelKey: 'theme.light' },
    { value: 'dark', icon: 'moon', labelKey: 'theme.dark' },
  ];

  protected readonly languageOptions: readonly LanguageOption[] = [
    { code: 'fr', label: 'FR' },
    { code: 'en', label: 'EN' },
  ];

  constructor() {
    if (!this.appWindow) {
      return;
    }

    void this.syncMaximized();

    void this.appWindow
      .onResized(() => {
        void this.syncMaximized();
      })
      .then((unlisten) => this.destroyRef.onDestroy(unlisten));
  }

  protected themeOptionClass(active: boolean): string {
    const base = 'grid size-6 place-items-center rounded-[5px] transition-colors';

    return active
      ? `${base} bg-panel text-ink shadow-sm`
      : `${base} text-ink-muted hover:bg-panel-muted hover:text-ink`;
  }

  protected languageOptionClass(active: boolean): string {
    const base =
      'h-6 rounded-[5px] px-1.5 font-mono text-[0.6rem] font-semibold uppercase tracking-[0.04em] transition-colors';

    return active
      ? `${base} bg-panel text-ink shadow-sm`
      : `${base} text-ink-muted hover:bg-panel-muted hover:text-ink`;
  }

  protected minimize(): void {
    void this.appWindow?.minimize();
  }

  protected async toggleMaximize(): Promise<void> {
    if (!this.appWindow) {
      return;
    }

    await this.appWindow.toggleMaximize();
    await this.syncMaximized();
  }

  protected close(): void {
    void this.appWindow?.close();
  }

  private async syncMaximized(): Promise<void> {
    if (!this.appWindow) {
      return;
    }

    this.maximized.set(await this.appWindow.isMaximized());
  }
}
