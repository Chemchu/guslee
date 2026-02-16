document.addEventListener("htmx:afterSwap", function (evt) {
  if (evt.detail.target.id === "main-section") {
    // Clear mobile search when content loads
    const mobileInput = document.querySelector(
      ".md\\:hidden input[name=query]",
    );
    if (mobileInput) {
      mobileInput.value = "";
      document.getElementById("mobile-search-results").classList.add("hidden");
    }
  }
});
