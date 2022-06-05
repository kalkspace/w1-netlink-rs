use std::{iter, mem};

use self::raw::W1NetlinkMsg;
use super::{
    command::W1NetlinkCommand, connector::NlConnectorType, Deserializable, InvalidValue,
    Serializable,
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
enum W1MessageType {
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

#[derive(Debug, Clone, Copy)]
pub enum EventKind {
    Add,
    Remove,
}

#[derive(Debug, Clone)]
pub enum W1NetlinkMessage {
    ListMasters(Option<Vec<u32>>),
    MasterCommand {
        target: u32,
        cmds: Vec<W1NetlinkCommand>,
    },
    SlaveCommand {
        target: u64,
        cmds: Vec<W1NetlinkCommand>,
    },
    MasterEvent {
        kind: EventKind,
        target: u32,
    },
    SlaveEvent {
        kind: EventKind,
        target: u64,
    },
}

impl W1NetlinkMessage {
    pub const HEADER_LEN: usize = mem::size_of::<W1NetlinkMsg>();
}

impl NlConnectorType for W1NetlinkMessage {
    fn idx() -> u32 {
        raw::constants::CONNECTOR_W1_IDX
    }

    fn val() -> u32 {
        raw::constants::CONNECTOR_W1_VAL
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
    #[error("Invalid header payload: {0}")]
    InvalidHeader(safe_transmute::Error<'static, u8, W1NetlinkMsg>),

    #[error("Invalid message type: {0}")]
    InvalidMessageType(InvalidValue),

    #[error("Payload length does not match header")]
    InvalidPayloadLength,

    #[error(transparent)]
    Command(#[from] super::command::DeserializeError),
}

impl Deserializable for W1NetlinkMessage {
    type Error = DeserializeError;

    fn deserialize(payload: &[u8]) -> Result<(Self, usize), Self::Error> {
        let (header, payload) = payload.split_at(Self::HEADER_LEN);
        let W1NetlinkMsg {
            r#type,
            status,
            len,
            id,
        } = safe_transmute::transmute_one_pedantic(header)
            .map_err(|e| Self::Error::InvalidHeader(e.without_src()))?;

        if status > 0 {
            todo!(); // error handling
        }

        let len = len as usize;
        let msg_type = r#type.try_into().map_err(Self::Error::InvalidMessageType)?;
        let ret = match msg_type {
            W1MessageType::SlaveAdd => Self::SlaveEvent {
                kind: EventKind::Add,
                target: u64::from_le_bytes(id),
            },
            W1MessageType::SlaveRemove => Self::SlaveEvent {
                kind: EventKind::Remove,
                target: u64::from_le_bytes(id),
            },
            W1MessageType::MasterAdd => Self::MasterEvent {
                kind: EventKind::Add,
                target: u32::from_le_bytes(id[..4].try_into().unwrap()),
            },
            W1MessageType::MasterRemove => Self::MasterEvent {
                kind: EventKind::Remove,
                target: u32::from_le_bytes(id[..4].try_into().unwrap()),
            },
            W1MessageType::MasterCmd => {
                let target = u32::from_le_bytes(id[..4].try_into().unwrap());
                let (cmds, _) = Deserializable::deserialize(payload)?;
                Self::MasterCommand { target, cmds }
            }
            W1MessageType::SlaveCmd => {
                let target = u64::from_le_bytes(id);
                let (cmds, _) = Deserializable::deserialize(payload)?;
                Self::SlaveCommand { target, cmds }
            }
            W1MessageType::ListMasters => {
                // read from payload
                let mut bus_ids = Vec::new();
                for chunk in payload.chunks(4) {
                    if chunk.len() < 4 {
                        return Err(DeserializeError::InvalidPayloadLength);
                    }
                    let id = u32::from_le_bytes(chunk.try_into().unwrap());
                    bus_ids.push(id);
                }
                Self::ListMasters(Some(bus_ids))
            }
        };
        Ok((ret, len + Self::HEADER_LEN))
    }
}

impl Serializable for W1NetlinkMessage {
    fn buffer_len(&self) -> usize {
        use W1NetlinkMessage::*;
        let inner = match self {
            ListMasters(ids) => ids.as_ref().map(|v| v.len() * 4).unwrap_or(0),
            MasterCommand { cmds, .. } => cmds.iter().map(Serializable::buffer_len).sum(),
            SlaveCommand { cmds, .. } => cmds.iter().map(Serializable::buffer_len).sum(),
            MasterEvent { .. } => 0,
            SlaveEvent { .. } => 0,
        };
        inner + Self::HEADER_LEN
    }

    fn serialize(&self, buffer: &mut [u8]) {
        let len = (self.buffer_len() - Self::HEADER_LEN) as u16;

        use W1NetlinkMessage::*;
        let (msg_type, id, payload) = match self {
            ListMasters(ids) => {
                let pl: Vec<u8> = ids
                    .as_ref()
                    .map(|ids| {
                        ids.iter()
                            .cloned()
                            .map(u32::to_le_bytes)
                            .map(IntoIterator::into_iter)
                            .flatten()
                            .collect()
                    })
                    .unwrap_or_default();
                (W1MessageType::ListMasters, 0u64, pl)
            }
            MasterCommand { target, cmds } => {
                let id: Vec<u8> = target
                    .to_le_bytes()
                    .into_iter()
                    .chain(iter::repeat(0).take(4))
                    .collect();
                let id = u64::from_le_bytes(id.try_into().unwrap());
                let buffer_len = cmds.iter().map(|cmd| cmd.buffer_len()).sum();
                let mut pl = vec![0; buffer_len];
                for cmd in cmds {
                    cmd.serialize(&mut pl);
                }
                (W1MessageType::MasterCmd, id, pl)
            }
            SlaveCommand { target, cmds } => todo!(),
            MasterEvent { kind, target } => todo!(),
            SlaveEvent { kind, target } => todo!(),
        };

        let raw = W1NetlinkMsg {
            r#type: msg_type.into(),
            status: 0,
            len,
            id: id.to_le_bytes(),
        };
        let msg = safe_transmute::transmute_one_to_bytes(&raw);

        debug_assert_eq!(Self::HEADER_LEN, mem::size_of::<W1NetlinkMsg>());
        buffer[0..Self::HEADER_LEN].copy_from_slice(msg);
        buffer[Self::HEADER_LEN..].copy_from_slice(&payload);
    }
}

#[cfg(test)]
mod tests {
    use netlink_packet_core::{NetlinkDeserializable, NetlinkHeader};

    use crate::proto::connector::NlConnectorMessage;

    use super::*;

    #[test]
    fn deserialize_search_response() {
        let payload = vec![
            0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00,
            0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x04, 0x00, 0x04, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
        ];
        let header = NetlinkHeader {
            length: 52,
            message_type: 3,
            flags: 0,
            sequence_number: 0,
            port_number: 0,
        };

        let deserialized_message =
            NlConnectorMessage::<W1NetlinkMessage>::deserialize(&header, &payload).unwrap();

        println!("{:?}", deserialized_message);
    }
}
