use crate::InvalidValue;

pub enum W1Command {
    Read,
    Write,
    Search,
    AlarmSearch,
    Touch,
    Reset,
    SlaveAdd,
    SlaveRemove,
    ListSlaves,
    Max,
}

impl TryFrom<u8> for W1Command {
    type Error = InvalidValue;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use crate::constants::*;
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
            W1_CMD_MAX => Self::Max,
            v => return Err(InvalidValue(v)),
        };
        Ok(cmd)
    }
}

impl From<W1Command> for u8 {
    fn from(cmd: W1Command) -> Self {
        use crate::constants::*;
        match cmd {
            W1Command::Write => W1_CMD_WRITE,
            W1Command::Read => W1_CMD_READ,
            W1Command::Search => W1_CMD_SEARCH,
            W1Command::AlarmSearch => W1_CMD_ALARM_SEARCH,
            W1Command::Touch => W1_CMD_TOUCH,
            W1Command::Reset => W1_CMD_RESET,
            W1Command::SlaveAdd => W1_CMD_SLAVE_ADD,
            W1Command::SlaveRemove => W1_CMD_SLAVE_REMOVE,
            W1Command::ListSlaves => W1_CMD_LIST_SLAVES,
            W1Command::Max => W1_CMD_MAX,
        }
    }
}

pub struct W1NetlinkCommand<T> {
    cmd: W1Command,
    data: T,
}

impl<T> W1NetlinkCommand<T> {
    pub fn new(cmd: W1Command, data: T) -> Self {
        Self { cmd, data }
    }
}
