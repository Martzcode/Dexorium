import { Injectable, computed, effect, inject } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { TranslocoService } from '@jsverse/transloco';

import { SUPPORTED_LANGUAGES, isSupportedLanguage } from './languages';

const STORAGE_KEY = 'dexorium.language';

/**
 * Single source of truth for the active interface language.
 *
 * Keeps Transloco, the document `lang` attribute and localStorage in sync so
 * the title bar and the rest of the application never diverge.
 */
@Injectable({ providedIn: 'root' })
export class LanguageService {
  private readonly transloco = inject(TranslocoService);

  readonly languages = SUPPORTED_LANGUAGES;
  readonly current = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });
  readonly currentLabel = computed(
    () =>
      SUPPORTED_LANGUAGES.find((language) => language.code === this.current())?.label ??
      this.current().toUpperCase(),
  );

  constructor() {
    effect(() => {
      document.documentElement.lang = this.current();
    });
  }

  setLanguage(language: string): void {
    if (!isSupportedLanguage(language) || language === this.current()) {
      return;
    }

    this.transloco.setActiveLang(language);
    localStorage.setItem(STORAGE_KEY, language);
  }
}
