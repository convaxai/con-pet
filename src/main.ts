import { getCurrentWindow } from "@tauri-apps/api/window";
import { renderOverlay } from "./overlay";
import { renderSettings } from "./settings";
import "./styles.css";

const isOverlay = getCurrentWindow().label === "overlay";
document.documentElement.classList.toggle("overlay-root", isOverlay);

window.addEventListener("DOMContentLoaded", () => {
  const render = isOverlay ? renderOverlay : renderSettings;
  void render().catch((error) => {
    console.error(error);
    document.body.innerHTML = `<pre class="fatal-error">Con Pet\n${String(error)}</pre>`;
  });
});
