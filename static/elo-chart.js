function initChessChart(chartData) {
  
  d3.select('#elo-chart').selectAll('*').remove();
  
  const numOfMatchesChart = 30;
  const data = chartData.slice(numOfMatchesChart - 1, chartData.length);
  
  if (!data || data.length === 0) {
    console.warn('No data available for chart');
    return;
  }
  
  const margin = { top: 20, right: 50, bottom: 30, left: 50 };
  const width = 900 - margin.left - margin.right;
  const height = 400 - margin.top - margin.bottom;

  const svg = d3.select('#elo-chart')
    .append('svg')
    .attr('width', '100%')
    .attr('height', height + margin.top + margin.bottom)
    .attr('viewBox', `0 0 ${width + margin.left + margin.right} ${height + margin.top + margin.bottom}`)
    .append('g')
    .attr('transform', `translate(${margin.left},${margin.top})`);

  data.forEach(d => {
    d.date = new Date(d.timestamp);
  });

  const minRating = d3.min(data, d => d.rating);
  const maxRating = d3.max(data, d => d.rating);
  const ratingPadding = (maxRating - minRating) * 0.1 || 100;

  const x = d3.scaleTime()
    .domain(d3.extent(data, d => d.date))
    .range([0, width]);

  const y = d3.scaleLinear()
    .domain([minRating - ratingPadding, maxRating + ratingPadding])
    .range([height, 0]);

  const line = d3.line()
    .x(d => x(d.date))
    .y(d => y(d.rating))
    .curve(d3.curveBasis);

  svg.append('g')
    .attr('class', 'grid')
    .selectAll('line')
    .data(y.ticks(8))
    .enter()
    .append('line')
    .attr('x1', 0)
    .attr('x2', width)
    .attr('y1', d => y(d))
    .attr('y2', d => y(d))
    .attr('stroke', '#374151')
    .attr('stroke-width', 1)
    .attr('stroke-dasharray', '2,2');

  // X Axis
  svg.append('g')
    .attr('transform', `translate(0,${height})`)
    .call(d3.axisBottom(x).ticks(6))
    .attr('color', '#9ca3af')
    .selectAll('text')
    .style('font-size', '12px');

  // Y Axis
  svg.append('g')
    .call(d3.axisLeft(y).ticks(8))
    .attr('color', '#9ca3af')
    .selectAll('text')
    .style('font-size', '12px');

  svg.append('path')
    .datum(data)
    .attr('fill', 'none')
    .attr('stroke', '#bc6c25')
    .attr('stroke-width', 3)
    .attr('d', line);

  svg.selectAll('.dot')
    .data(data)
    .enter()
    .append('circle')
    .attr('class', 'dot')
    .attr('cx', d => x(d.date))
    .attr('cy', d => y(d.rating))
    .attr('r', 3)
    .attr('fill', '#F58A07')
    .attr('stroke', '#F58A07')
    .attr('stroke-width', 2)
    .style('cursor', 'pointer')
    .on('mouseover', function(_event, d) {
      d3.select(this)
        .transition()
        .duration(200)
        .attr('r', 6);
      
      const tooltip = svg.append('g')
        .attr('class', 'tooltip')
        .attr('transform', `translate(${x(d.date)},${y(d.rating) - 20})`);
      
      tooltip.append('rect')
        .attr('x', -40)
        .attr('y', -25)
        .attr('width', 80)
        .attr('height', 40)
        .attr('fill', '#1f2937')
        .attr('stroke', '#F58A07')
        .attr('rx', 4);
      
      tooltip.append('text')
        .attr('text-anchor', 'middle')
        .attr('y', 2.5)
        .attr('fill', '#fff')
        .style('font-size', '24px')
        .text(d.rating);
    })
    .on('mouseout', function() {
      d3.select(this)
        .transition()
        .duration(200)
        .attr('r', 3);
      
      svg.selectAll('.tooltip').remove();
    });
}

document.addEventListener('htmx:afterSettle', function(evt) {
  if (evt.detail.target.id === 'chess-stats') {
    const chartContainer = document.getElementById('elo-chart');
    const dataset = JSON.parse(chartContainer.getAttribute("dataset"));
    if (chartContainer && dataset) {
      try {
        initChessChart(dataset);
      } catch (e) {
        console.error('Failed to parse chart data:', e);
      }
    }
  }
});
