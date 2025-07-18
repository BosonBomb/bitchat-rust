#[repr(u8)]
pub enum MessageType {
    Message = 0x01,
    Announce = 0x02,
    Leave = 0x03,
    KeyExchange = 0x04,
    Fragment = 0x05,
    DeliveryAck = 0x06,
    ReadReceipt = 0x07,
}
