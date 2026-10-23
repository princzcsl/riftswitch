import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";

// Pas de menu contextuel du navigateur ni de raccourcis de rechargement : on est dans un launcher.
window.addEventListener("contextmenu", (e) => {
  const el = e.target as HTMLElement;
  if (!el.closest("input, textarea, pre")) e.preventDefault();
});
window.addEventListener("keydown", (e) => {
  if (e.key === "F5" || (e.ctrlKey && e.key.toLowerCase() === "r") || e.key === "F11") e.preventDefault();
});

async function start() {
  // Mode démo : uniquement en développement et hors de Tauri (navigateur classique).
  if (import.meta.env.DEV && !("__TAURI_INTERNALS__" in window)) {
    const { installMock } = await import("./lib/mock");
    installMock();
  }
  // Mini-fenêtre de confirmation (fermeture de l'app) : seulement la carte, sans l'interface.
  if (new URLSearchParams(location.search).get("view") === "confirm") {
    document.documentElement.classList.add("confirm-window");
    const { default: ConfirmWindow } = await import("./ConfirmWindow.svelte");
    mount(ConfirmWindow, { target: document.getElementById("app")! });
    return;
  }
  mount(App, { target: document.getElementById("app")! });
}

start();
