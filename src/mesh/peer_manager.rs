use std::collections::HashMap;
use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};

pub struct Peer {
    pub nickname: String,
    pub last_seen: DateTime<Utc>,
    pub rssi: i32,
    pub announced_to: bool,
}

pub trait PeerManagerDelegate: Send + Sync {
    fn on_peer_connected(&self, nickname: &str);
    fn on_peer_disconnected(&self, nickname: &str);
    fn on_peer_list_updated(&self, peer_ids: &[String]);
}

pub struct PeerManager {
    peers: HashMap<String, Peer>,
    delegate: Option<Arc<Mutex<dyn PeerManagerDelegate>>>,
}

impl PeerManager {
    pub fn new() -> Self {
        PeerManager {
            peers: HashMap::new(),
            delegate: None,
        }
    }

    pub fn set_delegate(&mut self, delegate: Arc<Mutex<dyn PeerManagerDelegate>>) {
        self.delegate = Some(delegate);
    }

    pub fn add_or_update_peer(&mut self, peer_id: &str, nickname: &str) -> bool {
        let is_new = !self.peers.contains_key(peer_id);
        let peer = self.peers.entry(peer_id.to_string()).or_insert_with(|| {
            Peer {
                nickname: nickname.to_string(),
                last_seen: Utc::now(),
                rssi: 0,
                announced_to: false,
            }
        });

        peer.nickname = nickname.to_string();
        peer.last_seen = Utc::now();

        if is_new {
            if let Some(delegate) = &self.delegate {
                let delegate = delegate.lock().unwrap();
                delegate.on_peer_connected(nickname);
                delegate.on_peer_list_updated(&self.get_all_peer_ids());
            }
        }
        is_new
    }

    pub fn remove_peer(&mut self, peer_id: &str) {
        if let Some(peer) = self.peers.remove(peer_id) {
            if let Some(delegate) = &self.delegate {
                let delegate = delegate.lock().unwrap();
                delegate.on_peer_disconnected(&peer.nickname);
                delegate.on_peer_list_updated(&self.get_all_peer_ids());
            }
        }
    }

    pub fn update_peer_last_seen(&mut self, peer_id: &str) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.last_seen = Utc::now();
        }
    }

    pub fn update_peer_rssi(&mut self, peer_id: &str, rssi: i32) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.rssi = rssi;
        }
    }

    pub fn mark_peer_as_announced_to(&mut self, peer_id: &str) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.announced_to = true;
        }
    }

    pub fn has_announced_to_peer(&self, peer_id: &str) -> bool {
        self.peers.get(peer_id).map_or(false, |p| p.announced_to)
    }

    pub fn get_peer_nickname(&self, peer_id: &str) -> Option<String> {
        self.peers.get(peer_id).map(|p| p.nickname.clone())
    }

    pub fn get_all_peer_nicknames(&self) -> HashMap<String, String> {
        self.peers
            .iter()
            .map(|(id, peer)| (id.clone(), peer.nickname.clone()))
            .collect()
    }

    pub fn get_all_peer_rssi(&self) -> HashMap<String, i32> {
        self.peers
            .iter()
            .map(|(id, peer)| (id.clone(), peer.rssi))
            .collect()
    }

    pub fn get_all_peer_ids(&self) -> Vec<String> {
        self.peers.keys().cloned().collect()
    }

    pub fn get_active_peer_count(&self) -> usize {
        self.peers.len()
    }

    pub fn is_peer_active(&self, peer_id: &str) -> bool {
        self.peers.contains_key(peer_id)
    }

    pub fn shutdown(&mut self) {
        self.peers.clear();
    }
}
