//! Defines the data structures for HTX frames as per the Betanet specification.

use bytes::{Buf, BufMut, Bytes, BytesMut};
use quinn_proto::coding::Codec;
use quinn_proto::{VarInt, VarIntBoundsExceeded};

/// Represents the type of an HTX frame, corresponding to the `type` field in the spec.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum FrameType {
    Stream = 0,
    Ping = 1,
    Close = 2,
    KeyUpdate = 3,
    WindowUpdate = 4,
}

impl TryFrom<u8> for FrameType {
    type Error = u8;

    /// Attempts to convert a raw byte into a `FrameType`.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FrameType::Stream),
            1 => Ok(FrameType::Ping),
            2 => Ok(FrameType::Close),
            3 => Ok(FrameType::KeyUpdate),
            4 => Ok(FrameType::WindowUpdate),
            _ => Err(value),
        }
    }
}

/// A high-level, in-memory representation of a decoded HTX frame.
/// Application logic will primarily interact with this enum.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Frame {
    /// Carries application data for a specific stream.
    /// Corresponds to `FrameType::Stream`.
    Stream { stream_id: u64, data: Vec<u8> },

    /// Used for keep-alive and RTT measurements. Can carry an opaque payload.
    /// Corresponds to `FrameType::Ping`.
    Ping { payload: Vec<u8> },

    /// Terminates the connection. The spec is silent on a payload, but a `varint`
    /// error code is a sensible choice for a robust implementation.
    /// Corresponds to `FrameType::Close`.
    Close { error_code: u64 },

    /// Signals an update to the session's encryption keys.
    /// Corresponds to `FrameType::KeyUpdate`.
    KeyUpdate,

    /// Updates the flow control window for a specific stream.
    /// Corresponds to `FrameType::WindowUpdate`.
    WindowUpdate { stream_id: u64, increment: u64 },
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("connection closed, not enough data to decode")]
    ConnectionClosed,
    #[error("frame is larger than the 16MB limit")]
    FrameTooLarge,
    #[error("invalid frame type byte: {0}")]
    InvalidFrameType(u8),
    #[error("varint encoding or decoding failed")]
    VarInt(#[from] quinn_proto::coding::UnexpectedEnd),
    #[error("varint value out of bounds")]
    VarIntBounds(#[from] VarIntBoundsExceeded),
    #[error("a required stream_id was missing from a STREAM or WINDOW_UPDATE frame")]
    MissingStreamId,
}

// Per spec, length is a u24, so max frame payload size is 2^24 - 1, which is ~16MB.
const MAX_FRAME_PAYLOAD_SIZE: u32 = (1 << 24) - 1;

impl Frame {
    /// Encodes a `Frame` into the provided buffer according to the spec's wire format.
    pub fn encode(&self, buf: &mut BytesMut) {
        let (frame_type, stream_id, payload) = self.parts();

        let length = payload.len() as u32;
        // This should be enforced by a higher layer, but we assert here as a safeguard.
        assert!(length <= MAX_FRAME_PAYLOAD_SIZE);

        // Write header: length (u24) and type (u8)
        buf.put_uint(length as u64, 3);
        buf.put_u8(frame_type as u8);

        // Write stream_id if present
        if let Some(sid) = stream_id {
            VarInt::from_u64(sid).unwrap().encode(buf);
        }

        // Write the payload (which would be ciphertext in a real scenario)
        buf.put(payload);
    }

    /// Attempts to decode a `Frame` from the provided buffer.
    ///
    /// Returns `Ok(None)` if the buffer does not contain a full frame yet.
    /// On success, the parsed frame's bytes are consumed from the buffer.
    pub fn decode(buf: &mut BytesMut) -> Result<Option<Self>, Error> {
        // A temporary cursor to avoid modifying `buf` until we're sure we have a full frame.
        let mut cursor = &buf[..];

        // 1. Read fixed-size header (length + type)
        if cursor.len() < 4 {
            return Ok(None);
        }
        let length = cursor.get_uint(3) as u32;
        let type_byte = cursor.get_u8();

        if length > MAX_FRAME_PAYLOAD_SIZE {
            return Err(Error::FrameTooLarge);
        }

        let frame_type = FrameType::try_from(type_byte).map_err(Error::InvalidFrameType)?;

        // 2. Read variable-length stream_id if required
        let stream_id = if matches!(frame_type, FrameType::Stream | FrameType::WindowUpdate) {
            let sid_varint = match VarInt::decode(&mut cursor) {
                Ok(vi) => vi,
                Err(quinn_proto::coding::UnexpectedEnd) => return Ok(None), // Not enough data
            };
            Some(sid_varint.into_inner())
        } else {
            None
        };

        // 3. Check if we have the full frame payload
        if cursor.len() < length as usize {
            return Ok(None); // Not enough data for the payload
        }

        // We have a full frame, so we can consume it from the original buffer.
        let header_len = buf.len() - cursor.len();
        buf.advance(header_len);
        let payload = buf.split_to(length as usize).freeze();

        Ok(Some(Self::from_parts(frame_type, stream_id, payload)?))
    }

    /// Helper to deconstruct a `Frame` into its raw parts for encoding.
    fn parts(&self) -> (FrameType, Option<u64>, Bytes) {
        match self {
            Frame::Stream { stream_id, data } => {
                (FrameType::Stream, Some(*stream_id), Bytes::copy_from_slice(data))
            }
            Frame::Ping { payload } => (FrameType::Ping, None, Bytes::copy_from_slice(payload)),
            Frame::Close { error_code } => {
                let mut payload = BytesMut::new();
                VarInt::from_u64(*error_code).unwrap().encode(&mut payload);
                (FrameType::Close, None, payload.freeze())
            }
            Frame::KeyUpdate => (FrameType::KeyUpdate, None, Bytes::new()),
            Frame::WindowUpdate { stream_id, increment } => {
                let mut payload = BytesMut::new();
                VarInt::from_u64(*increment).unwrap().encode(&mut payload);
                (FrameType::WindowUpdate, Some(*stream_id), payload.freeze())
            }
        }
    }

    /// Helper to construct a `Frame` from its raw parts after decoding.
    fn from_parts(
        frame_type: FrameType,
        stream_id: Option<u64>,
        mut payload: Bytes,
    ) -> Result<Self, Error> {
        match frame_type {
            FrameType::Stream => {
                let stream_id = stream_id.ok_or(Error::MissingStreamId)?;
                Ok(Frame::Stream { stream_id, data: payload.to_vec() })
            }
            FrameType::Ping => Ok(Frame::Ping { payload: payload.to_vec() }),
            FrameType::Close => {
                let error_code = VarInt::decode(&mut payload)?.into_inner();
                Ok(Frame::Close { error_code })
            }
            FrameType::KeyUpdate => Ok(Frame::KeyUpdate),
            FrameType::WindowUpdate => {
                let stream_id = stream_id.ok_or(Error::MissingStreamId)?;
                let increment = VarInt::decode(&mut payload)?.into_inner();
                Ok(Frame::WindowUpdate { stream_id, increment })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    // A simple round-trip test for all frame types.
    fn roundtrip_test(frame: Frame) {
        let mut buf = BytesMut::new();
        frame.encode(&mut buf);
        let decoded = Frame::decode(&mut buf).unwrap().unwrap();
        assert_eq!(frame, decoded);
        assert_eq!(buf.len(), 0, "Buffer should be fully consumed");
    }

    #[test]
    fn roundtrip_ping() {
        roundtrip_test(Frame::Ping {
            payload: b"hello world".to_vec(),
        });
    }

    #[test]
    fn roundtrip_ping_empty() {
        roundtrip_test(Frame::Ping { payload: vec![] });
    }

    #[test]
    fn roundtrip_stream() {
        roundtrip_test(Frame::Stream {
            stream_id: 42,
            data: vec![1, 2, 3, 4, 5],
        });
    }

    #[test]
    fn roundtrip_stream_big_id() {
        // VarInts are limited to 62 bits.
        roundtrip_test(Frame::Stream {
            stream_id: (1 << 62) - 1,
            data: vec![1],
        });
    }

    #[test]
    fn roundtrip_close() {
        roundtrip_test(Frame::Close { error_code: 0 });
        roundtrip_test(Frame::Close { error_code: 12345 });
    }

    #[test]
    fn roundtrip_key_update() {
        roundtrip_test(Frame::KeyUpdate);
    }

    #[test]
    fn roundtrip_window_update() {
        roundtrip_test(Frame::WindowUpdate {
            stream_id: 100,
            increment: 65536,
        });
    }

    #[test]
    fn decode_needs_more_data() {
        let frame = Frame::Stream {
            stream_id: 1,
            data: b"some data".to_vec(),
        };
        let mut buf = BytesMut::new();
        frame.encode(&mut buf);

        // Try to decode from an incomplete buffer
        for i in 0..buf.len() {
            let mut incomplete_buf = BytesMut::from(&buf[0..i]);
            if i == buf.len() {
                // The last iteration with the full buffer should succeed
                assert!(Frame::decode(&mut incomplete_buf).unwrap().is_some());
            } else {
                assert!(Frame::decode(&mut incomplete_buf).unwrap().is_none());
            }
        }

        // Full buffer should decode fine
        assert!(Frame::decode(&mut buf).unwrap().is_some());
    }

    #[test]
    fn decode_multiple_frames_from_buffer() {
        let frame1 = Frame::Ping { payload: b"ping".to_vec() };
        let frame2 = Frame::Close { error_code: 42 };
        let frame3 = Frame::KeyUpdate;

        let mut buf = BytesMut::new();
        frame1.encode(&mut buf);
        frame2.encode(&mut buf);
        frame3.encode(&mut buf);

        let d1 = Frame::decode(&mut buf).unwrap().unwrap();
        assert_eq!(frame1, d1);

        let d2 = Frame::decode(&mut buf).unwrap().unwrap();
        assert_eq!(frame2, d2);

        let d3 = Frame::decode(&mut buf).unwrap().unwrap();
        assert_eq!(frame3, d3);

        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn invalid_frame_type() {
        let mut buf = BytesMut::from(&[0x00, 0x00, 0x01, 0xFF, 0x01][..]); // Invalid type 255
        let err = Frame::decode(&mut buf).unwrap_err();
        assert!(matches!(err, Error::InvalidFrameType(255)));
    }
}
