use netlink_packet_core::{NetlinkDeserializable, NetlinkPayload, NetlinkSerializable};
use std::mem;

use self::raw::CnMsg;
use super::{Deserializable, Serializable};

mod raw {
    use safe_transmute::TriviallyTransmutable;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct CnMsg {
        pub idx: u32,
        pub val: u32,
        pub seq: u32,
        pub ack: u32,
        /// Kernel docs:
        /// > Its length field is equal to size of the attached data
        pub len: u16,
        pub flags: u16,
    }

    /// Safety: Struct uses repr(C) and should we well aligned for byte slices
    unsafe impl TriviallyTransmutable for CnMsg {}
}

pub trait NlConnectorType {
    fn idx() -> u32;
    fn val() -> u32;
}

#[derive(Debug, Clone)]
pub struct NlConnectorHeader {
    seq: u32,
    ack: u32,
    flags: u16,
}

#[derive(Debug, Clone)]
pub struct NlConnectorMessage<T> {
    header: NlConnectorHeader,
    payload: Vec<T>,
}

impl<T> NlConnectorMessage<T> {
    pub const HEADER_LEN: usize = mem::size_of::<CnMsg>();

    pub fn new(seq: u32, payload: impl IntoIterator<Item = T>) -> Self {
        let payload = payload.into_iter().collect();
        Self {
            header: NlConnectorHeader {
                seq,
                ack: 0,
                flags: 0,
            },
            payload,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeserializeError<E: std::error::Error> {
    #[error("Invalid Netlink message type")]
    InvalidMessageType,

    #[error("Invalid payload: {0}")]
    InvalidHeader(safe_transmute::Error<'static, u8, CnMsg>),

    #[error("Payload length does not match header field")]
    InvalidPayloadLength,

    #[error("Invalid connector index, expected {0}, got {1}")]
    UnexpectedIdx(u32, u32),

    #[error("Invalid connector value, expected {0}, got {1}")]
    UnexpectedVal(u32, u32),

    #[error(transparent)]
    Inner(#[from] E),
}

impl<T> NetlinkDeserializable for NlConnectorMessage<T>
where
    T: Deserializable<Header = NlConnectorHeader> + NlConnectorType,
    T::Error: std::error::Error,
{
    type Error = DeserializeError<T::Error>;

    fn deserialize(
        header: &netlink_packet_core::NetlinkHeader,
        payload: &[u8],
    ) -> Result<Self, Self::Error> {
        if header.message_type as isize != netlink_sys::constants::NETLINK_CONNECTOR {
            return Err(Self::Error::InvalidMessageType);
        }

        let (header, payload_bytes) = payload.split_at(mem::size_of::<CnMsg>());
        let CnMsg {
            idx,
            val,
            seq,
            ack,
            len,
            flags,
        } = safe_transmute::transmute_one_pedantic(header)
            .map_err(|e| Self::Error::InvalidHeader(e.without_src()))?;

        if len as usize != payload.len() {
            return Err(Self::Error::InvalidPayloadLength);
        }
        if idx != T::idx() {
            return Err(Self::Error::UnexpectedIdx(T::idx(), idx));
        }
        if val != T::val() {
            return Err(Self::Error::UnexpectedVal(T::val(), val));
        }

        let header = NlConnectorHeader { seq, ack, flags };
        let mut payload = Vec::new();
        let mut cursor = 0;
        while cursor < payload.len() {
            let (item, n) = T::deserialize(&header, &payload_bytes[cursor..])?;
            payload.push(item);
            cursor += n;
        }

        Ok(Self { header, payload })
    }
}

impl<T> NetlinkSerializable for NlConnectorMessage<T>
where
    T: Serializable + NlConnectorType,
{
    fn message_type(&self) -> u16 {
        netlink_sys::constants::NETLINK_CONNECTOR as u16
    }

    fn buffer_len(&self) -> usize {
        let inner_len: usize = self.payload.iter().map(Serializable::buffer_len).sum();
        inner_len + Self::HEADER_LEN
    }

    fn serialize(&self, buffer: &mut [u8]) {
        let len = (buffer.len() - Self::HEADER_LEN) as u16;
        let NlConnectorHeader { seq, ack, flags } = self.header;
        let raw = CnMsg {
            idx: T::idx(),
            val: T::val(),
            seq,
            ack,
            len,
            flags,
        };
        let msg = safe_transmute::transmute_one_to_bytes(&raw);

        debug_assert_eq!(Self::HEADER_LEN, mem::size_of::<CnMsg>());
        buffer[0..Self::HEADER_LEN].copy_from_slice(msg);

        let mut cursor = 0;
        for item in &self.payload {
            let len = item.buffer_len();
            item.serialize(&mut buffer[cursor..cursor + len]);
            cursor += len;
        }
    }
}

impl<T> From<NlConnectorMessage<T>> for NetlinkPayload<NlConnectorMessage<T>> {
    fn from(msg: NlConnectorMessage<T>) -> Self {
        Self::InnerMessage(msg)
    }
}
