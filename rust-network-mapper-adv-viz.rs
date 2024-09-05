use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::time::{timeout, Duration};
use serde::{Serialize, Deserialize};
use serde_json::json;
use clap::Parser;
use std::fs::File;
use std::io::Write;
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = "192.168.1.1")]
    start_ip: String,
    #[clap(short, long, default_value = "192.168.1.254")]
    end_ip: String,
    #[clap(short, long, default_value = "network_topology.html")]
    output_file: String,
}

#[derive(Serialize, Deserialize)]
struct ScanResult {
    ip: String,
    open_ports: Vec<u16>,
    os_guess: String,
    subnet: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ... (previous main function code remains the same)
}

async fn scan_host(ip: Ipv4Addr, timeout_duration: Duration) -> Option<ScanResult> {
    // ... (previous scan_host function code remains the same)
}

fn guess_os(open_ports: &[u16]) -> String {
    // ... (previous guess_os function code remains the same)
}

fn generate_interactive_visualization(results: &[ScanResult], output_file: &str) -> Result<(), std::io::Error> {
    let mut file = File::create(output_file)?;
    
    // Group devices by subnet
    let mut subnets: HashMap<String, Vec<&ScanResult>> = HashMap::new();
    for result in results {
        subnets.entry(result.subnet.clone()).or_default().push(result);
    }

    // Prepare data for D3.js
    let nodes: Vec<HashMap<String, serde_json::Value>> = results.iter()
        .map(|r| {
            json!({
                "id": r.ip,
                "os": r.os_guess,
                "subnet": r.subnet,
                "ports": r.open_ports,
                "type": "device"
            })
        })
        .collect();

    let subnet_nodes: Vec<HashMap<String, serde_json::Value>> = subnets.keys()
        .map(|subnet| {
            json!({
                "id": subnet,
                "type": "subnet"
            })
        })
        .collect();

    let links: Vec<HashMap<String, String>> = results.iter()
        .map(|device| {
            HashMap::from([
                ("source".to_string(), device.subnet.clone()),
                ("target".to_string(), device.ip.clone())
            ])
        })
        .collect();

    let data = json!({
        "nodes": [nodes, subnet_nodes].concat(),
        "links": links
    });

    // HTML template with embedded D3.js visualization
    let html_content = format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Network Topology Visualization</title>
            <script src="https://d3js.org/d3.v7.min.js"></script>
            <style>
                body {{
                    font-family: Arial, sans-serif;
                    margin: 0;
                    padding: 0;
                    display: flex;
                    flex-direction: column;
                    height: 100vh;
                    background-color: #f0f0f0;
                }}
                #header {{
                    background-color: #333;
                    color: white;
                    padding: 1rem;
                    text-align: center;
                }}
                #network-graph {{
                    flex-grow: 1;
                    background-color: white;
                    border-radius: 8px;
                    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
                    margin: 1rem;
                    overflow: hidden;
                }}
                .node {{
                    stroke: #fff;
                    stroke-width: 1.5px;
                }}
                .link {{
                    stroke: #999;
                    stroke-opacity: 0.6;
                }}
                .subnet {{
                    fill: #f9f9f9;
                    stroke: #666;
                    stroke-width: 2px;
                    stroke-dasharray: 5, 5;
                }}
                #tooltip {{
                    position: absolute;
                    background-color: rgba(0, 0, 0, 0.8);
                    color: white;
                    padding: 10px;
                    border-radius: 4px;
                    font-size: 12px;
                    pointer-events: none;
                    opacity: 0;
                    transition: opacity 0.3s;
                }}
                #legend {{
                    position: absolute;
                    top: 20px;
                    right: 20px;
                    background-color: rgba(255, 255, 255, 0.8);
                    padding: 10px;
                    border-radius: 4px;
                    font-size: 12px;
                }}
                .legend-item {{
                    display: flex;
                    align-items: center;
                    margin-bottom: 5px;
                }}
                .legend-color {{
                    width: 20px;
                    height: 20px;
                    margin-right: 5px;
                    border-radius: 50%;
                }}
            </style>
        </head>
        <body>
            <div id="header">
                <h1>Network Topology Visualization</h1>
            </div>
            <div id="network-graph"></div>
            <div id="tooltip"></div>
            <div id="legend"></div>
            <script>
                const data = {};

                const width = window.innerWidth - 40;
                const height = window.innerHeight - 100;

                const color = d3.scaleOrdinal()
                    .domain(["Linux", "Windows", "Unknown"])
                    .range(["#4CAF50", "#2196F3", "#FFC107"]);

                const simulation = d3.forceSimulation(data.nodes)
                    .force("link", d3.forceLink(data.links).id(d => d.id).distance(100))
                    .force("charge", d3.forceManyBody().strength(-300))
                    .force("center", d3.forceCenter(width / 2, height / 2))
                    .force("collision", d3.forceCollide().radius(30));

                const svg = d3.select("#network-graph")
                    .append("svg")
                    .attr("viewBox", [0, 0, width, height])
                    .attr("width", "100%")
                    .attr("height", "100%");

                const link = svg.append("g")
                    .selectAll("line")
                    .data(data.links)
                    .join("line")
                    .attr("class", "link");

                const node = svg.append("g")
                    .selectAll("circle")
                    .data(data.nodes)
                    .join("circle")
                    .attr("class", d => d.type === "subnet" ? "node subnet" : "node")
                    .attr("r", d => d.type === "subnet" ? 30 : 10)
                    .attr("fill", d => d.type === "subnet" ? "none" : color(d.os))
                    .call(drag(simulation));

                const label = svg.append("g")
                    .selectAll("text")
                    .data(data.nodes)
                    .join("text")
                    .text(d => d.type === "subnet" ? d.id : "")
                    .attr("font-size", "10px")
                    .attr("text-anchor", "middle")
                    .attr("dy", ".35em");

                const tooltip = d3.select("#tooltip");

                node.on("mouseover", (event, d) => {{
                    if (d.type === "device") {{
                        tooltip.style("opacity", 1)
                            .html(`IP: ${d.id}<br>OS: ${d.os}<br>Subnet: ${d.subnet}<br>Ports: ${d.ports.join(", ")}`)
                            .style("left", (event.pageX + 10) + "px")
                            .style("top", (event.pageY - 10) + "px");
                    }}
                }})
                .on("mouseout", () => {{
                    tooltip.style("opacity", 0);
                }});

                simulation.on("tick", () => {{
                    link
                        .attr("x1", d => d.source.x)
                        .attr("y1", d => d.source.y)
                        .attr("x2", d => d.target.x)
                        .attr("y2", d => d.target.y);

                    node
                        .attr("cx", d => d.x)
                        .attr("cy", d => d.y);

                    label
                        .attr("x", d => d.x)
                        .attr("y", d => d.y);
                }});

                function drag(simulation) {{
                    function dragstarted(event) {{
                        if (!event.active) simulation.alphaTarget(0.3).restart();
                        event.subject.fx = event.subject.x;
                        event.subject.fy = event.subject.y;
                    }}

                    function dragged(event) {{
                        event.subject.fx = event.x;
                        event.subject.fy = event.y;
                    }}

                    function dragended(event) {{
                        if (!event.active) simulation.alphaTarget(0);
                        event.subject.fx = null;
                        event.subject.fy = null;
                    }}

                    return d3.drag()
                        .on("start", dragstarted)
                        .on("drag", dragged)
                        .on("end", dragended);
                }}

                // Create legend
                const legend = d3.select("#legend");
                const legendItems = [
                    {{ name: "Linux", color: color("Linux") }},
                    {{ name: "Windows", color: color("Windows") }},
                    {{ name: "Unknown", color: color("Unknown") }},
                    {{ name: "Subnet", color: "none" }}
                ];

                legend.selectAll(".legend-item")
                    .data(legendItems)
                    .join("div")
                    .attr("class", "legend-item")
                    .html(d => `
                        <div class="legend-color" style="background-color: ${d.color}; ${d.name === 'Subnet' ? 'border: 2px dashed #666;' : ''}"></div>
                        <span>${d.name}</span>
                    `);
            </script>
        </body>
        </html>
        "#,
        serde_json::to_string(&data)?
    );

    file.write_all(html_content.as_bytes())?;

    Ok(())
}
