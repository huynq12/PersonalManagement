import i18n from 'i18next'
import LanguageDetector from 'i18next-browser-languagedetector'
import { initReactI18next } from 'react-i18next'
import en from './locales/en.json'
import vi from './locales/vi.json'

export const SUPPORTED_LANGUAGES = ['en', 'vi'] as const
export type Language = (typeof SUPPORTED_LANGUAGES)[number]

export const FALLBACK_LANGUAGE: Language = 'en'

// Versioned so the stored value can change shape later without clashing with old data.
const STORAGE_KEY = 'app.language.v1'

// Keeps the <html lang> attribute and the browser tab title in the active language.
function syncDocument(language: string) {
  document.documentElement.lang = language
  document.title = i18n.t('app.title')
}

i18n.on('languageChanged', syncDocument)

void i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources: { en: { translation: en }, vi: { translation: vi } },
    supportedLngs: SUPPORTED_LANGUAGES,
    fallbackLng: FALLBACK_LANGUAGE,
    // Treat regional variants such as "vi-VN" or "en-GB" as their base language.
    load: 'languageOnly',
    detection: {
      // A saved choice wins over the browser language.
      order: ['localStorage', 'navigator'],
      lookupLocalStorage: STORAGE_KEY,
      caches: ['localStorage'],
    },
    // React already escapes rendered values.
    interpolation: { escapeValue: false },
  })

export default i18n
