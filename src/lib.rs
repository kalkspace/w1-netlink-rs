use std::io::Write;

use netlink_packet_core::{NetlinkDeserializable, NetlinkSerializable};

pub mod command;
pub mod message;

pub mod constants {
    pub const CONNECTOR_W1_IDX: u32 = 0x3;
    pub const CONNECTOR_W1_VAL: u32 = 0x1;

    pub const W1_SLAVE_ADD: u8 = 0;
    pub const W1_SLAVE_REMOVE: u8 = 1;
    pub const W1_MASTER_ADD: u8 = 2;
    pub const W1_MASTER_REMOVE: u8 = 3;
    pub const W1_MASTER_CMD: u8 = 4;
    pub const W1_SLAVE_CMD: u8 = 5;
    pub const W1_LIST_MASTERS: u8 = 6;

    pub const W1_CMD_READ: u8 = 0;
    pub const W1_CMD_WRITE: u8 = 1;
    pub const W1_CMD_SEARCH: u8 = 2;
    pub const W1_CMD_ALARM_SEARCH: u8 = 3;
    pub const W1_CMD_TOUCH: u8 = 4;
    pub const W1_CMD_RESET: u8 = 5;
    pub const W1_CMD_SLAVE_ADD: u8 = 6;
    pub const W1_CMD_SLAVE_REMOVE: u8 = 7;
    pub const W1_CMD_LIST_SLAVES: u8 = 8;
    pub const W1_CMD_MAX: u8 = 9;
}

pub trait NlConnectorType {
    fn idx() -> u32;
    fn val() -> u32;
}

pub struct NlConnectorMessage<T> {
    seq: u32,
    ack: u32,
    flags: u16,
    payload: T,
}

impl<T> NlConnectorMessage<T> {
    const HEADER_LEN: usize = 20;

    pub fn new(seq: u32, payload: T) -> Self {
        Self {
            seq,
            ack: 0,
            flags: 0,
            payload,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid value received: {0}")]
pub struct InvalidValue(u8);

#[derive(Debug, thiserror::Error)]
pub enum DeserializeError<E: std::error::Error> {
    #[error("Payload is missing bytes")]
    InvalidLength,

    #[error("Invalid connector index, expected {0}, got {1}")]
    UnexpectedIdx(u32, u32),

    #[error("Invalid connector value, expected {0}, got {1}")]
    UnexpectedVal(u32, u32),

    #[error(transparent)]
    Inner(#[from] E),
}

impl<T> NetlinkDeserializable for NlConnectorMessage<T>
where
    T: NetlinkDeserializable + NlConnectorType,
    T::Error: std::error::Error,
{
    type Error = DeserializeError<T::Error>;

    fn deserialize(
        header: &netlink_packet_core::NetlinkHeader,
        payload: &[u8],
    ) -> Result<Self, Self::Error> {
        if payload.len() < Self::HEADER_LEN {
            return Err(DeserializeError::InvalidLength);
        }

        let idx = u32::from_le_bytes(payload[0..4].try_into().unwrap());
        if idx != T::idx() {
            return Err(DeserializeError::UnexpectedIdx(T::idx(), idx));
        }
        let val = u32::from_le_bytes(payload[4..8].try_into().unwrap());
        if val != T::val() {
            return Err(DeserializeError::UnexpectedVal(T::val(), val));
        }

        let seq = u32::from_le_bytes(payload[8..12].try_into().unwrap());
        let ack = u32::from_le_bytes(payload[12..16].try_into().unwrap());

        let _len = u16::from_le_bytes(payload[16..18].try_into().unwrap());
        let flags = u16::from_le_bytes(payload[18..20].try_into().unwrap());

        let payload = T::deserialize(header, &payload[Self::HEADER_LEN..])?;

        Ok(Self {
            seq,
            ack,
            flags,
            payload,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SerializeError {}

impl<T> NetlinkSerializable for NlConnectorMessage<T>
where
    T: NetlinkSerializable + NlConnectorType,
{
    fn message_type(&self) -> u16 {
        netlink_sys::constants::NETLINK_CONNECTOR as u16
    }

    fn buffer_len(&self) -> usize {
        let inner_len = self.payload.buffer_len();
        inner_len + Self::HEADER_LEN
    }

    fn serialize(&self, mut buffer: &mut [u8]) {
        let len = self.buffer_len() as u32;
        let to_write: Vec<u8> = std::iter::empty()
            .chain(T::idx().to_le_bytes().iter())
            .chain(T::val().to_le_bytes().iter())
            .chain(self.seq.to_le_bytes().iter())
            .chain(self.ack.to_le_bytes().iter())
            .chain(len.to_le_bytes().iter())
            .chain(self.flags.to_le_bytes().iter())
            .cloned()
            .collect();
        buffer.write_all(&to_write).unwrap();

        self.payload.serialize(&mut buffer[Self::HEADER_LEN..])
    }
}
