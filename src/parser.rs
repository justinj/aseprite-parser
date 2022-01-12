use std::io::{BufReader, Read, Seek, SeekFrom};
use std::mem::size_of;

use crate::AsepriteError;

pub(crate) trait Parse: Sized {
    fn parse<R: Read + Seek>(p: &mut Parser<R>) -> Result<Self, AsepriteError>;
}

macro_rules! impl_parse {
    ($type_name:ty) => {
        impl Parse for $type_name {
            fn parse<R: Read + Seek>(p: &mut Parser<R>) -> Result<Self, AsepriteError> {
                let n = size_of::<Self>();
                let next_n = p.next_n(n)?;
                Ok(Self::from_le_bytes(next_n.try_into()?))
            }
        }
    };
}

impl_parse!(u8);
impl_parse!(u16);
impl_parse!(u32);
impl_parse!(u64);
impl_parse!(i8);
impl_parse!(i16);
impl_parse!(i32);
impl_parse!(i64);

impl Parse for String {
    fn parse<R: Read + Seek>(p: &mut Parser<R>) -> Result<Self, AsepriteError> {
        // Strings in Aseprite files are always length-prefixed with a u16.
        let len = u16::parse(p)?.try_into()?;
        Ok(String::from_utf8(p.next_n(len)?.to_vec())?)
    }
}

#[derive(Debug)]
pub struct Skip<const N: usize>;

impl<const N: usize> Parse for Skip<N> {
    fn parse<R: Read + Seek>(p: &mut Parser<R>) -> Result<Self, AsepriteError> {
        p.skip(N)?;
        Ok(Self)
    }
}

#[derive(Debug)]
pub(crate) struct Parser<R>
where
    R: Read,
{
    buf: Vec<u8>,
    reader: BufReader<R>,
    pos: usize,
}

impl<R> Parser<R>
where
    R: Read + Seek,
{
    pub(crate) fn new(r: R) -> Self {
        Parser {
            buf: Vec::new(),
            reader: BufReader::new(r),
            pos: 0,
        }
    }

    pub(crate) fn seek(&mut self, n: u64) -> Result<(), AsepriteError> {
        self.reader.seek(SeekFrom::Start(n))?;
        Ok(())
    }

    pub(crate) fn next_n(&mut self, n: usize) -> Result<&[u8], AsepriteError> {
        self.pos += n;
        self.buf.clear();
        self.buf.extend((0..n).map(|_| 0));
        self.reader.read_exact(&mut self.buf)?;
        Ok(&self.buf)
    }

    pub(crate) fn next<P: Parse>(&mut self) -> Result<P, AsepriteError> {
        P::parse(self)
    }

    pub(crate) fn skip(&mut self, n: usize) -> Result<(), AsepriteError> {
        self.next_n(n)?;
        Ok(())
    }

    pub(crate) fn position(&self) -> usize {
        self.pos
    }

    pub(crate) fn advance_to(&mut self, n: usize) -> Result<(), AsepriteError> {
        if n < self.pos {
            return Err(AsepriteError::CorruptFile(
                "cannot advance past current position".into(),
            ));
        }
        let extra = n - self.pos;
        let _ = self.next_n(extra)?;
        Ok(())
    }
}
