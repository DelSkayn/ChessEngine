use std::{
    fmt::{self, Write},
    io,
};

use bytes::BytesMut;
use chess_core::uci::{Cmd, Msg};
use tokio_util::codec::{Decoder, Encoder};
use tracing::warn;

pub struct MsgCodec;

impl Decoder for MsgCodec {
    type Item = Msg;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Msg>, io::Error> {
        loop {
            let Some(newline) = src.iter().copied().enumerate().find(|(_,x)| *x == b'\n').map(|(x,_)| x) else {
            return Ok(None)
            };
            let line = src.split_to(newline + 1);
            let Ok(line) = std::str::from_utf8(&line) else {
                continue
            };
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let Some(msg) = Msg::from_line(line) else {
                warn!("read invalid uci message: {}",line);
                continue;
            };
            return Ok(Some(msg));
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Io error: {0}")]
    Io(#[from] io::Error),
    #[error("Fmt error: {0}")]
    Fmt(#[from] fmt::Error),
}

impl Encoder<Msg> for MsgCodec {
    type Error = Error;

    fn encode(&mut self, item: Msg, dst: &mut BytesMut) -> Result<(), Self::Error> {
        write!(dst, "{}\n", item)?;
        Ok(())
    }
}

pub struct CmdCodec;

impl Decoder for CmdCodec {
    type Item = Cmd;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Cmd>, io::Error> {
        loop {
            let Some(newline) = src.iter().copied().enumerate().find(|(_,x)| *x == b'\n').map(|(x,_)| x) else {
            return Ok(None)
            };
            let line = src.split_to(newline + 1);
            let Ok(line) = std::str::from_utf8(&line) else {
                continue
            };
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let Some(msg) = Cmd::from_line(line) else {
                warn!("read invalid uci message: {}",line);
                continue;
            };
            return Ok(Some(msg));
        }
    }
}

impl Encoder<Cmd> for CmdCodec {
    type Error = Error;

    fn encode(&mut self, item: Cmd, dst: &mut BytesMut) -> Result<(), Self::Error> {
        write!(dst, "{}\n", item)?;
        Ok(())
    }
}

pub struct EngineCodec;

impl Encoder<Cmd> for EngineCodec {
    type Error = Error;

    fn encode(&mut self, item: Cmd, dst: &mut BytesMut) -> Result<(), Self::Error> {
        (CmdCodec).encode(item, dst)
    }
}

impl Decoder for EngineCodec {
    type Item = Msg;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Msg>, io::Error> {
        MsgCodec.decode(src)
    }
}
