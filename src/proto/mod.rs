//! Each connector message can include one or more w1_netlink_msg
//! with zero or more attached w1_netlink_cmd messages.
//!
//! For event messages there are no w1_netlink_cmd embedded structures,
//! only connector header and w1_netlink_msg structure with "len" field
//! being zero and filled type (one of event types) and id: either 8 bytes
//! of slave unique id in host order, or master's id, which is assigned
//! to bus master device when it is added to w1 core.
//!
//! Currently replies to userspace commands are only generated for read
//! command request. One reply is generated exactly for one w1_netlink_cmd
//! read request.

pub mod command;
pub mod connector;
pub mod message;

#[derive(Debug, thiserror::Error)]
#[error("Invalid value received: {0}")]
pub struct InvalidValue(u8);

pub trait Serializable {
    fn buffer_len(&self) -> usize;

    fn serialize(&self, buffer: &mut [u8]);
}

pub trait Deserializable
where
    Self: Sized,
{
    type Header;
    type Error: std::error::Error + Send + Sync + 'static;

    fn deserialize(header: &Self::Header, payload: &[u8]) -> Result<(Self, usize), Self::Error>;
}
