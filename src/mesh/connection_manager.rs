use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use std::time::Duration;
use tokio::time;
use anyhow::Result;
use std::sync::{Arc, Mutex};

pub trait BluetoothConnectionManagerDelegate: Send + Sync {
    fn on_packet_received(&self, packet: &[u8], peer_id: &str);
}

pub struct BluetoothConnectionManager {
    delegate: Option<Arc<Mutex<dyn BluetoothConnectionManagerDelegate>>>,
}

impl BluetoothConnectionManager {
    pub fn new() -> Self {
        BluetoothConnectionManager { delegate: None }
    }

    pub fn set_delegate(&mut self, delegate: Arc<Mutex<dyn BluetoothConnectionManagerDelegate>>) {
        self.delegate = Some(delegate);
    }

    pub async fn start_services(&self) -> Result<()> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        let central = adapters.into_iter().next().unwrap();

        central.start_scan(ScanFilter::default()).await?;
        time::sleep(Duration::from_secs(2)).await;

        for p in central.peripherals().await? {
            println!("Connecting to peripheral {:?}", p.properties().await?);
            if p.connect().await.is_ok() {
                println!("Connected to peripheral");
            }
        }
        Ok(())
    }

    pub fn stop_services(&self) {
        // TODO: Stop scanning and disconnect from peripherals
    }

    pub fn broadcast_packet(&self, packet: &[u8]) {
        // TODO: Implement packet broadcasting
    }
}
