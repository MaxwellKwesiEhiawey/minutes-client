import { useCallback, useMemo, useState, type ReactNode } from "react";
import {
  I18nContext,
  applyLocale,
  getLocale,
  setLocale as persistLocale,
  translatorFor,
  type Locale,
} from "./index";

/**
 * Holds the current UI language and hands a translator to the tree.
 *
 * Both windows mount their own provider: the main window and the floating
 * meeting prompt are separate webviews with separate React roots, and each reads
 * the same `localStorage` key, so a language chosen in Settings applies to the
 * prompt the next time it opens.
 */
export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(getLocale);

  const setLocale = useCallback((next: Locale) => {
    setLocaleState(next);
    persistLocale(next);
  }, []);

  const value = useMemo(
    () => ({ locale, t: translatorFor(locale), setLocale }),
    [locale, setLocale],
  );

  // Keep `<html lang>` in step with the rendered language, for assistive tech.
  applyLocale(locale);

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}
