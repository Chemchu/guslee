// Whenever the HTMX swaps main-section
document.addEventListener("htmx:afterSettle", function (evt) {
  if (evt.detail.target.id === "main-section") {
    document.dispatchEvent(new CustomEvent("contentUpdated"));
  }
});
