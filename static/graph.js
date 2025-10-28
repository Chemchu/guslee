function initializeGraph() {
    const container = document.getElementById('graph-container');
    if (!container) {
        console.log("No graph container found");
        return;
    }
    
    // Parse data from data attributes. This data comes from the maud (rust) /graph endpoint
    const nodes = JSON.parse(container.dataset.nodes || '[]');
    const links = JSON.parse(container.dataset.edges || '[]');
    const width = container.clientWidth;
    const height = container.clientHeight;
    
    // Clear any existing SVG (important for re-initialization)
    container.innerHTML = '';
    
    const svg = d3.select('#graph-container')
        .append('svg')
        .attr('width', width)
        .attr('height', height);
    
    const g = svg.append('g'); // Zoom functionality 
    
    svg.call(d3.zoom()
        .scaleExtent([0.1, 4])
        .on('zoom', (event) => {
            g.attr('transform', event.transform);
        }));
    
    // Obsidian-like physics
    const simulation = d3.forceSimulation(nodes)
        .force('link', d3.forceLink(links)
            .id(d => d.id)
            .distance(100))
        .force('charge', d3.forceManyBody()
            .strength(-300))
        .force('center', d3.forceCenter(width / 2, height / 2))
        .force('collision', d3.forceCollide().radius(30));
    
    const link = g.append('g')
        .selectAll('line')
        .data(links)
        .join('line')
        .attr('stroke', '#999')
        .attr('stroke-opacity', 0.6)
        .attr('stroke-width', 1);
    
    const node = g.append('g')
        .selectAll('circle')
        .data(nodes)
        .join('circle')
        .attr('r', 8)
        .attr('fill', '#F58A07')
        .style('cursor', 'pointer')
        .call(drag(simulation))
        .on('click', handleNodeClickNavigation)
        .on('mouseover', function() {
            d3.select(this)
                .attr('r', 10)
                .attr('fill', '#bc6c25');
        })
        .on('mouseout', function() {
            d3.select(this)
                .attr('r', 8)
                .attr('fill', '#F58A07');
        });
    
    const label = g.append('g')
        .selectAll('text')
        .data(nodes)
        .join('text')
        .text(d => d.label || d.id)
        .attr('font-size', 12)
        .attr('dx', 12)
        .attr('dy', 4)
        .style('fill', '#DBDFE5')
        .style('pointer-events', 'none')
        .style('user-select', 'none');
    
    simulation.on('tick', () => {
        link
            .attr('x1', d => d.source.x)
            .attr('y1', d => d.source.y)
            .attr('x2', d => d.target.x)
            .attr('y2', d => d.target.y);
        
        node
            .attr('cx', d => d.x)
            .attr('cy', d => d.y);
        
        label
            .attr('x', d => d.x)
            .attr('y', d => d.y);
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
        
        return d3.drag()
            .on('start', dragStarted)
            .on('drag', dragging)
            .on('end', dragEnded);
    }
    
    function handleNodeClickNavigation(event, d) {
        event.stopPropagation();
        
        if (d.file_path) {
            const url = '/' + d.file_path.replace(/\.md$/, '');
            
            // Use htmx for navigation if available
            if (typeof htmx !== 'undefined') {
                htmx.ajax('GET', url, {
                    target: '#content-section',
                    swap: 'innerHTML',
                    headers: {
                        'HX-Current-URL': window.location.origin + url
                    }
                }).then(() => {
                    window.history.pushState({}, '', url);
                    document.body.dispatchEvent(new CustomEvent('graphUpdate'));
                });
            } else {
                // Fallback to regular navigation
                window.location.href = url;
            }
        }
    }
}

document.addEventListener('htmx:afterSettle', function(evt) {
    if (evt.detail.target.id === 'content-section') {
        document.body.dispatchEvent(new CustomEvent('graphUpdate'));
    }
    if (evt.detail.target.id === 'graph-section') {
        initializeGraph();
    }
});
