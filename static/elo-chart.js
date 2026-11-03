function initChessChart(chartData) {
  d3.select("#elo-chart").selectAll("*").remove();

  const numOfMatchesChart = 30;
  const data = chartData.slice(numOfMatchesChart - 1, chartData.length);

  if (!data || data.length === 0) {
    console.warn("No data available for chart");
    return;
  }

  const container = document.getElementById("elo-chart");
  const outerWidth = container.clientWidth || 900;
  const narrow = outerWidth < 640;
  const margin = narrow
    ? { top: 20, right: 12, bottom: 30, left: 44 }
    : { top: 20, right: 50, bottom: 30, left: 50 };
  const width = outerWidth - margin.left - margin.right;
  const height = (narrow ? 240 : 400) - margin.top - margin.bottom;
  container.dataset.renderedWidth = outerWidth;

  const svg = d3
    .select("#elo-chart")
    .append("svg")
    .attr("width", "100%")
    .attr("height", height + margin.top + margin.bottom)
    .attr(
      "viewBox",
      `0 0 ${width + margin.left + margin.right} ${height + margin.top + margin.bottom}`,
    )
    .append("g")
    .attr("transform", `translate(${margin.left},${margin.top})`);

  data.forEach((d) => {
    d.date = new Date(d.timestamp);
  });

  const minRating = d3.min(data, (d) => d.rating);
  const maxRating = d3.max(data, (d) => d.rating);
  const ratingPadding = (maxRating - minRating) * 0.1 || 100;

  const x = d3
    .scaleTime()
    .domain(d3.extent(data, (d) => d.date))
    .range([0, width]);

  const y = d3
    .scaleLinear()
    .domain([minRating - ratingPadding, maxRating + ratingPadding])
    .range([height, 0]);

  const line = d3
    .line()
    .x((d) => x(d.date))
    .y((d) => y(d.rating))
    .curve(d3.curveBasis);

  svg
    .append("g")
    .attr("class", "grid")
    .selectAll("line")
    .data(y.ticks(narrow ? 5 : 8))
    .enter()
    .append("line")
    .attr("x1", 0)
    .attr("x2", width)
    .attr("y1", (d) => y(d))
    .attr("y2", (d) => y(d))
    .attr("stroke", "#374151")
    .attr("stroke-width", 1)
    .attr("stroke-dasharray", "2,2");

  // X Axis
  svg
    .append("g")
    .attr("transform", `translate(0,${height})`)
    .call(d3.axisBottom(x).ticks(narrow ? 3 : 6))
    .attr("color", "#9ca3af")
    .selectAll("text")
    .style("font-size", "12px");

  // Y Axis
  svg
    .append("g")
    .call(narrow ? d3.axisLeft(y).ticks(5, "d") : d3.axisLeft(y).ticks(8))
    .attr("color", "#9ca3af")
    .selectAll("text")
    .style("font-size", "12px");

  svg
    .append("path")
    .datum(data)
    .attr("fill", "none")
    .attr("stroke", "#bc6c25")
    .attr("stroke-width", narrow ? 2 : 3)
    .attr("d", line);

  svg
    .selectAll(".dot")
    .data(data)
    .enter()
    .append("circle")
    .attr("class", "dot")
    .attr("cx", (d) => x(d.date))
    .attr("cy", (d) => y(d.rating))
    .attr("r", 3)
    .attr("fill", "#F58A07")
    .attr("stroke", "#F58A07")
    .attr("stroke-width", 2)
    .style("cursor", "pointer")
    .on("mouseover", function (_event, d) {
      d3.select(this).transition().duration(200).attr("r", 6);

      svg.selectAll(".tooltip").remove();

      const tipWidth = narrow ? 56 : 80;
      const tipHeight = narrow ? 28 : 40;
      const tipX = Math.min(Math.max(x(d.date), tipWidth / 2), width - tipWidth / 2);
      const tipY = Math.max(y(d.rating) - 20, tipHeight - 5 - margin.top);

      const tooltip = svg
        .append("g")
        .attr("class", "tooltip")
        .attr("pointer-events", "none")
        .attr("transform", `translate(${tipX},${tipY})`);

      tooltip
        .append("rect")
        .attr("x", -tipWidth / 2)
        .attr("y", -tipHeight + 15)
        .attr("width", tipWidth)
        .attr("height", tipHeight)
        .attr("fill", "#1f2937")
        .attr("stroke", "#F58A07")
        .attr("rx", 4);

      tooltip
        .append("text")
        .attr("text-anchor", "middle")
        .attr("y", narrow ? 6 : 2.5)
        .attr("fill", "#fff")
        .style("font-size", narrow ? "16px" : "24px")
        .text(d.rating);
    })
    .on("mouseout", function () {
      d3.select(this).transition().duration(200).attr("r", 3);

      svg.selectAll(".tooltip").remove();
    });
}

function tryInitChart() {
  const chartContainer = document.getElementById("elo-chart");

  if (chartContainer) {
    const datasetAttr = chartContainer.getAttribute("dataset");

    if (datasetAttr) {
      const hasSvg = chartContainer.querySelector("svg");

      if (!hasSvg) {
        try {
          const dataset = JSON.parse(datasetAttr);
          initChessChart(dataset);
        } catch (e) {
          console.error("Failed to parse chart data:", e);
        }
      }
    }
  }
}

document.addEventListener("DOMContentLoaded", function () {
  tryInitChart();

  const targetContainer = document.getElementById("main-section");

  if (!targetContainer) {
    console.error("main-section not found");
    return;
  }

  const observer = new MutationObserver(() => {
    tryInitChart();
  });

  observer.observe(targetContainer, {
    childList: true,
    subtree: true,
  });

  let resizeFrame = null;
  window.addEventListener("resize", function () {
    if (resizeFrame) return;
    resizeFrame = requestAnimationFrame(() => {
      resizeFrame = null;
      const chart = document.getElementById("elo-chart");
      if (!chart || !chart.querySelector("svg")) return;
      if (Math.abs(chart.clientWidth - Number(chart.dataset.renderedWidth)) < 2) return;
      chart.innerHTML = "";
      tryInitChart();
    });
  });
});
