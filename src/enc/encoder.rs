use super::{write::Writer, Encoder};
use crate::{
    config::{Config, FixintConfig},
    utils::Sealed,
};

/// An Encoder that writes bytes into a given writer `W`.
///
/// This struct should rarely be used.
/// In most cases, prefer any of the `encode` functions.
///
/// The ByteOrder that is chosen will impact the endianness that
/// is used to write integers to the writer.
///
/// ```
/// # use bincode::enc::{write::SliceWriter, EncoderImpl, Encode};
/// let slice: &mut [u8] = &mut [0, 0, 0, 0];
/// let config = bincode::config::legacy().with_big_endian();
///
/// let mut encoder = EncoderImpl::new(SliceWriter::new(slice), config);
/// // this u32 can be any Encodable
/// 5u32.encode(&mut encoder).unwrap();
/// assert_eq!(encoder.into_writer().bytes_written(), 4);
/// assert_eq!(slice, [0, 0, 0, 5]);
/// ```
pub struct EncoderImpl<W: Writer, C: Config> {
    writer: W,
    config: C,
}

impl<W: Writer, C: Config> EncoderImpl<W, C> {
    /// Create a new Encoder
    pub const fn new(writer: W, config: C) -> EncoderImpl<W, C> {
        EncoderImpl { writer, config }
    }

    /// Return the underlying writer
    #[inline]
    pub fn into_writer(self) -> W {
        self.writer
    }
}

impl<W: Writer, C: Config> Encoder for EncoderImpl<W, C> {
    type W = W;

    type C = C;

    #[inline]
    fn writer(&mut self) -> &mut Self::W {
        &mut self.writer
    }

    #[inline]
    fn config(&self) -> &Self::C {
        &self.config
    }
}

impl<W: Writer, C: Config> Sealed for EncoderImpl<W, C> {}

/// An [Encoder] that forwards to another encoder, but always uses fixed integer encoding.
///
/// Created by [Encoder::with_fixint].
pub struct FixintEncoder<'a, E: Encoder + ?Sized> {
    encoder: &'a mut E,
    config: FixintConfig<E::C>,
}

impl<'a, E: Encoder + ?Sized> FixintEncoder<'a, E> {
    pub(crate) const fn new(encoder: &'a mut E) -> Self {
        FixintEncoder {
            encoder,
            config: FixintConfig::new(),
        }
    }
}

impl<E: Encoder + ?Sized> Encoder for FixintEncoder<'_, E> {
    type W = E::W;

    type C = FixintConfig<E::C>;

    #[inline]
    fn writer(&mut self) -> &mut Self::W {
        self.encoder.writer()
    }

    #[inline]
    fn config(&self) -> &Self::C {
        &self.config
    }
}

impl<E: Encoder + ?Sized> Sealed for FixintEncoder<'_, E> {}
