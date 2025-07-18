mod bitchat_packet;
mod ui;
mod mesh;

use mesh::service::BluetoothMeshService;
use std::panic;
use tokio::sync::mpsc;
use crate::bitchat_packet::BitchatMessage;

#[tokio::main]
async fn main() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
        original_hook(panic_info);
    }));

    let (tx, rx) = mpsc::channel::<BitchatMessage>(100);
    let mesh_service = BluetoothMeshService::new(tx);
    BluetoothMeshService::start(mesh_service.clone()).await.unwrap();

    let result = ui::run_ui(rx).await;

    BluetoothMeshService::stop(mesh_service.clone()).unwrap();

    if let Err(e) = result {
        println!("Error: {}", e);
    }
}
