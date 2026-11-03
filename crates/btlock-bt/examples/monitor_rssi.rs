//! Surveillance RSSI en ligne de commande (test BlueZ).
//!
//! Usage: cargo run -p btlock-bt --example monitor_rssi -- <ADRESSE_BT>

use std::env;
use std::time::Duration;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let Some(addr) = env::args().nth(1) else {
        eprintln!("Usage: monitor_rssi <ADRESSE>, ex. AA:BB:CC:DD:EE:FF");
        std::process::exit(1);
    };

    println!("BTlock — lecture RSSI toutes les 2 s pour {addr}");
    println!("(Téléphone/montre : Bluetooth activé, appareil visible / appairé si besoin)\n");

    loop {
        match btlock_bt::read_rssi(&addr).await {
            Ok(Some(rssi)) => println!("{rssi} dBm"),
            Ok(None) => println!("— (pas de mesure, appareil hors portée ou inconnu)"),
            Err(e) => eprintln!("Erreur : {e}"),
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
