document.addEventListener("DOMContentLoaded", () => {
  let opacityImg1 = 1;
  let opacityImg2 = 0;

  window.addEventListener("scroll", () => {
    const sections = 4;
    const scrollPercentage =
      window.scrollY /
      (document.documentElement.scrollHeight - window.innerHeight);

    const firstSectionPercentage = 1 / sections;
    const lastSectionPercentage = (sections - 1) / sections;

    if (scrollPercentage <= firstSectionPercentage) {
      opacityImg1 = 1 - scrollPercentage * sections;
      opacityImg2 = 0;
    } else if (scrollPercentage >= lastSectionPercentage) {
      opacityImg1 = 0;
      opacityImg2 = (scrollPercentage - lastSectionPercentage) * sections;
    } else if (
      scrollPercentage > firstSectionPercentage &&
      scrollPercentage < lastSectionPercentage
    ) {
      opacityImg1 = 0;
      opacityImg2 = 0;
    }

    // Update the opacity of the images directly
    const image1 = document.getElementById("image1");
    const image2 = document.getElementById("image2");

    if (image1) {
      image1.style.opacity = opacityImg1;
    }

    if (image2) {
      image2.style.opacity = opacityImg2;
    }
  });
});
