use std::mem;

use self::raw::W1NetlinkCmd;
use super::{message::W1MessageHeader, Deserializable, InvalidValue, Serializable};

mod raw {
    //! Taken from https://www.kernel.org/doc/Documentation/w1/w1.netlink

    use safe_transmute::TriviallyTransmutable;

    pub mod constants {
        pub const W1_CMD_READ: u8 = 0;
        pub const W1_CMD_WRITE: u8 = 1;
        pub const W1_CMD_SEARCH: u8 = 2;
        pub const W1_CMD_ALARM_SEARCH: u8 = 3;
        pub const W1_CMD_TOUCH: u8 = 4;
        pub const W1_CMD_RESET: u8 = 5;
        pub const W1_CMD_SLAVE_ADD: u8 = 6;
        pub const W1_CMD_SLAVE_REMOVE: u8 = 7;
        pub const W1_CMD_LIST_SLAVES: u8 = 8;
        #[allow(dead_code)]
        pub const W1_CMD_MAX: u8 = 9;
    }

    /// Command for given master or slave device
    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct W1NetlinkCmd {
        /// Command opcode. See also [constants].
        pub cmd: u8,
        /// reserved
        pub _res: u8,
        /// length of data for this command
        pub len: u16,
    }

    unsafe impl TriviallyTransmutable for W1NetlinkCmd {}
}

#[derive(Debug, Clone, Copy)]
pub enum W1CommandType {
    Read,
    Write,
    Search,
    AlarmSearch,
    Touch,
    Reset,
    SlaveAdd,
    SlaveRemove,
    ListSlaves,
}

impl TryFrom<u8> for W1CommandType {
    type Error = InvalidValue;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use self::raw::constants::*;
        let cmd = match value {
            W1_CMD_WRITE => Self::Write,
            W1_CMD_READ => Self::Read,
            W1_CMD_SEARCH => Self::Search,
            W1_CMD_ALARM_SEARCH => Self::AlarmSearch,
            W1_CMD_TOUCH => Self::Touch,
            W1_CMD_RESET => Self::Reset,
            W1_CMD_SLAVE_ADD => Self::SlaveAdd,
            W1_CMD_SLAVE_REMOVE => Self::SlaveRemove,
            W1_CMD_LIST_SLAVES => Self::ListSlaves,
            v => return Err(InvalidValue(v)),
        };
        Ok(cmd)
    }
}

impl From<W1CommandType> for u8 {
    fn from(cmd: W1CommandType) -> Self {
        use self::raw::constants::*;
        match cmd {
            W1CommandType::Write => W1_CMD_WRITE,
            W1CommandType::Read => W1_CMD_READ,
            W1CommandType::Search => W1_CMD_SEARCH,
            W1CommandType::AlarmSearch => W1_CMD_ALARM_SEARCH,
            W1CommandType::Touch => W1_CMD_TOUCH,
            W1CommandType::Reset => W1_CMD_RESET,
            W1CommandType::SlaveAdd => W1_CMD_SLAVE_ADD,
            W1CommandType::SlaveRemove => W1_CMD_SLAVE_REMOVE,
            W1CommandType::ListSlaves => W1_CMD_LIST_SLAVES,
        }
    }
}

#[derive(Debug, Clone)]
pub enum W1NetlinkCommand {
    Write(Vec<u8>),
    Read(Option<Vec<u8>>),
    Search,
    AlarmSearch,
    Touch,
    Reset,
    //SlaveAdd(), todo
    //SlaveRemove(), todo
    ListSlaves,
}

impl W1NetlinkCommand {
    pub const HEADER_LEN: usize = mem::size_of::<W1NetlinkCmd>();

    fn cmd_type(&self) -> W1CommandType {
        match self {
            W1NetlinkCommand::Write(_) => W1CommandType::Write,
            W1NetlinkCommand::Read(_) => W1CommandType::Read,
            W1NetlinkCommand::Search => W1CommandType::Search,
            W1NetlinkCommand::AlarmSearch => W1CommandType::AlarmSearch,
            W1NetlinkCommand::Touch => W1CommandType::Touch,
            W1NetlinkCommand::Reset => W1CommandType::Reset,
            W1NetlinkCommand::ListSlaves => W1CommandType::ListSlaves,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
    #[error("Invalid command type")]
    InvalidType(#[from] InvalidValue),

    #[error("Unable to read header: {0}")]
    InvalidHeader(safe_transmute::Error<'static, u8, W1NetlinkCmd>),
}

impl Deserializable for W1NetlinkCommand {
    type Header = W1MessageHeader;
    type Error = DeserializeError;

    fn deserialize(_header: &Self::Header, payload: &[u8]) -> Result<(Self, usize), Self::Error> {
        let (header, payload) = payload.split_at(mem::size_of::<W1NetlinkCmd>());
        let W1NetlinkCmd { cmd, len, .. } = safe_transmute::transmute_one_pedantic(header)
            .map_err(|e| Self::Error::InvalidHeader(e.without_src()))?;

        let cmd = match W1CommandType::try_from(cmd)? {
            W1CommandType::Read => {
                let payload = Some(len).filter(|l| *l > 0).map(|_| payload.to_vec());
                Self::Read(payload)
            }
            W1CommandType::Write => {
                let payload = payload.to_vec();
                Self::Write(payload)
            }
            W1CommandType::Search => Self::Search,
            W1CommandType::AlarmSearch => Self::AlarmSearch,
            W1CommandType::Touch => Self::Touch,
            W1CommandType::Reset => Self::Reset,
            W1CommandType::SlaveAdd => unimplemented!(),
            W1CommandType::SlaveRemove => unimplemented!(),
            W1CommandType::ListSlaves => Self::ListSlaves,
        };
        Ok((cmd, len as usize))
    }
}

impl Serializable for W1NetlinkCommand {
    fn buffer_len(&self) -> usize {
        let inner = match self {
            W1NetlinkCommand::Write(pl) => pl.len(),
            W1NetlinkCommand::Read(pl) => pl.as_ref().map(Vec::len).unwrap_or_default(),
            W1NetlinkCommand::Search => 0,
            W1NetlinkCommand::AlarmSearch => 0,
            W1NetlinkCommand::Touch => 0,
            W1NetlinkCommand::Reset => 0,
            W1NetlinkCommand::ListSlaves => todo!(),
        };
        inner + Self::HEADER_LEN
    }

    fn serialize(&self, buffer: &mut [u8]) {
        let cmd_type = self.cmd_type();
        let len = (self.buffer_len() - Self::HEADER_LEN) as u16;
        let raw = W1NetlinkCmd {
            cmd: cmd_type.into(),
            _res: Default::default(),
            len,
        };

        let msg = safe_transmute::transmute_one_to_bytes(&raw);
        debug_assert_eq!(mem::size_of::<W1NetlinkCmd>(), Self::HEADER_LEN);
        buffer[0..Self::HEADER_LEN].copy_from_slice(msg);

        match self {
            W1NetlinkCommand::Write(pl) => buffer[Self::HEADER_LEN..].copy_from_slice(pl),
            W1NetlinkCommand::Read(pl) => {
                if let Some(pl) = pl {
                    buffer[Self::HEADER_LEN..].copy_from_slice(pl);
                }
            }
            W1NetlinkCommand::Search => {}
            W1NetlinkCommand::AlarmSearch => {}
            W1NetlinkCommand::Touch => {}
            W1NetlinkCommand::Reset => {}
            W1NetlinkCommand::ListSlaves => todo!(),
        }
    }
}
