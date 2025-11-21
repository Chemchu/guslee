// Whenever the HTMX swaps content-section
document.addEventListener("htmx:afterSettle", function (evt) {
  if (evt.detail.target.id === "content-section") {
    document.dispatchEvent(new CustomEvent("contentUpdated"));
  }
});
