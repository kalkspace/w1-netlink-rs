use crate::InvalidValue;

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
        use crate::constants::*;
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
        use crate::constants::*;
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

pub struct W1NetlinkMessage<T> {
    r#type: W1MessageType,
    status: u8,
    id: u64,
    data: T,
}

impl<T> W1NetlinkMessage<T> {
    pub fn new(msg_type: W1MessageType, id: u64, data: T) -> Self {
        Self {
            r#type: msg_type,
            status: 0,
            id,
            data,
        }
    }
}
