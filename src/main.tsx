import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";
import { initTheme } from "./theme";
import { initReadingPrefs } from "./readingPrefs";
import { applyLocale } from "./i18n";
import { I18nProvider } from "./i18n/I18nProvider";

initTheme();
initReadingPrefs();
applyLocale();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <I18nProvider>
      <App />
    </I18nProvider>
  </React.StrictMode>,
);
