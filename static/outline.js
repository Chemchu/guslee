(function () {
  const STORAGE_KEY = "side-panel-tab";
  const CURRENT_OFFSET_PX = 24;

  function savedTab() {
    try {
      return localStorage.getItem(STORAGE_KEY);
    } catch {
      return null;
    }
  }

  function saveTab(name) {
    try {
      localStorage.setItem(STORAGE_KEY, name);
    } catch {}
  }

  function selectTab(name) {
    const tabs = [...document.querySelectorAll("[data-panel-tab]")];
    if (!tabs.some((tab) => tab.dataset.panelTab === name)) return;

    for (const tab of tabs) {
      tab.setAttribute("aria-selected", String(tab.dataset.panelTab === name));
    }
    for (const panel of document.querySelectorAll("[data-panel]")) {
      panel.hidden = panel.dataset.panel !== name;
    }
  }

  function markCurrentHeading() {
    const post = document.querySelector("#main-section .prose");
    const links = [...document.querySelectorAll('#outline-section a[href^="#"]')];
    if (!post || links.length === 0) return;

    const atBottom = post.scrollTop + post.clientHeight >= post.scrollHeight - 2;
    const top = post.getBoundingClientRect().top + CURRENT_OFFSET_PX;
    let current = null;
    for (const link of links) {
      const heading = document.getElementById(decodeURIComponent(link.hash.slice(1)));
      if (heading && heading.getBoundingClientRect().top <= top) current = link;
    }
    if (atBottom && post.scrollTop > 0) current = links[links.length - 1];
    current ??= links[0];

    for (const link of links) {
      link.setAttribute("aria-current", String(link === current));
    }
  }

  function setUp() {
    const tab = savedTab();
    if (tab) selectTab(tab);
    markCurrentHeading();
  }

  document.addEventListener("click", (event) => {
    const tab = event.target.closest?.("[data-panel-tab]");
    if (!tab) return;
    selectTab(tab.dataset.panelTab);
    saveTab(tab.dataset.panelTab);
  });

  let frame = null;
  document.addEventListener(
    "scroll",
    (event) => {
      if (frame || !event.target.matches?.("#main-section .prose")) return;
      frame = requestAnimationFrame(() => {
        frame = null;
        markCurrentHeading();
      });
    },
    true,
  );

  document.addEventListener("DOMContentLoaded", setUp);
  document.addEventListener("htmx:after:settle", setUp);
})();
