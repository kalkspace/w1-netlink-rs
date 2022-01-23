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

use std::{convert::Infallible, marker::PhantomData};

pub mod command;
pub mod connector;
pub mod message;

#[derive(Debug, thiserror::Error)]
#[error("Invalid value received: {0}")]
pub struct InvalidValue(u8);

#[derive(Debug, thiserror::Error)]
#[error("Invalid length: {0}")]
pub struct InvalidLength(usize);

pub trait Serializable {
    fn buffer_len(&self) -> usize;

    fn serialize(&self, buffer: &mut [u8]);
}

impl Serializable for () {
    fn buffer_len(&self) -> usize {
        0
    }

    fn serialize(&self, _buffer: &mut [u8]) {}
}

pub trait Deserializable
where
    Self: Sized,
{
    type Header;
    type Error: std::error::Error + Send + Sync + 'static;

    fn deserialize(header: &Self::Header, payload: &[u8]) -> Result<(Self, usize), Self::Error>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Empty<H>(PhantomData<H>);

impl<H> Deserializable for Empty<H> {
    type Header = H;
    type Error = Infallible;

    fn deserialize(_header: &Self::Header, _payload: &[u8]) -> Result<(Self, usize), Self::Error> {
        Ok((Self(PhantomData::default()), 0))
    }
}
