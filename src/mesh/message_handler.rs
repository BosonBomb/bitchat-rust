use crate::bitchat_packet::{BitchatMessage, DeliveryAck, ReadReceipt};
use std::sync::{Arc, Mutex};

pub trait MessageHandlerDelegate: Send + Sync {
    fn on_message_received(&self, message: &BitchatMessage);
    fn on_delivery_ack_received(&self, ack: &DeliveryAck);
    fn on_read_receipt_received(&self, receipt: &ReadReceipt);
}

pub struct MessageHandler {
    my_peer_id: String,
    delegate: Option<Arc<Mutex<dyn MessageHandlerDelegate>>>,
}

impl MessageHandler {
    pub fn new(my_peer_id: String) -> Self {
        MessageHandler {
            my_peer_id,
            delegate: None,
        }
    }

    pub fn set_delegate(&mut self, delegate: Arc<Mutex<dyn MessageHandlerDelegate>>) {
        self.delegate = Some(delegate);
    }

    pub fn handle_message(&self, message: &BitchatMessage) {
        if let Some(delegate) = &self.delegate {
            let delegate = delegate.lock().unwrap();
            delegate.on_message_received(message);
        }
    }

    pub fn handle_delivery_ack(&self, ack: &DeliveryAck) {
        if let Some(delegate) = &self.delegate {
            let delegate = delegate.lock().unwrap();
            delegate.on_delivery_ack_received(ack);
        }
    }

    pub fn handle_read_receipt(&self, receipt: &ReadReceipt) {
        if let Some(delegate) = &self.delegate {
            let delegate = delegate.lock().unwrap();
            delegate.on_read_receipt_received(receipt);
        }
    }

    pub fn shutdown(&mut self) {
        // No-op
    }
}
