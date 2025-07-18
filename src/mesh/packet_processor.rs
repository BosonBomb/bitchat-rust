use crate::bitchat_packet::BitchatMessage;
use super::protocol::MessageType;
use anyhow::Result;
use std::sync::{Arc, Mutex};

pub trait PacketProcessorDelegate: Send + Sync {
    fn handle_message(&self, message: &BitchatMessage);
}

pub struct PacketProcessor {
    my_peer_id: String,
    delegate: Option<Arc<Mutex<dyn PacketProcessorDelegate>>>,
}

impl PacketProcessor {
    pub fn new(my_peer_id: String) -> Self {
        PacketProcessor {
            my_peer_id,
            delegate: None,
        }
    }

    pub fn set_delegate(&mut self, delegate: Arc<Mutex<dyn PacketProcessorDelegate>>) {
        self.delegate = Some(delegate);
    }

    pub fn process_packet(&self, packet: &[u8], peer_id: &str) -> Result<()> {
        let message_type = packet[0];
        match message_type {
            t if t == MessageType::Message as u8 => {
                let message = BitchatMessage::from_binary_payload(&packet[1..])?;
                if let Some(delegate) = &self.delegate {
                    let delegate = delegate.lock().unwrap();
                    delegate.handle_message(&message);
                }
            }
            _ => {
                // TODO: Handle other message types
            }
        }
        Ok(())
    }

    pub fn shutdown(&mut self) {
        // No-op
    }
}
