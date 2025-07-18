use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, TimeZone};
use uuid::Uuid;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write};
use anyhow::Result;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum DeliveryStatus {
    Sending,
    Sent,
    Delivered { to: String, at: DateTime<Utc> },
    Read { by: String, at: DateTime<Utc> },
    Failed { reason: String },
    PartiallyDelivered { reached: u32, total: u32 },
}

impl DeliveryStatus {
    pub fn get_display_text(&self) -> String {
        match self {
            DeliveryStatus::Sending => "Sending...".to_string(),
            DeliveryStatus::Sent => "Sent".to_string(),
            DeliveryStatus::Delivered { to, at: _ } => format!("Delivered to {}", to),
            DeliveryStatus::Read { by, at: _ } => format!("Read by {}", by),
            DeliveryStatus::Failed { reason } => format!("Failed: {}", reason),
            DeliveryStatus::PartiallyDelivered { reached, total } => {
                format!("Delivered to {}/{}", reached, total)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BitchatMessage {
    pub id: String,
    pub sender: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub is_relay: bool,
    pub original_sender: Option<String>,
    pub is_private: bool,
    pub recipient_nickname: Option<String>,
    pub sender_peer_id: Option<String>,
    pub mentions: Option<Vec<String>>,
    pub channel: Option<String>,
    pub encrypted_content: Option<Vec<u8>>,
    pub is_encrypted: bool,
    pub delivery_status: Option<DeliveryStatus>,
}

impl BitchatMessage {
    pub fn new(sender: String, content: String) -> Self {
        BitchatMessage {
            id: Uuid::new_v4().to_string(),
            sender,
            content,
            timestamp: Utc::now(),
            is_relay: false,
            original_sender: None,
            is_private: false,
            recipient_nickname: None,
            sender_peer_id: None,
            mentions: None,
            channel: None,
            encrypted_content: None,
            is_encrypted: false,
            delivery_status: Some(DeliveryStatus::Sending),
        }
    }

    pub fn to_binary_payload(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::with_capacity(4096);
        let mut flags: u8 = 0;

        if self.is_relay { flags |= 0x01; }
        if self.is_private { flags |= 0x02; }
        if self.original_sender.is_some() { flags |= 0x04; }
        if self.recipient_nickname.is_some() { flags |= 0x08; }
        if self.sender_peer_id.is_some() { flags |= 0x10; }
        if self.mentions.is_some() { flags |= 0x20; }
        if self.channel.is_some() { flags |= 0x40; }
        if self.is_encrypted { flags |= 0x80; }

        buffer.write_u8(flags)?;
        buffer.write_i64::<BigEndian>(self.timestamp.timestamp_millis())?;

        let id_bytes = self.id.as_bytes();
        buffer.write_u8(id_bytes.len() as u8)?;
        buffer.write_all(id_bytes)?;

        let sender_bytes = self.sender.as_bytes();
        buffer.write_u8(sender_bytes.len() as u8)?;
        buffer.write_all(sender_bytes)?;

        if self.is_encrypted {
            if let Some(encrypted_content) = &self.encrypted_content {
                buffer.write_u16::<BigEndian>(encrypted_content.len() as u16)?;
                buffer.write_all(encrypted_content)?;
            }
        } else {
            let content_bytes = self.content.as_bytes();
            buffer.write_u16::<BigEndian>(content_bytes.len() as u16)?;
            buffer.write_all(content_bytes)?;
        }

        if let Some(original_sender) = &self.original_sender {
            let bytes = original_sender.as_bytes();
            buffer.write_u8(bytes.len() as u8)?;
            buffer.write_all(bytes)?;
        }

        if let Some(recipient_nickname) = &self.recipient_nickname {
            let bytes = recipient_nickname.as_bytes();
            buffer.write_u8(bytes.len() as u8)?;
            buffer.write_all(bytes)?;
        }

        if let Some(sender_peer_id) = &self.sender_peer_id {
            let bytes = sender_peer_id.as_bytes();
            buffer.write_u8(bytes.len() as u8)?;
            buffer.write_all(bytes)?;
        }

        if let Some(mentions) = &self.mentions {
            buffer.write_u8(mentions.len() as u8)?;
            for mention in mentions {
                let bytes = mention.as_bytes();
                buffer.write_u8(bytes.len() as u8)?;
                buffer.write_all(bytes)?;
            }
        }

        if let Some(channel) = &self.channel {
            let bytes = channel.as_bytes();
            buffer.write_u8(bytes.len() as u8)?;
            buffer.write_all(bytes)?;
        }

        Ok(buffer)
    }

    pub fn from_binary_payload(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let flags = cursor.read_u8()?;
        let is_relay = (flags & 0x01) != 0;
        let is_private = (flags & 0x02) != 0;
        let has_original_sender = (flags & 0x04) != 0;
        let has_recipient_nickname = (flags & 0x08) != 0;
        let has_sender_peer_id = (flags & 0x10) != 0;
        let has_mentions = (flags & 0x20) != 0;
        let has_channel = (flags & 0x40) != 0;
        let is_encrypted = (flags & 0x80) != 0;

        let timestamp_millis = cursor.read_i64::<BigEndian>()?;
        let timestamp = Utc.timestamp_millis_opt(timestamp_millis).unwrap();

        let id_len = cursor.read_u8()? as usize;
        let mut id_bytes = vec![0; id_len];
        cursor.read_exact(&mut id_bytes)?;
        let id = String::from_utf8(id_bytes)?;

        let sender_len = cursor.read_u8()? as usize;
        let mut sender_bytes = vec![0; sender_len];
        cursor.read_exact(&mut sender_bytes)?;
        let sender = String::from_utf8(sender_bytes)?;

        let content_len = cursor.read_u16::<BigEndian>()? as usize;
        let mut content = String::new();
        let mut encrypted_content = None;

        if is_encrypted {
            let mut encrypted_bytes = vec![0; content_len];
            cursor.read_exact(&mut encrypted_bytes)?;
            encrypted_content = Some(encrypted_bytes);
        } else {
            let mut content_bytes = vec![0; content_len];
            cursor.read_exact(&mut content_bytes)?;
            content = String::from_utf8(content_bytes)?;
        }

        let original_sender = if has_original_sender {
            let len = cursor.read_u8()? as usize;
            let mut bytes = vec![0; len];
            cursor.read_exact(&mut bytes)?;
            Some(String::from_utf8(bytes)?)
        } else {
            None
        };

        let recipient_nickname = if has_recipient_nickname {
            let len = cursor.read_u8()? as usize;
            let mut bytes = vec![0; len];
            cursor.read_exact(&mut bytes)?;
            Some(String::from_utf8(bytes)?)
        } else {
            None
        };

        let sender_peer_id = if has_sender_peer_id {
            let len = cursor.read_u8()? as usize;
            let mut bytes = vec![0; len];
            cursor.read_exact(&mut bytes)?;
            Some(String::from_utf8(bytes)?)
        } else {
            None
        };

        let mentions = if has_mentions {
            let count = cursor.read_u8()? as usize;
            let mut mention_list = Vec::with_capacity(count);
            for _ in 0..count {
                let len = cursor.read_u8()? as usize;
                let mut bytes = vec![0; len];
                cursor.read_exact(&mut bytes)?;
                mention_list.push(String::from_utf8(bytes)?);
            }
            Some(mention_list)
        } else {
            None
        };

        let channel = if has_channel {
            let len = cursor.read_u8()? as usize;
            let mut bytes = vec![0; len];
            cursor.read_exact(&mut bytes)?;
            Some(String::from_utf8(bytes)?)
        } else {
            None
        };

        Ok(BitchatMessage {
            id,
            sender,
            content,
            timestamp,
            is_relay,
            original_sender,
            is_private,
            recipient_nickname,
            sender_peer_id,
            mentions,
            channel,
            encrypted_content,
            is_encrypted,
            delivery_status: None,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeliveryAck {
    pub original_message_id: String,
    pub ack_id: String,
    pub recipient_id: String,
    pub recipient_nickname: String,
    pub timestamp: DateTime<Utc>,
    pub hop_count: u8,
}

impl DeliveryAck {
    pub fn new(original_message_id: String, recipient_id: String, recipient_nickname: String, hop_count: u8) -> Self {
        DeliveryAck {
            original_message_id,
            ack_id: Uuid::new_v4().to_string(),
            recipient_id,
            recipient_nickname,
            timestamp: Utc::now(),
            hop_count,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadReceipt {
    pub original_message_id: String,
    pub receipt_id: String,
    pub reader_id: String,
    pub reader_nickname: String,
    pub timestamp: DateTime<Utc>,
}

impl ReadReceipt {
    pub fn new(original_message_id: String, reader_id: String, reader_nickname: String) -> Self {
        ReadReceipt {
            original_message_id,
            receipt_id: Uuid::new_v4().to_string(),
            reader_id,
            reader_nickname,
            timestamp: Utc::now(),
        }
    }
}
