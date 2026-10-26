(function () {
  const STORAGE_KEY = "explorer-view";
  const VIEWS = ["folders", "topics"];
  const DEFAULT_VIEW = "topics";

  function savedView() {
    try {
      const view = localStorage.getItem(STORAGE_KEY);
      return VIEWS.includes(view) ? view : DEFAULT_VIEW;
    } catch {
      return DEFAULT_VIEW;
    }
  }

  function saveView(view) {
    try {
      localStorage.setItem(STORAGE_KEY, view);
    } catch {}
  }

  window.explorerView = savedView;

  function showView(view) {
    for (const tab of document.querySelectorAll("[data-explorer-view]")) {
      tab.setAttribute("aria-selected", String(tab.dataset.explorerView === view));
    }
    const input = document.getElementById("explorer-view");
    if (input) input.value = view;
  }

  function reloadList() {
    document
      .getElementById("desktop-search-input")
      ?.dispatchEvent(new KeyboardEvent("keyup", { key: "Enter" }));
  }

  let openedFor = null;

  function markCurrentPost() {
    const list = document.getElementById("list-section");
    if (!list) return;

    const current = location.pathname === "/" ? "/posts/welcome" : location.pathname;
    const shouldOpen = openedFor !== current;
    for (const link of list.querySelectorAll('a[href^="/posts/"]')) {
      if (link.pathname !== current) {
        link.removeAttribute("aria-current");
        continue;
      }
      link.setAttribute("aria-current", "page");
      if (!shouldOpen) continue;
      for (let folder = link.closest("details"); folder; folder = folder.parentElement?.closest("details")) {
        folder.open = true;
      }
      openedFor = current;
    }
  }

  function syncClearButton() {
    const input = document.getElementById("desktop-search-input");
    const clear = document.getElementById("desktop-search-clear");
    if (input && clear) clear.hidden = input.value === "";
  }

  document.addEventListener("input", (event) => {
    if (event.target.id === "desktop-search-input") syncClearButton();
  });

  document.addEventListener("click", (event) => {
    if (!event.target.closest?.("#desktop-search-clear")) return;
    const input = document.getElementById("desktop-search-input");
    input.value = "";
    syncClearButton();
    reloadList();
    input.focus();
  });

  document.addEventListener("click", (event) => {
    const tab = event.target.closest?.("[data-explorer-view]");
    if (!tab) return;
    showView(tab.dataset.explorerView);
    saveView(tab.dataset.explorerView);
    openedFor = null;
    reloadList();
  });

  document.addEventListener("DOMContentLoaded", () => {
    showView(savedView());
    syncClearButton();
  });
  window.addEventListener("pageshow", syncClearButton);
  document.addEventListener("htmx:after:settle", (event) => {
    if (event.target?.id === "list-section") openedFor = null;
    markCurrentPost();
  });
})();
