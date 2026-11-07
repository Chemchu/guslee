const GRAPH_ROOT_COLOR = "#F58A07";
const GRAPH_GOLDEN_FRACTION = 0.381966;
const GRAPH_ROOT_HUES = { from: 15, to: 55 };
const GRAPH_FIRST_FOLDER_HUE = 210;

function graphFolderColor(index) {
  const span = 360 - (GRAPH_ROOT_HUES.to - GRAPH_ROOT_HUES.from);
  const phase = GRAPH_FIRST_FOLDER_HUE - GRAPH_ROOT_HUES.to;
  const hue = GRAPH_ROOT_HUES.to + ((phase + index * span * GRAPH_GOLDEN_FRACTION) % span);
  return `hsl(${Math.round(hue % 360)}, 65%, 62%)`;
}
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

  const parentFolder = (d) => {
    const slash = d.file_path?.lastIndexOf("/") ?? -1;
    return slash > 0 ? d.file_path.slice(0, slash) : null;
  };
  const folderColor = (folder) =>
    folder === null ? GRAPH_ROOT_COLOR : graphFolderColor(Math.max(folders.indexOf(folder), 0));
  const colorOf = (d) => folderColor(parentFolder(d));
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
  container.style.position = "relative";
  container.style.overflow = "hidden";

  const svg = d3
    .select("#" + targetContainerId)
    .append("svg")
    .attr("width", width)
    .attr("height", height)
    .attr("viewBox", `0 0 ${width} ${height}`)
    .attr("preserveAspectRatio", "xMidYMid meet")
    .style("position", "absolute")
    .style("inset", "0");

  const g = svg.append("g"); // Zoom functionality

  let userMoved = false;
  const zoom = d3
    .zoom()
    .scaleExtent([0.1, 4])
    .on("zoom", (event) => {
      if (event.sourceEvent) userMoved = true;
      g.attr("transform", event.transform);
    });
  svg.call(zoom);

  const FIT_PADDING = 16;

  function fitToView(force) {
    const box = g.node().getBBox();
    const { width: viewWidth, height: viewHeight } = getDimensions();
    if (!box.width || !box.height || !viewWidth || !viewHeight) return;

    const fits =
      box.width + FIT_PADDING * 2 <= viewWidth && box.height + FIT_PADDING * 2 <= viewHeight;
    if (fits && !force) return;

    const scale = Math.min(
      1,
      (viewWidth - FIT_PADDING * 2) / box.width,
      (viewHeight - FIT_PADDING * 2) / box.height,
    );
    const transform = d3.zoomIdentity
      .translate(viewWidth / 2, viewHeight / 2)
      .scale(scale)
      .translate(-(box.x + box.width / 2), -(box.y + box.height / 2));
    svg.transition().duration(400).call(zoom.transform, transform);
  }

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

  if (!isLocalGraph) {
    addLegend();
    addRecenterButton();
    simulation.on("end", () => {
      if (!userMoved) fitToView(false);
    });
  }

  function addRecenterButton() {
    const label = container.dataset.recenterLabel || "Re-center graph";
    const button = document.createElement("button");
    button.type = "button";
    button.title = label;
    button.setAttribute("aria-label", label);
    button.className =
      "absolute bottom-2 right-2 p-2 border border-shade-color bg-background-color/80 text-zinc-400 hover:text-primary-color hover:border-primary-color transition-colors cursor-pointer";
    button.innerHTML =
      '<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5"><path stroke-linecap="round" stroke-linejoin="round" d="M7.5 3.75H6A2.25 2.25 0 0 0 3.75 6v1.5M16.5 3.75H18A2.25 2.25 0 0 1 20.25 6v1.5m0 9V18A2.25 2.25 0 0 1 18 20.25h-1.5m-9 0H6A2.25 2.25 0 0 1 3.75 18v-1.5M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" /></svg>';
    button.addEventListener("click", () => fitToView(true));
    container.append(button);
  }

  function addLegend() {
    const shown = [...new Set(nodes.map(parentFolder))].sort((a, b) =>
      a === null ? -1 : b === null ? 1 : a.localeCompare(b),
    );
    if (shown.length < 2) return;

    const nameOf = (folder) => folder.slice(folder.lastIndexOf("/") + 1);
    const nameCounts = new Map();
    for (const folder of shown) {
      if (folder !== null) nameCounts.set(nameOf(folder), (nameCounts.get(nameOf(folder)) ?? 0) + 1);
    }
    const labelOf = (folder) => {
      if (folder === null) return container.dataset.rootLabel || "garden entry";
      return nameCounts.get(nameOf(folder)) > 1 ? folder : nameOf(folder);
    };

    const legend = document.createElement("ul");
    legend.className =
      "absolute top-2 left-2 flex flex-col gap-1 p-2 max-w-[60%] max-h-[45%] overflow-y-auto text-xs bg-background-color/80 border border-shade-color";
    for (const folder of shown) {
      const item = document.createElement("li");
      item.className = "flex items-center gap-2";
      const dot = document.createElement("span");
      dot.className = "inline-block shrink-0 w-2.5 h-2.5 rounded-full";
      dot.style.background = folderColor(folder);
      const name = document.createElement("span");
      name.className = "truncate";
      name.textContent = labelOf(folder);
      if (folder !== null) name.title = folder;
      item.append(dot, name);
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
      userMoved = true;
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
