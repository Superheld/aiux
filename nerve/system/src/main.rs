// nerve-system: System-Monitor Nerve.
//
// Meldet sich per Self-Registration an, dann periodisch CPU/RAM/Disk/Temperatur.
// Erster Nerve mit dem neuen Registration-Protokoll.

use std::time::Duration;

use sysinfo::System;
use serde_json::json;

use nerve_shared::mqtt;
use nerve_shared::registration::{self, NerveInfo};

const INTERVAL_SECS: u64 = 60;

#[tokio::main]
async fn main() {
    let host = std::env::var("MQTT_HOST").unwrap_or_else(|_| "localhost".into());
    let port: u16 = std::env::var("MQTT_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(1883);

    // MQTT verbinden
    let (client, mut eventloop) = mqtt::connect("nerve-system", &host, port);

    // EventLoop pumpen
    let pump_client = client.clone();
    tokio::spawn(async move {
        let _ = pump_client; // Client am Leben halten
        loop {
            match eventloop.poll().await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("MQTT-Fehler: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    });

    // Kurz warten bis MQTT-Verbindung steht
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Self-Registration
    let info = NerveInfo {
        name: "system-monitor".into(),
        version: "0.1.0".into(),
        description: "Ueberwacht CPU, RAM, Disk, Temperatur".into(),
        source: "nerve/system".into(),
        channels: vec![
            "aiux/nerve/system/stats".into(),
        ],
        home: Some("nerves/system-monitor".into()),
    };

    match registration::register(&client, &info).await {
        Ok(_) => println!("nerve-system v0.1.0 registriert"),
        Err(e) => {
            eprintln!("Registration fehlgeschlagen: {}", e);
            std::process::exit(1);
        }
    }

    // System-Info sammeln
    let mut sys = System::new_all();

    println!("Sende Stats alle {}s. Ctrl+C zum Beenden.", INTERVAL_SECS);

    loop {
        tokio::time::sleep(Duration::from_secs(INTERVAL_SECS)).await;

        sys.refresh_all();

        let stats = collect_stats(&sys);
        let payload = mqtt::build_message("nerve/system", "stats", stats);

        if let Err(e) = mqtt::publish(&client, "aiux/nerve/system/stats", &payload).await {
            eprintln!("Publish fehlgeschlagen: {}", e);
        }
    }
}

/// System-Stats als JSON sammeln.
fn collect_stats(sys: &System) -> serde_json::Value {
    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();
    let cpu_usage: f32 = sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>()
        / sys.cpus().len().max(1) as f32;

    let mut stats = json!({
        "cpu_percent": (cpu_usage * 10.0).round() / 10.0,
        "mem_total_mb": total_mem / 1_048_576,
        "mem_used_mb": used_mem / 1_048_576,
        "mem_percent": if total_mem > 0 {
            ((used_mem as f64 / total_mem as f64) * 100.0 * 10.0).round() / 10.0
        } else { 0.0 },
    });

    // Disk: Root-Partition
    let disks = sysinfo::Disks::new_with_refreshed_list();
    if let Some(root) = disks.iter().find(|d| d.mount_point() == std::path::Path::new("/")) {
        let total = root.total_space();
        let avail = root.available_space();
        stats["disk_total_gb"] = json!((total as f64 / 1_073_741_824.0 * 10.0).round() / 10.0);
        stats["disk_avail_gb"] = json!((avail as f64 / 1_073_741_824.0 * 10.0).round() / 10.0);
    }

    // Temperatur (wenn verfuegbar, z.B. Raspi)
    let components = sysinfo::Components::new_with_refreshed_list();
    if let Some(temp) = components.iter().next() {
        if let Some(t) = temp.temperature() {
            stats["temp_celsius"] = json!((t * 10.0).round() / 10.0);
        }
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_stats_hat_pflichtfelder() {
        let sys = System::new_all();
        let stats = collect_stats(&sys);

        assert!(stats["cpu_percent"].is_number());
        assert!(stats["mem_total_mb"].is_number());
        assert!(stats["mem_used_mb"].is_number());
        assert!(stats["mem_percent"].is_number());
    }
}
