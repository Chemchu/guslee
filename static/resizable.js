(function () {
  const MIN_WIDTH = 200;
  const MAX_WIDTH = 600;
  const VARIABLES = ["--sidebar-width", "--right-section-width"];

  const root = document.documentElement;
  const storageKey = (variable) => variable.replace(/^--/, "");
  const clamp = (width) => Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, width));
  const setWidth = (variable, width) =>
    root.style.setProperty(variable, `${clamp(width)}px`);

  try {
    for (const variable of VARIABLES) {
      const saved = parseInt(localStorage.getItem(storageKey(variable)), 10);
      if (!Number.isNaN(saved)) setWidth(variable, saved);
    }
  } catch (_) {}

  const resizerFor = (event) => event.target.closest?.("[data-resizer]");
  const targetOf = (handle) =>
    document.getElementById(handle.dataset.resizeTarget);

  document.addEventListener("pointerdown", function (event) {
    const handle = resizerFor(event);
    const target = handle && targetOf(handle);
    if (!target) return;

    event.preventDefault();
    const variable = handle.dataset.resizeVar;
    const fromLeftEdge = handle.dataset.resizeEdge === "left";

    const onMove = (moveEvent) => {
      const rect = target.getBoundingClientRect();
      setWidth(
        variable,
        fromLeftEdge ? rect.right - moveEvent.clientX : moveEvent.clientX - rect.left,
      );
    };

    const stopDragging = (endEvent) => {
      handle.releasePointerCapture(endEvent.pointerId);
      handle.removeEventListener("pointermove", onMove);
      handle.removeEventListener("pointerup", stopDragging);
      handle.removeEventListener("pointercancel", stopDragging);
      document.body.classList.remove("select-none", "cursor-col-resize");
      try {
        localStorage.setItem(
          storageKey(variable),
          Math.round(target.getBoundingClientRect().width),
        );
      } catch (_) {}
    };

    handle.setPointerCapture(event.pointerId);
    document.body.classList.add("select-none", "cursor-col-resize");
    handle.addEventListener("pointermove", onMove);
    handle.addEventListener("pointerup", stopDragging);
    handle.addEventListener("pointercancel", stopDragging);
  });

  document.addEventListener("dblclick", function (event) {
    const handle = resizerFor(event);
    if (!handle) return;

    root.style.removeProperty(handle.dataset.resizeVar);
    try {
      localStorage.removeItem(storageKey(handle.dataset.resizeVar));
    } catch (_) {}
  });
})();
