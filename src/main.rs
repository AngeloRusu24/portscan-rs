use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

fn known_services() -> HashMap<u16, &'static str> {
    let mut m = HashMap::new();
    m.insert(21, "FTP");
    m.insert(22, "SSH");
    m.insert(23, "Telnet");
    m.insert(25, "SMTP");
    m.insert(53, "DNS");
    m.insert(80, "HTTP");
    m.insert(443, "HTTPS");
    m.insert(3306, "MySQL");
    m.insert(5432, "PostgreSQL");
    m.insert(6379, "Redis");
    m.insert(6463, "Discord");
    m.insert(27036, "Steam");
    m
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let target = if args.len() > 1 {
        args[1].clone()
    } else {
        println!("Uso: portscan-rs <indirizzo>");
        println!("Esempio: portscan-rs 192.168.1.1");
        std::process::exit(1);
    };

    let services = known_services();

    println!("╔══════════════════════════════════════╗");
    println!("║        portscan-rs  v0.1.0           ║");
    println!("╚══════════════════════════════════════╝");
    println!("  Target : {}", target);
    println!("  Porte  : 1 - 65535");
    println!("  Timeout: 500ms\n");

    let mut handles = vec![];

    for port in 1u16..=65535 {
        let target = target.clone();
        let handle = tokio::spawn(async move {
            let addr: SocketAddr = format!("{}:{}", target, port).parse().unwrap();
            let result = timeout(Duration::from_millis(500), TcpStream::connect(&addr)).await;
            if result.is_ok_and(|r| r.is_ok()) {
                Some(port)
            } else {
                None
            }
        });
        handles.push(handle);
    }

    let mut open_ports = vec![];
    for handle in handles {
        if let Ok(Some(port)) = handle.await {
            open_ports.push(port);
        }
    }

    open_ports.sort();

    println!("  {:<8} {}", "PORTA", "SERVIZIO");
    println!("  {}", "─".repeat(30));
    for port in &open_ports {
        let service = services.get(port).unwrap_or(&"unknown");
        println!("  {:<8} {}", port, service);
    }

    println!("\n  {} porte aperte trovate.", open_ports.len());
}