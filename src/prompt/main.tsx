import ReactDOM from "react-dom/client";
import { MeetingPrompt } from "./MeetingPrompt";
import "../styles.css";
import "./prompt.css";
import { initTheme } from "../theme";
import { applyLocale } from "../i18n";
import { I18nProvider } from "../i18n/I18nProvider";

initTheme();
applyLocale();

// No StrictMode here: the prompt window is short-lived and StrictMode's
// double effect mount previously raced a consume-once payload lookup.
ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <I18nProvider>
    <MeetingPrompt />
  </I18nProvider>,
);
