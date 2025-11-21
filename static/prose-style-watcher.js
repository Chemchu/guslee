let currentPage = window.location.pathname;
document.addEventListener("DOMContentLoaded", (_event) => {
  handleProseClasses(currentPage);
});

setInterval(function () {
  if (currentPage != window.location.pathname) {
    currentPage = window.location.pathname;
    handleProseClasses(currentPage);
  }
}, 60);

function handleProseClasses(currentPage) {
  let isPostPage = currentPage.includes("/posts/") || currentPage == "/";
  let content = document.getElementById("content-section");
  if (isPostPage) {
    content.classList.add("prose");
  } else {
    content.classList.remove("prose");
  }
}
