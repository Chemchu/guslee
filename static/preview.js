(function () {
  const LINKS = "#main-section .prose a[href], #backlinks a[href]";
  const SHOW_DELAY_MS = 400;
  const HIDE_DELAY_MS = 250;
  const GAP_PX = 8;

  const cache = new Map();
  let popover = null;
  let activeLink = null;
  let showTimer = null;
  let hideTimer = null;

  function previewTarget(link) {
    const url = new URL(link.href, location.href);
    if (url.origin !== location.origin || !url.pathname.startsWith("/posts/")) {
      return null;
    }
    return {
      url: "/preview/" + url.pathname.slice("/posts/".length),
      page: url.pathname,
      anchor: decodeURIComponent(url.hash.slice(1)),
    };
  }

  async function fetchPreview(url) {
    if (!cache.has(url)) {
      const response = await fetch(url).catch(() => null);
      cache.set(url, response?.ok ? await response.text() : null);
    }
    return cache.get(url);
  }

  function getPopover() {
    if (popover?.isConnected) return popover;

    popover = document.createElement("div");
    popover.id = "post-preview";
    popover.className =
      "fixed z-50 w-[28rem] max-w-[calc(100vw-1rem)] max-h-80 overflow-y-auto p-4 bg-background-color border border-shade-color shadow-xl";
    popover.hidden = true;
    popover.addEventListener("mouseenter", () => clearTimeout(hideTimer));
    popover.addEventListener("mouseleave", scheduleHide);
    document.getElementById("main-section").appendChild(popover);
    return popover;
  }

  function place(box, link) {
    const rect = link.getBoundingClientRect();
    const left = Math.min(rect.left, innerWidth - box.offsetWidth - GAP_PX);
    const below = rect.bottom + GAP_PX;
    const above = rect.top - GAP_PX - box.offsetHeight;
    const fitsBelow = below + box.offsetHeight <= innerHeight - GAP_PX;

    box.style.left = Math.max(GAP_PX, left) + "px";
    box.style.top = (fitsBelow || above < GAP_PX ? below : above) + "px";
  }

  async function show(link) {
    const target = previewTarget(link);
    if (!target) return;

    const html = await fetchPreview(target.url);
    if (!html || activeLink !== link || !link.isConnected) return;

    const box = getPopover();
    box.innerHTML = html;
    for (const anchor of box.querySelectorAll('a[href^="#"]')) {
      anchor.setAttribute("href", target.page + anchor.getAttribute("href"));
    }
    box.hidden = false;
    place(box, link);
    htmx.process(box);

    const heading = target.anchor
      ? [...box.querySelectorAll("[data-preview-anchor]")].find(
          (h) => h.dataset.previewAnchor === target.anchor,
        )
      : null;
    box.scrollTop = heading ? heading.offsetTop : 0;
  }

  function hide() {
    clearTimeout(showTimer);
    activeLink = null;
    if (popover) popover.hidden = true;
  }

  function scheduleHide() {
    clearTimeout(showTimer);
    clearTimeout(hideTimer);
    hideTimer = setTimeout(hide, HIDE_DELAY_MS);
  }

  document.addEventListener("mouseover", (event) => {
    if (!matchMedia("(hover: hover)").matches) return;

    const link = event.target.closest?.(LINKS);
    if (!link || popover?.contains(link)) return;
    if (link === activeLink) {
      clearTimeout(hideTimer);
      return;
    }

    clearTimeout(showTimer);
    clearTimeout(hideTimer);
    activeLink = link;
    showTimer = setTimeout(() => show(link), SHOW_DELAY_MS);
  });

  document.addEventListener("mouseout", (event) => {
    const link = event.target.closest?.(LINKS);
    if (link && link === activeLink && !link.contains(event.relatedTarget)) {
      scheduleHide();
    }
  });

  document.addEventListener("click", hide);
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape") hide();
  });
  document.addEventListener(
    "scroll",
    (event) => {
      if (!popover?.contains(event.target)) hide();
    },
    true,
  );
})();
