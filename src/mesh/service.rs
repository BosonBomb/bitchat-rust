use anyhow::Result;
use uuid::Uuid;
use super::peer_manager::PeerManager;
use super::fragment_manager::FragmentManager;
use super::security_manager::SecurityManager;
use super::message_handler::MessageHandler;
use super::connection_manager::BluetoothConnectionManager;
use super::message_handler::MessageHandlerDelegate;
use super::peer_manager::PeerManagerDelegate;
use super::connection_manager::BluetoothConnectionManagerDelegate;
use super::packet_processor::{PacketProcessor, PacketProcessorDelegate};
use crate::bitchat_packet::{BitchatMessage, DeliveryAck, ReadReceipt};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct BluetoothMeshService {
    my_peer_id: String,
    is_active: bool,
    peer_manager: Arc<Mutex<PeerManager>>,
    fragment_manager: Arc<Mutex<FragmentManager>>,
    security_manager: Arc<Mutex<SecurityManager>>,
    message_handler: Arc<Mutex<MessageHandler>>,
    connection_manager: Arc<Mutex<BluetoothConnectionManager>>,
    packet_processor: Arc<Mutex<PacketProcessor>>,
    message_tx: mpsc::Sender<BitchatMessage>,
}

impl BluetoothMeshService {
    pub fn new(message_tx: mpsc::Sender<BitchatMessage>) -> Arc<Mutex<Self>> {
        let my_peer_id = Uuid::new_v4().to_string();
        let service = Arc::new(Mutex::new(BluetoothMeshService {
            my_peer_id: my_peer_id.clone(),
            is_active: false,
            peer_manager: Arc::new(Mutex::new(PeerManager::new())),
            fragment_manager: Arc::new(Mutex::new(FragmentManager::new())),
            security_manager: Arc::new(Mutex::new(SecurityManager::new())),
            message_handler: Arc::new(Mutex::new(MessageHandler::new(my_peer_id.clone()))),
            connection_manager: Arc::new(Mutex::new(BluetoothConnectionManager::new())),
            packet_processor: Arc::new(Mutex::new(PacketProcessor::new(my_peer_id))),
            message_tx,
        }));

        let s = service.clone() as Arc<Mutex<dyn PeerManagerDelegate>>;
        service.lock().unwrap().peer_manager.lock().unwrap().set_delegate(s);
        let s = service.clone() as Arc<Mutex<dyn MessageHandlerDelegate>>;
        service.lock().unwrap().message_handler.lock().unwrap().set_delegate(s);
        let s = service.clone() as Arc<Mutex<dyn BluetoothConnectionManagerDelegate>>;
        service.lock().unwrap().connection_manager.lock().unwrap().set_delegate(s);
        let s = service.clone() as Arc<Mutex<dyn PacketProcessorDelegate>>;
        service.lock().unwrap().packet_processor.lock().unwrap().set_delegate(s);

        service
    }

    pub async fn start(service: Arc<Mutex<Self>>) -> Result<()> {
        let mut s = service.lock().unwrap();
        if s.is_active {
            return Ok(());
        }
        s.is_active = true;
        let connection_manager = s.connection_manager.clone();
        drop(s);

        connection_manager.lock().unwrap().start_services().await?;
        Ok(())
    }

    pub fn stop(service: Arc<Mutex<Self>>) -> Result<()> {
        let mut s = service.lock().unwrap();
        if !s.is_active {
            return Ok(());
        }
        s.is_active = false;
        s.peer_manager.lock().unwrap().shutdown();
        s.fragment_manager.lock().unwrap().shutdown();
        s.security_manager.lock().unwrap().shutdown();
        s.message_handler.lock().unwrap().shutdown();
        s.connection_manager.lock().unwrap().stop_services();
        s.packet_processor.lock().unwrap().shutdown();
        Ok(())
    }
}

impl MessageHandlerDelegate for BluetoothMeshService {
    fn on_message_received(&self, message: &BitchatMessage) {
        self.message_tx.blocking_send(message.clone()).unwrap();
    }

    fn on_delivery_ack_received(&self, ack: &DeliveryAck) {
        // TODO: Handle delivery ack
    }

    fn on_read_receipt_received(&self, receipt: &ReadReceipt) {
        // TODO: Handle read receipt
    }
}

impl PeerManagerDelegate for BluetoothMeshService {
    fn on_peer_connected(&self, nickname: &str) {
        // TODO: Handle peer connected
    }

    fn on_peer_disconnected(&self, nickname: &str) {
        // TODO: Handle peer disconnected
    }

    fn on_peer_list_updated(&self, peer_ids: &[String]) {
        // TODO: Handle peer list updated
    }
}

impl BluetoothConnectionManagerDelegate for BluetoothMeshService {
    fn on_packet_received(&self, packet: &[u8], peer_id: &str) {
        self.packet_processor.lock().unwrap().process_packet(packet, peer_id).unwrap();
    }
}

impl PacketProcessorDelegate for BluetoothMeshService {
    fn handle_message(&self, message: &BitchatMessage) {
        self.message_handler.lock().unwrap().handle_message(message);
    }
}
