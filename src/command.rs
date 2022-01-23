use crate::{message::W1MessageHeader, Deserializable, InvalidValue, Serializable};

mod raw {
    //! Taken from https://www.kernel.org/doc/Documentation/w1/w1.netlink

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
    pub struct W1NetlinkCmd {
        /// Command opcode. See also [constants].
        cmd: u8,
        /// reserved
        _res: u8,
        /// length of data for this command
        len: u16,
    }
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
pub enum DeserializeError {}

impl Deserializable for W1NetlinkCommand {
    type Header = W1MessageHeader;
    type Error = DeserializeError;

    fn deserialize(header: &Self::Header, payload: &[u8]) -> Result<(Self, usize), Self::Error> {
        todo!()
    }
}

impl Serializable for W1NetlinkCommand {
    fn buffer_len(&self) -> usize {
        todo!()
    }

    fn serialize(&self, buffer: &mut [u8]) {
        todo!()
    }
}
