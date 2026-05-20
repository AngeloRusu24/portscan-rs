// ============================================================
// portscan-rs — Fast async TCP port scanner
// ============================================================
// Dipendenze:
//   tokio     — runtime async per la concorrenza
//   colored   — colori nel terminale
//   indicatif — barra di progresso
// ============================================================

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

// ============================================================
// SEZIONE COLORI — modifica qui per cambiare i colori
// ============================================================
// Colori disponibili: red, green, blue, cyan, magenta,
//                    yellow, white, black, purple, bright_*
//
// Stili disponibili: .bold(), .dimmed(), .italic()
// ============================================================

fn print_banner(target: &str, port_start: u16, port_end: u16) {
    // Colore del banner principale
    println!("{}", "╔══════════════════════════════════════╗".cyan());
    println!("{}", "║        portscan-rs  v0.2.0           ║".cyan());
    println!("{}", "╚══════════════════════════════════════╝".cyan());

    // Colore delle etichette e valori
    println!("  {} {}", "Target :".bold(), target.green());
    println!("  {} {}-{}", "Porte  :".bold(), port_start, port_end);
    println!("  {} 500ms\n", "Timeout:".bold());
}

fn print_results(open_ports: &[u16], services: &HashMap<u16, &'static str>) {
    if open_ports.is_empty() {
        // Colore messaggio nessuna porta trovata
        println!("{}", "  Nessuna porta aperta trovata.".red());
        return;
    }

    // Colore intestazione tabella
    println!("  {:<8} {}", "PORTA".bold(), "SERVIZIO".bold());
    println!("  {}", "─".repeat(30).dimmed());

    for port in open_ports {
        let service = services.get(port).unwrap_or(&"unknown");

        // Colore numero porta e nome servizio
        println!(
            "  {:<8} {}",
            port.to_string().green().bold(), // <- colore porta aperta
            service.cyan()                   // <- colore nome servizio
        );
    }

    // Colore messaggio finale
    println!(
        "\n  {}",
        format!("✓ {} porte aperte trovate.", open_ports.len())
            .green()
            .bold()
    );
}

// ============================================================
// SEZIONE SERVIZI — aggiungi qui nuovi servizi conosciuti
// ============================================================
// Formato: m.insert(NUMERO_PORTA, "NOME SERVIZIO");
// Riferimento completo: https://en.wikipedia.org/wiki/List_of_TCP_and_UDP_port_numbers
// ============================================================

fn known_services() -> HashMap<u16, &'static str> {
    let mut m = HashMap::new();

    // Protocolli di rete base
    m.insert(21, "FTP");
    m.insert(22, "SSH");
    m.insert(23, "Telnet");
    m.insert(25, "SMTP");
    m.insert(53, "DNS");
    m.insert(80, "HTTP");
    m.insert(110, "POP3");
    m.insert(143, "IMAP");
    m.insert(443, "HTTPS");
    m.insert(465, "SMTPS");
    m.insert(993, "IMAPS");

    // Database
    m.insert(3306, "MySQL");
    m.insert(5432, "PostgreSQL");
    m.insert(6379, "Redis");
    m.insert(27017, "MongoDB");

    // Applicazioni desktop
    m.insert(1716, "KDE Connect");
    m.insert(6463, "Discord");
    m.insert(27036, "Steam Remote Play");
    m.insert(27060, "Steam");
    m.insert(57621, "Spotify");

    // Sviluppo
    m.insert(3000, "Dev server (Node/React)");
    m.insert(4200, "Dev server (Angular)");
    m.insert(5173, "Dev server (Vite)");
    m.insert(8080, "HTTP alternativo");
    m.insert(8443, "HTTPS alternativo");

    m
}

// ============================================================
// SEZIONE BARRA DI PROGRESSO — modifica qui lo stile
// ============================================================
// Template: {spinner} {bar} {pos}/{len} {eta}
// Caratteri barra: "█▓░" oppure "=>-" oppure "##-"
// Colori spinner/barra: .cyan .green .yellow .red ecc.
// ============================================================

fn build_progress_bar(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.cyan} [{bar:40.cyan/blue}] {pos}/{len} porte ({eta})")
            .unwrap()
            .progress_chars("█▓░"), // <- cambia i caratteri della barra qui
    );
    pb
}

// ============================================================
// MAIN — logica principale del programma
// ============================================================

#[tokio::main]
async fn main() {
    // Leggi gli argomenti da riga di comando
    let args: Vec<String> = std::env::args().collect();

    let target = if args.len() > 1 {
        args[1].clone()
    } else {
        println!("{}", "Uso: portscan-rs <indirizzo> [porta-inizio] [porta-fine]".yellow());
        println!("{}", "Esempio: portscan-rs 192.168.1.1 1 1024".yellow());
        std::process::exit(1);
    };

    // Range di porte — default 1-65535 se non specificato
    let port_start: u16 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let port_end: u16 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(65535);

    let services = known_services();

    // Stampa il banner iniziale
    print_banner(&target, port_start, port_end);

    // Crea la barra di progresso
    let total = (port_end - port_start + 1) as u64;
    let pb = build_progress_bar(total);

    // --------------------------------------------------------
    // Avvia la scansione — ogni porta viene testata in parallelo
    // con tokio::spawn (un task async per ogni porta)
    // --------------------------------------------------------
    let mut handles = vec![];

    for port in port_start..=port_end {
        let target = target.clone();
        let pb = pb.clone();

        let handle = tokio::spawn(async move {
            let addr: SocketAddr = format!("{}:{}", target, port)
                .parse()
                .unwrap();

            // Prova a connettersi con timeout di 500ms
            let result = timeout(
                Duration::from_millis(500),
                TcpStream::connect(&addr),
            )
            .await;

            pb.inc(1); // aggiorna la barra di progresso

            // Restituisce la porta se aperta, None se chiusa
            if result.is_ok_and(|r| r.is_ok()) {
                Some(port)
            } else {
                None
            }
        });

        handles.push(handle);
    }

    // Raccoglie i risultati da tutti i task
    let mut open_ports = vec![];
    for handle in handles {
        if let Ok(Some(port)) = handle.await {
            open_ports.push(port);
        }
    }

    // Rimuovi la barra di progresso e mostra i risultati
    pb.finish_and_clear();
    open_ports.sort();
    print_results(&open_ports, &services);
}