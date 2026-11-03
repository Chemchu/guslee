document.addEventListener("htmx:after:swap", function (evt) {
  if (evt.detail.ctx.target?.id === "main-section") {
    // Clear mobile search when content loads
    const mobileInput = document.getElementById("mobile-search-input");
    if (mobileInput) {
      mobileInput.value = "";
      document.getElementById("mobile-search-clear").hidden = true;
    }
    window.closeMobilePanels?.();
  }
});
