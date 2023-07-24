#[cfg(feature = "tokio")]
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Status {
    Succeeded = 0x00,
    Failed = 0xff,
}

impl From<Status> for u8 {
    fn from(value: Status) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for Status {
    type Error = std::io::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let err = format!("Invalid sub-negotiation status {0:#x}", value);
        match value {
            0x00 => Ok(Status::Succeeded),
            0xff => Ok(Status::Failed),
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err)),
        }
    }
}

/// SOCKS5 password handshake response
///
/// ```plain
/// +-----+--------+
/// | VER | STATUS |
/// +-----+--------+
/// |  1  |   1    |
/// +-----+--------+
/// ```
#[derive(Clone, Debug)]
pub struct Response {
    pub status: Status,
}

impl Response {
    pub fn new(status: Status) -> Self {
        Self { status }
    }

    pub fn rebuild_from_stream<R: std::io::Read>(r: &mut R) -> std::io::Result<Self> {
        let mut ver = [0; 1];
        r.read_exact(&mut ver)?;
        let ver = ver[0];

        if ver != super::SUBNEGOTIATION_VERSION {
            let err = format!("Unsupported sub-negotiation version {0:#x}", ver);
            return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, err));
        }

        let mut status = [0; 1];
        r.read_exact(&mut status)?;
        let status = Status::try_from(status[0])?;
        Ok(Self { status })
    }

    #[cfg(feature = "tokio")]
    pub async fn async_rebuild_from_stream<R: AsyncRead + Unpin>(r: &mut R) -> std::io::Result<Self> {
        let ver = r.read_u8().await?;

        if ver != super::SUBNEGOTIATION_VERSION {
            let err = format!("Unsupported sub-negotiation version {0:#x}", ver);
            return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, err));
        }

        let status = Status::try_from(r.read_u8().await?)?;
        Ok(Self { status })
    }

    pub fn write_to_stream<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        use bytes::BytesMut;
        let mut buf = BytesMut::with_capacity(self.serialized_len());
        self.write_to_buf(&mut buf);
        w.write_all(&buf)
    }

    #[cfg(feature = "tokio")]
    pub async fn async_write_to_stream<W: AsyncWrite + Unpin>(&self, w: &mut W) -> std::io::Result<()> {
        use bytes::BytesMut;
        let mut buf = BytesMut::with_capacity(self.serialized_len());
        self.write_to_buf(&mut buf);
        w.write_all(&buf).await
    }

    pub fn write_to_buf<B: bytes::BufMut>(&self, buf: &mut B) {
        buf.put_u8(super::SUBNEGOTIATION_VERSION);
        buf.put_u8(self.status.into());
    }

    pub fn serialized_len(&self) -> usize {
        2
    }
}
