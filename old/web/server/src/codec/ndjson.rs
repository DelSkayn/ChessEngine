use std::{fmt::Write, io, marker::PhantomData};

use bytes::BufMut;
use serde::{de::DeserializeOwned, Serialize};
use tokio_util::codec::{Decoder as DecoderTrait, Encoder as EncoderTrait};

pub struct Encoder<T: Serialize>(PhantomData<T>);

impl<T: Serialize> Encoder<T> {
    pub fn new() -> Self {
        Encoder(PhantomData)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Io error: {0}")]
    Io(#[from] io::Error),
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

impl<T: Serialize> EncoderTrait<T> for Encoder<T> {
    type Error = Error;

    fn encode(&mut self, item: T, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        serde_json::to_writer(dst.writer(), &item)?;
        writeln!(dst).unwrap();
        Ok(())
    }
}

pub struct Decoder<T: DeserializeOwned>(PhantomData<T>);

impl<T: DeserializeOwned> Decoder<T> {
    pub fn new() -> Self {
        Decoder(PhantomData)
    }
}

impl<T: DeserializeOwned> DecoderTrait for Decoder<T> {
    type Item = T;

    type Error = Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            let Some(newline) = src.iter().copied().enumerate().find(|(_,x)| *x == b'\n').map(|(x,_)| x) else {
            return Ok(None)
            };
            let line = src.split_to(newline);
            let res = serde_json::from_slice(&line)?;
            return Ok(Some(res));
        }
    }
}

pub struct Codec<T: Serialize + DeserializeOwned> {
    encoder: Encoder<T>,
    decoder: Decoder<T>,
}

impl<T: Serialize + DeserializeOwned> Codec<T> {
    pub fn new() -> Self {
        Codec {
            encoder: Encoder::new(),
            decoder: Decoder::new(),
        }
    }
}

impl<T: Serialize + DeserializeOwned> EncoderTrait<T> for Codec<T> {
    type Error = Error;

    fn encode(&mut self, item: T, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        self.encoder.encode(item, dst)
    }
}

impl<T: Serialize + DeserializeOwned> DecoderTrait for Codec<T> {
    type Item = T;

    type Error = Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.decoder.decode(src)
    }
}
