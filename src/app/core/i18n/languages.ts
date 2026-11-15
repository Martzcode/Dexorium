/** Languages shipped with the interface bundle. Add an entry here to expose it. */
export const SUPPORTED_LANGUAGES = [
  { code: 'fr', label: 'Français' },
  { code: 'en', label: 'English' },
] as const;

export type SupportedLanguage = (typeof SUPPORTED_LANGUAGES)[number];
export type SupportedLanguageCode = SupportedLanguage['code'];

export const SUPPORTED_LANGUAGE_CODES: readonly SupportedLanguageCode[] = SUPPORTED_LANGUAGES.map(
  (language) => language.code,
);

export function isSupportedLanguage(
  value: string | null | undefined,
): value is SupportedLanguageCode {
  return SUPPORTED_LANGUAGE_CODES.includes(value as SupportedLanguageCode);
}
