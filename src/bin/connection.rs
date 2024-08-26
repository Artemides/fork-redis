use std::io::Cursor;

use bytes::{Buf, BytesMut};
use mini_redis::{Frame, Result};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

struct Conection {
    stream: BufWriter<TcpStream>,
    buf: BytesMut,
}

impl Conection {
    async fn parse_frame(&mut self) -> Result<Option<Frame>> {
        let mut buf = Cursor::new(&self.buf[..]);
        match mini_redis::Frame::check(&mut buf) {
            Ok(_) => {
                let len = buf.position() as usize;
                buf.set_position(0);
                let frame = mini_redis::Frame::parse(&mut buf)?;
                self.buf.advance(len);

                Ok(Some(frame))
            }
            Err(mini_redis::frame::Error::Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse_frame().await? {
                return Ok(Some(frame));
            }

            if 0 == self.stream.read(&mut self.buf).await? {
                if self.buf.is_empty() {
                    return Ok(None);
                } else {
                    return Err("Connection closed by peer".into());
                }
            }
        }
    }

    async fn write_frame(&mut self, frame: Frame) -> io::Result<()> {
        match frame {
            Frame::Simple(value) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(value.as_bytes()).await?;
            }
            Frame::Integer(val) => {
                self.stream.write_u8(b':').await?;
                self.write_decimal(val).await?;
            }
            Frame::Bulk(val) => {
                self.stream.write_u8(b'$').await?;
                self.write_decimal(val.len() as u64).await?;
                self.stream.write_all(&val).await?;
            }
            Frame::Null => {
                self.stream.write_all(b"$-1").await?;
            }
            Frame::Error(val) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(val.as_bytes()).await?;
            }
            Frame::Array(_val) => {
                unimplemented!()
            }
        }
        self.stream.write_all(b"\r\n").await?;
        self.stream.flush().await?;

        Ok(())
    }

    async fn write_decimal(&mut self, value: u64) -> io::Result<()> {
        use std::io::Write;
        let mut buf = [0u8; 12];
        let mut buf = Cursor::new(&mut buf[..]);
        write!(&mut buf, "{}", value)?;
        let pos = buf.position() as usize;
        self.stream.write_all(&buf.get_ref()[..pos]).await?;
        Ok(())
    }
}

#[tokio::main]

async fn main() {}
