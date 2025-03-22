// Global variables
window.offset = 0;
window.limit = 4;
window.prevBtn = document.getElementById("prev-btn");
window.nextBtn = document.getElementById("next-btn");

// Function to remove event listeners
function removePaginationEventListeners() {
  if (prevBtn) {
    prevBtn.removeEventListener("click", handlePrevClick);
  }
  if (nextBtn) {
    nextBtn.removeEventListener("click", handleNextClick);
  }
}

// Function to clean up variables
function cleanupVariables() {
  removePaginationEventListeners(); // Remove event listeners

  // Nullify references to DOM elements and variables
  prevBtn = null;
  nextBtn = null;
  offset = null;
  limit = null;
}

// Event handler for the previous button
function handlePrevClick() {
  if (offset > 0) {
    offset = Math.max(0, offset - limit);
    fetchArticles();
  }
}

// Event handler for the next button
function handleNextClick() {
  offset += limit;
  fetchArticles();
}

// Function to initialize pagination
function initializePagination() {
  // Remove any previously attached event listeners
  removePaginationEventListeners();

  // Add new event listeners
  if (prevBtn) {
    prevBtn.addEventListener("click", handlePrevClick);
  }

  if (nextBtn) {
    nextBtn.addEventListener("click", handleNextClick);
  }
}

// Function to fetch articles based on the updated offset
function fetchArticles() {
  htmx.ajax(
    "GET",
    `/articles/content?limit=${limit}&offset=${offset}`,
    "#articles-list",
  );
}

// Handle HTMX response error (404)
document.addEventListener("htmx:responseError", (evt) => {
  const xhr = evt.detail.xhr;
  if (xhr.status == 404) {
    offset = Math.max(0, offset - limit); // Reset offset if 404 (no more content)
  }
});

// HTMX event listener to handle dynamically swapped content
document.addEventListener("htmx:afterSwap", (evt) => {
  // Only initialize pagination if the content swapped is for #articles-list
  if (evt.target.id === "articles-list") {
    initializePagination();
  }
});

// Clean up when the page is being unloaded or content is replaced
window.addEventListener("beforeunload", cleanupVariables); // Ensures cleanup when navigating away

// Initial pagination setup when page loads
initializePagination();
