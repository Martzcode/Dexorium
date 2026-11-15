import { Component, DestroyRef, inject, signal } from '@angular/core';
import { getCurrentWindow, type Window as TauriWindow } from '@tauri-apps/api/window';
import { TranslocoPipe } from '@jsverse/transloco';

import { LanguageService } from '../../core/i18n/language.service';
import { isTauriRuntime } from '../../core/ipc/tauri-ipc';
import { ThemeService } from '../../core/theme/theme.service';
import { AppIcon } from '../../shared/icon/app-icon';

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

  protected onLanguageChange(event: Event): void {
    this.language.setLanguage((event.target as HTMLSelectElement).value);
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
