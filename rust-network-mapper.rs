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
    let args = Args::parse();
    let start_ip: Ipv4Addr = args.start_ip.parse()?;
    let end_ip: Ipv4Addr = args.end_ip.parse()?;
    let timeout_duration = Duration::from_secs(1);
    let max_concurrent_scans = 100;

    let semaphore = Arc::new(Semaphore::new(max_concurrent_scans));
    let mut tasks = Vec::new();

    for ip in u32::from(start_ip)..=u32::from(end_ip) {
        let ip = Ipv4Addr::from(ip);
        let semaphore = Arc::clone(&semaphore);

        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            scan_host(ip, timeout_duration).await
        });

        tasks.push(task);
    }

    let mut results = Vec::new();
    for task in tasks {
        if let Some(result) = task.await? {
            results.push(result);
        }
    }

    println!("{}", serde_json::to_string_pretty(&results)?);

    // Generate interactive network topology visualization
    generate_interactive_visualization(&results, &args.output_file)?;

    Ok(())
}

async fn scan_host(ip: Ipv4Addr, timeout_duration: Duration) -> Option<ScanResult> {
    let ports_to_scan = vec![21, 22, 80, 443, 3306, 5432];
    let mut open_ports = Vec::new();

    for &port in &ports_to_scan {
        if let Ok(Ok(_)) = timeout(
            timeout_duration,
            TcpStream::connect((ip, port))
        ).await {
            open_ports.push(port);
        }
    }

    if !open_ports.is_empty() {
        Some(ScanResult {
            ip: ip.to_string(),
            open_ports,
            os_guess: guess_os(&open_ports),
            subnet: format!("{}.{}.{}.0/24", ip.octets()[0], ip.octets()[1], ip.octets()[2]),
        })
    } else {
        None
    }
}

fn guess_os(open_ports: &[u16]) -> String {
    if open_ports.contains(&22) && open_ports.contains(&80) {
        "Linux".to_string()
    } else if open_ports.contains(&3389) {
        "Windows".to_string()
    } else {
        "Unknown".to_string()
    }
}

fn generate_interactive_visualization(results: &[ScanResult], output_file: &str) -> Result<(), std::io::Error> {
    let mut file = File::create(output_file)?;
    
    // Group devices by subnet
    let mut subnets: HashMap<String, Vec<&ScanResult>> = HashMap::new();
    for result in results {
        subnets.entry(result.subnet.clone()).or_default().push(result);
    }

    // Prepare data for D3.js
    let nodes: Vec<HashMap<String, String>> = results.iter()
        .map(|r| {
            let mut node = HashMap::new();
            node.insert("id".to_string(), r.ip.clone());
            node.insert("os".to_string(), r.os_guess.clone());
            node.insert("subnet".to_string(), r.subnet.clone());
            node
        })
        .collect();

    let links: Vec<HashMap<String, String>> = subnets.iter()
        .flat_map(|(subnet, devices)| {
            devices.iter().map(move |device| {
                let mut link = HashMap::new();
                link.insert("source".to_string(), subnet.clone());
                link.insert("target".to_string(), device.ip.clone());
                link
            })
        })
        .collect();

    let data = json!({
        "nodes": nodes,
        "links": links
    });

    // HTML template with embedded D3.js visualization
    let html_content = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="utf-8">
            <title>Network Topology Visualization</title>
            <script src="https://d3js.org/d3.v7.min.js"></script>
            <style>
                body {{ font-family: Arial, sans-serif; }}
                .node {{ stroke: #fff; stroke-width: 1.5px; }}
                .link {{ stroke: #999; stroke-opacity: 0.6; }}
            </style>
        </head>
        <body>
            <h1>Network Topology Visualization</h1>
            <div id="network-graph"></div>
            <script>
                const data = {};

                const width = 960;
                const height = 600;

                const color = d3.scaleOrdinal(d3.schemeCategory10);

                const simulation = d3.forceSimulation(data.nodes)
                    .force("link", d3.forceLink(data.links).id(d => d.id))
                    .force("charge", d3.forceManyBody())
                    .force("center", d3.forceCenter(width / 2, height / 2));

                const svg = d3.select("#network-graph")
                    .append("svg")
                    .attr("width", width)
                    .attr("height", height);

                const link = svg.append("g")
                    .selectAll("line")
                    .data(data.links)
                    .join("line")
                    .attr("class", "link");

                const node = svg.append("g")
                    .selectAll("circle")
                    .data(data.nodes)
                    .join("circle")
                    .attr("class", "node")
                    .attr("r", 5)
                    .attr("fill", d => color(d.os))
                    .call(drag(simulation));

                node.append("title")
                    .text(d => `IP: ${d.id}\nOS: ${d.os}\nSubnet: ${d.subnet}`);

                simulation.on("tick", () => {{
                    link
                        .attr("x1", d => d.source.x)
                        .attr("y1", d => d.source.y)
                        .attr("x2", d => d.target.x)
                        .attr("y2", d => d.target.y);

                    node
                        .attr("cx", d => d.x)
                        .attr("cy", d => d.y);
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
            </script>
        </body>
        </html>
        "#,
        serde_json::to_string(&data)?
    );

    file.write_all(html_content.as_bytes())?;

    Ok(())
}
