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
