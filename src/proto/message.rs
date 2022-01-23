use std::mem;

use self::raw::W1NetlinkMsg;
use super::{
    connector::{NlConnectorHeader, NlConnectorType},
    Deserializable, InvalidValue, Serializable,
};

mod raw {
    //! Taken from https://www.kernel.org/doc/Documentation/w1/w1.netlink

    use safe_transmute::TriviallyTransmutable;

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
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct W1NetlinkMsg {
        /// Message type. See also [constants].
        pub r#type: u8,
        /// Error indication from kernel
        pub status: u8,
        /// Size of data attached to this header data
        pub len: u16,
        /// Master or slave ID
        pub id: [u8; 8],
    }

    unsafe impl TriviallyTransmutable for W1NetlinkMsg {}
}

/// See also [raw::constants].
#[derive(Debug, Clone, Copy)]
pub enum W1MessageType {
    SlaveAdd,
    SlaveRemove,
    MasterAdd,
    MasterRemove,
    MasterCmd,
    SlaveCmd,
    ListMasters,
}

impl TryFrom<u8> for W1MessageType {
    type Error = InvalidValue;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use self::raw::constants::*;
        let t = match value {
            W1_SLAVE_ADD => Self::SlaveAdd,
            W1_SLAVE_REMOVE => Self::SlaveRemove,
            W1_MASTER_ADD => Self::MasterAdd,
            W1_MASTER_REMOVE => Self::MasterRemove,
            W1_MASTER_CMD => Self::MasterCmd,
            W1_SLAVE_CMD => Self::SlaveCmd,
            W1_LIST_MASTERS => Self::ListMasters,
            v => return Err(InvalidValue(v)),
        };
        Ok(t)
    }
}

impl From<W1MessageType> for u8 {
    fn from(mt: W1MessageType) -> Self {
        use self::raw::constants::*;
        match mt {
            W1MessageType::SlaveAdd => W1_SLAVE_ADD,
            W1MessageType::SlaveRemove => W1_SLAVE_REMOVE,
            W1MessageType::MasterAdd => W1_MASTER_ADD,
            W1MessageType::MasterRemove => W1_MASTER_REMOVE,
            W1MessageType::MasterCmd => W1_MASTER_CMD,
            W1MessageType::SlaveCmd => W1_SLAVE_CMD,
            W1MessageType::ListMasters => W1_LIST_MASTERS,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid length for Master ID: {0}")]
pub struct InvalidLength(usize);

/// Mainly needed to read replies to the [W1MessageType::ListMasters] message.
#[derive(Debug, Clone)]
pub struct MasterId(u32);

impl Deserializable for MasterId {
    type Header = W1MessageHeader;
    type Error = InvalidLength;

    fn deserialize(_header: &Self::Header, payload: &[u8]) -> Result<(Self, usize), Self::Error> {
        if payload.len() < 4 {
            return Err(InvalidLength(payload.len()));
        }
        let val = u32::from_le_bytes(payload[0..4].try_into().unwrap());
        Ok((Self(val), 4))
    }
}

#[derive(Debug, Clone)]
pub struct TargetId([u8; 8]);

impl TargetId {
    pub fn master_id(id: u32) -> Self {
        let mut inner = [0u8; 8];
        (inner[0..4]).copy_from_slice(&id.to_le_bytes());
        Self(inner)
    }

    pub fn slave_id(id: [u8; 8]) -> Self {
        Self(id)
    }

    pub fn as_master_id(&self) -> u32 {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.0[0..4]);
        u32::from_le_bytes(bytes)
    }

    pub fn as_slave_id(&self) -> [u8; 8] {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct W1MessageHeader {
    /// Message type
    msg_type: W1MessageType,
    /// Error indication from kernel
    status: u8,
    /// Master or slave ID
    id: TargetId,
}

#[derive(Debug, Clone)]
pub struct W1NetlinkMessage<T> {
    header: W1MessageHeader,
    cmds: Vec<T>,
}

impl<T> W1NetlinkMessage<T> {
    pub const HEADER_LEN: usize = mem::size_of::<W1NetlinkMsg>();

    pub fn new(
        msg_type: W1MessageType,
        target_id: TargetId,
        cmds: impl IntoIterator<Item = T>,
    ) -> Self {
        let cmds = cmds.into_iter().collect();
        Self {
            header: W1MessageHeader {
                msg_type,
                status: 0,
                id: target_id,
            },
            cmds,
        }
    }
}

impl<T> NlConnectorType for W1NetlinkMessage<T> {
    fn idx() -> u32 {
        raw::constants::CONNECTOR_W1_IDX
    }

    fn val() -> u32 {
        raw::constants::CONNECTOR_W1_VAL
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeserializeError<E: std::error::Error> {
    #[error("Invalid header payload: {0}")]
    InvalidHeader(safe_transmute::Error<'static, u8, W1NetlinkMsg>),

    #[error("Invalid message type: {0}")]
    InvalidMessageType(InvalidValue),

    #[error("Payload length does not match header")]
    InvalidPayloadLength,

    #[error(transparent)]
    Inner(#[from] E),
}

impl<T> Deserializable for W1NetlinkMessage<T>
where
    T: Deserializable<Header = W1MessageHeader>,
{
    type Header = NlConnectorHeader;

    type Error = DeserializeError<T::Error>;

    fn deserialize(_header: &Self::Header, payload: &[u8]) -> Result<(Self, usize), Self::Error> {
        let (header, payload) = payload.split_at(Self::HEADER_LEN);
        let W1NetlinkMsg {
            r#type,
            status,
            len,
            id,
        } = safe_transmute::transmute_one_pedantic(header)
            .map_err(|e| Self::Error::InvalidHeader(e.without_src()))?;

        let msg_type = r#type.try_into().map_err(Self::Error::InvalidMessageType)?;
        let header = W1MessageHeader {
            msg_type,
            status,
            id: TargetId(id),
        };

        let len = len as usize;
        let mut cmds = Vec::new();
        let mut cursor = 0;
        while cursor < len {
            let (item, read) = T::deserialize(&header, &payload[cursor..len])?;
            cmds.push(item);
            cursor += read;
        }

        let read = len + Self::HEADER_LEN;
        Ok((Self { header, cmds }, read))
    }
}

impl<T> Serializable for W1NetlinkMessage<T>
where
    T: Serializable,
{
    fn buffer_len(&self) -> usize {
        let inner: usize = self.cmds.iter().map(Serializable::buffer_len).sum();
        inner + Self::HEADER_LEN
    }

    fn serialize(&self, buffer: &mut [u8]) {
        let len = (self.buffer_len() - Self::HEADER_LEN) as u16;
        let W1MessageHeader {
            msg_type,
            status,
            id,
        } = self.header.clone();
        let raw = W1NetlinkMsg {
            r#type: msg_type.into(),
            status,
            len,
            id: id.0,
        };
        let msg = safe_transmute::transmute_one_to_bytes(&raw);

        debug_assert_eq!(Self::HEADER_LEN, mem::size_of::<W1NetlinkMsg>());
        buffer[0..Self::HEADER_LEN].copy_from_slice(msg);

        let mut cursor = 0;
        for item in &self.cmds {
            let len = item.buffer_len();
            item.serialize(&mut buffer[cursor..cursor + len]);
            cursor += len;
        }
    }
}
