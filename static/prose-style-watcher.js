let currentPage = location.href;
let nonProseUrls = ["/news", "/chess"];

document.addEventListener("DOMContentLoaded", (_event) => {
  handleProseClasses(currentPage);
});

setInterval(function(){
  if (currentPage != location.href){
    currentPage = location.href;
    handleProseClasses(currentPage);
  }
}, 60);

function handleProseClasses(currentPage) {
  let nonPostPage = nonProseUrls.find(url => currentPage.endsWith(url));
  let content = document.getElementById("content-section");
  if(nonPostPage) {
    content.classList.remove("prose");
  }
  else {
    content.classList.add("prose");
  }
}
