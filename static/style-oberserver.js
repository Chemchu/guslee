let currentPage = window.location.pathname;
document.addEventListener("DOMContentLoaded", (_event) => {
  /* handleProseClasses(currentPage);
  handleRightSectionClasses(currentPage); */
});

/* setInterval(function () {
  if (currentPage != window.location.pathname) {
    currentPage = window.location.pathname;
    handleProseClasses(currentPage);
    handleRightSectionClasses(currentPage);

    // Trigger resize event after classes change
    setTimeout(() => {
      window.dispatchEvent(new Event("resize"));
    }, 100); // Small delay to let layout settle
  }
}, 60); */

function handleProseClasses(currentPage) {
  let isPostPage = currentPage.includes("/posts/") || currentPage == "/";
  let content = document.getElementById("content-section");
  if (isPostPage) {
    content.classList.add("prose");
  } else {
    content.classList.remove("prose");
  }
}

function handleRightSectionClasses(currentPage) {
  let isPostPage = currentPage.includes("/posts/") || currentPage == "/";
  let rightSection = document.getElementById("right-section");
  let content = document.getElementById("content-section");
  if (isPostPage) {
    rightSection.classList.add("lg:flex");
    content.classList.add("md:max-w-4xl");
  } else {
    rightSection.classList.remove("lg:flex");
    content.classList.remove("md:max-w-4xl");
  }
}
