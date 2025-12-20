function renderGraph(targetContainerId) {
  const container = document.getElementById(targetContainerId);
  if (!container) {
    console.log("No graph container found");
    return;
  }

  // Parse data from data attributes. This data comes from the maud (rust) /graph endpoint
  const nodes = JSON.parse(container.dataset.nodes || "[]");
  const links = JSON.parse(container.dataset.edges || "[]");

  // Function to get current dimensions
  function getDimensions() {
    return {
      width: container.clientWidth,
      height: container.clientHeight,
    };
  }

  let { width, height } = getDimensions();

  // Clear any existing SVG (important for re-initialization)
  container.innerHTML = "";

  const svg = d3
    .select("#" + targetContainerId)
    .append("svg")
    .attr("width", width)
    .attr("height", height)
    .attr("viewBox", `0 0 ${width} ${height}`)
    .attr("preserveAspectRatio", "xMidYMid meet");

  const g = svg.append("g"); // Zoom functionality

  svg.call(
    d3
      .zoom()
      .scaleExtent([0.1, 4])
      .on("zoom", (event) => {
        g.attr("transform", event.transform);
      }),
  );

  // Obsidian-like physics
  const simulation = d3
    .forceSimulation(nodes)
    .force(
      "link",
      d3
        .forceLink(links)
        .id((d) => d.id)
        .distance(50),
    )
    .force("charge", d3.forceManyBody().strength(-150))
    .force("center", d3.forceCenter(width / 2, height / 2))
    .force("collision", d3.forceCollide().radius(15));

  const link = g
    .append("g")
    .selectAll("line")
    .data(links)
    .join("line")
    .attr("stroke", "#999")
    .attr("stroke-opacity", 0.6)
    .attr("stroke-width", 1);

  const node = g
    .append("g")
    .selectAll("circle")
    .data(nodes)
    .join("circle")
    .attr("r", 8)
    .attr("fill", "#F58A07")
    .style("cursor", "pointer")
    .call(drag(simulation))
    .on("click", handleNodeClickNavigation)
    .on("mouseover", function () {
      d3.select(this).attr("r", 10).attr("fill", "#bc6c25");
    })
    .on("mouseout", function () {
      d3.select(this).attr("r", 8).attr("fill", "#F58A07");
    });

  const label = g
    .append("g")
    .selectAll("text")
    .data(nodes)
    .join("text")
    .text((d) => d.label || d.id)
    .attr("font-size", 12)
    .attr("dx", 12)
    .attr("dy", 4)
    .style("fill", "#DBDFE5")
    .style("pointer-events", "none")
    .style("user-select", "none");

  simulation.on("tick", () => {
    link
      .attr("x1", (d) => d.source.x)
      .attr("y1", (d) => d.source.y)
      .attr("x2", (d) => d.target.x)
      .attr("y2", (d) => d.target.y);

    node.attr("cx", (d) => d.x).attr("cy", (d) => d.y);

    label.attr("x", (d) => d.x).attr("y", (d) => d.y);
  });

  // Resize handler
  function handleResize() {
    const { width: newWidth, height: newHeight } = getDimensions();

    // Update SVG dimensions
    svg
      .attr("width", newWidth)
      .attr("height", newHeight)
      .attr("viewBox", `0 0 ${newWidth} ${newHeight}`);

    // Update force center
    simulation.force("center", d3.forceCenter(newWidth / 2, newHeight / 2));
    simulation.alpha(0.3).restart();
  }

  // Debounce resize to avoid excessive recalculations
  let resizeTimer;
  window.addEventListener("resize", () => {
    clearTimeout(resizeTimer);
    resizeTimer = setTimeout(handleResize, 250);
  });

  function drag(simulation) {
    function dragStarted(event) {
      if (!event.active) simulation.alphaTarget(0.3).restart();
      event.subject.fx = event.subject.x;
      event.subject.fy = event.subject.y;
    }

    function dragging(event) {
      event.subject.fx = event.x;
      event.subject.fy = event.y;
    }

    function dragEnded(event) {
      if (!event.active) simulation.alphaTarget(0);
      event.subject.fx = null;
      event.subject.fy = null;
    }

    return d3
      .drag()
      .on("start", dragStarted)
      .on("drag", dragging)
      .on("end", dragEnded);
  }

  function handleNodeClickNavigation(event, d) {
    event.stopPropagation();

    if (d.file_path) {
      const url = "/posts/" + d.file_path.replace(/\.md$/, "") + "/page";

      if (typeof htmx !== "undefined") {
        if (document.startViewTransition) {
          document.startViewTransition(async () => {
            const response = await fetch(url, {
              headers: {
                "HX-Request": "true",
                "HX-Current-URL": window.location.origin + url,
              },
            });
            const html = await response.text();
            document.getElementById("main-section").innerHTML = html;
            window.history.pushState({}, "", url);
          });
        } else {
          htmx
            .ajax("GET", url, {
              target: "#main-section",
              swap: "innerHTML",
              headers: {
                "HX-Current-URL": window.location.origin + url,
              },
            })
            .then(() => {
              window.history.pushState({}, "", url);
            });
        }
      } else {
        // Fallback to regular navigation
        window.location.href = url;
      }
    }
  }
}

/* document.addEventListener("contentUpdated", function (_evt) {
  renderGraph("graph-container");
  renderGraph("garden-view-section");
}); */

document.addEventListener("htmx:afterSettle", function (evt) {
  // Once the graph container loades, we can init the actual graph
  if (evt.detail.target.id === "upper-right-section") {
    renderGraph("graph-container");
  }

  if (evt.detail.target.id === "garden-view-section") {
    renderGraph("garden-view-section");
  }
});
