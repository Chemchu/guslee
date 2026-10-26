const GRAPH_ROOT_COLOR = "#F58A07";
const GRAPH_FOLDER_COLORS = [
  "#4c9aff",
  "#2ebfa5",
  "#a77bdb",
  "#e5534b",
  "#e0b43a",
  "#4fbf67",
  "#22c3d6",
  "#d16ba5",
];
const GRAPH_DEPTH_KEY = "graph-depth";
const GRAPH_DIMMED = 0.12;

window.graphDepth = function () {
  try {
    const depth = Number(localStorage.getItem(GRAPH_DEPTH_KEY));
    return depth >= 1 && depth <= 3 ? depth : 1;
  } catch {
    return 1;
  }
};

const graphResizeObservers = new Map();

function renderGraph(targetContainerId) {
  const container = document.getElementById(targetContainerId);
  if (!container) {
    console.log("No graph container found");
    return;
  }

  // Use getAttribute instead of dataset for large JSON strings
  const nodes = JSON.parse(container.getAttribute("data-nodes") || "[]");
  const links = JSON.parse(container.getAttribute("data-edges") || "[]");
  const folders = JSON.parse(container.getAttribute("data-folders") || "[]");
  const isLocalGraph = targetContainerId === "graph-container";

  const topFolder = (d) => (d.file_path?.includes("/") ? d.file_path.split("/")[0] : null);
  const colorOf = (d) => {
    const folder = topFolder(d);
    if (!folder) return GRAPH_ROOT_COLOR;
    const index = Math.max(folders.indexOf(folder), 0);
    return GRAPH_FOLDER_COLORS[index % GRAPH_FOLDER_COLORS.length];
  };
  const isCurrent = (d) => isLocalGraph && d.id === 1;
  const radiusOf = (d) => (isCurrent(d) ? 11 : 8);

  const neighbours = new Map(nodes.map((n) => [n.id, new Set([n.id])]));
  for (const { source, target } of links) {
    neighbours.get(source)?.add(target);
    neighbours.get(target)?.add(source);
  }

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
    .attr("r", radiusOf)
    .attr("fill", colorOf)
    .attr("stroke", (d) => (isCurrent(d) ? "#DBDFE5" : null))
    .attr("stroke-width", (d) => (isCurrent(d) ? 2 : null))
    .style("cursor", "pointer")
    .call(drag(simulation))
    .on("click", handleNodeClickNavigation)
    .on("mouseover", (event, d) => focusNode(d))
    .on("mouseout", () => focusNode(null));

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

  function focusNode(focused) {
    const isNear = (d) => !focused || neighbours.get(focused.id)?.has(d.id);
    const touches = (l) => focused && (l.source.id === focused.id || l.target.id === focused.id);

    node
      .attr("opacity", (d) => (isNear(d) ? 1 : GRAPH_DIMMED))
      .attr("r", (d) => (focused && d.id === focused.id ? radiusOf(d) + 2 : radiusOf(d)));
    label.style("opacity", (d) => (isNear(d) ? 1 : GRAPH_DIMMED));
    link
      .attr("stroke", (l) => (touches(l) ? colorOf(focused) : "#999"))
      .attr("stroke-opacity", (l) => (!focused ? 0.6 : touches(l) ? 1 : GRAPH_DIMMED));
  }

  if (!isLocalGraph) addLegend();

  function addLegend() {
    const shown = [...new Set(nodes.map(topFolder))].sort((a, b) =>
      a === null ? -1 : b === null ? 1 : a.localeCompare(b),
    );
    if (shown.length < 2) return;

    container.style.position = "relative";
    const legend = document.createElement("ul");
    legend.className =
      "absolute top-2 left-2 flex flex-col gap-1 p-2 text-xs bg-background-color/80 border border-shade-color pointer-events-none";
    for (const folder of shown) {
      const item = document.createElement("li");
      item.className = "flex items-center gap-2";
      const dot = document.createElement("span");
      dot.className = "inline-block w-2.5 h-2.5 rounded-full";
      dot.style.background = colorOf({ file_path: folder ? folder + "/" : "" });
      item.append(dot, folder ?? (container.dataset.rootLabel || "garden entry"));
      legend.append(item);
    }
    container.append(legend);
  }

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
    if (!newWidth || !newHeight) return;

    // Update SVG dimensions
    svg
      .attr("width", newWidth)
      .attr("height", newHeight)
      .attr("viewBox", `0 0 ${newWidth} ${newHeight}`);

    // Update force center
    simulation.force("center", d3.forceCenter(newWidth / 2, newHeight / 2));
    simulation.alpha(0.3).restart();
  }

  graphResizeObservers.get(targetContainerId)?.disconnect();
  let resizeFrame = null;
  const observer = new ResizeObserver(() => {
    if (resizeFrame) return;
    resizeFrame = requestAnimationFrame(() => {
      resizeFrame = null;
      handleResize();
    });
  });
  observer.observe(container);
  graphResizeObservers.set(targetContainerId, observer);

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
      const url = "/posts/" + d.file_path.replace(/\.md$/, "");

      if (typeof htmx !== "undefined") {
        htmx.ajax("GET", url, {
          target: "#main-section",
          swap: "innerHTML transition:true",
          push: "true",
        });
      } else {
        // Fallback to regular navigation
        window.location.href = url;
      }
    }
  }
}

document.addEventListener("click", function (event) {
  const button = event.target.closest?.("[data-graph-depth]");
  const container = document.getElementById("graph-container");
  if (!button || !container) return;

  try {
    localStorage.setItem(GRAPH_DEPTH_KEY, button.dataset.graphDepth);
  } catch {}
  htmx.ajax(
    "GET",
    `/graph/${encodeURI(container.dataset.postPath)}?depth=${button.dataset.graphDepth}`,
    { target: "#upper-right-section", swap: "innerHTML" },
  );
});

document.addEventListener("htmx:after:settle", function (evt) {
  // Once the graph container loads, we can init the actual graph
  if (evt.target.id === "upper-right-section") {
    renderGraph("graph-container");
  }

  if (evt.target.id === "garden-view-section") {
    renderGraph("garden-view-content");
  }
});
