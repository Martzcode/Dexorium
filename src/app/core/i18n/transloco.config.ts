import { isDevMode } from '@angular/core';
import { provideHttpClient } from '@angular/common/http';
import { provideTransloco, type Translation, TranslocoLoader } from '@jsverse/transloco';
import { inject, Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import type { Observable } from 'rxjs';

@Injectable({ providedIn: 'root' })
export class TranslocoHttpLoader implements TranslocoLoader {
  private readonly http = inject(HttpClient);

  getTranslation(lang: string): Observable<Translation> {
    return this.http.get<Translation>(`/i18n/${lang}.json`);
  }
}

export const translocoProviders = provideTransloco({
  config: {
    availableLangs: ['en', 'fr'],
    defaultLang: readInitialLanguage(),
    fallbackLang: 'en',
    reRenderOnLangChange: true,
    prodMode: !isDevMode(),
  },
  loader: TranslocoHttpLoader,
});

function readInitialLanguage(): string {
  const stored = localStorage.getItem('dexorium.language');
  if (stored === 'en' || stored === 'fr') {
    return stored;
  }

  return navigator.language.toLowerCase().startsWith('fr') ? 'fr' : 'en';
}
