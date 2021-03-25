//! Serial Line Internet Protocol (SLIP)
//!
//! Pure Rust implementation of [RFC 1055](https://tools.ietf.org/html/rfc1055)
//! Serial Line IP.
//!
//! ## What is SLIP
//!
//! SLIP is a very simple packet framing protocol. It is used to convert streams of
//! bytes into frames and vice versa. It has no addressing, packet types, error
//! correction or compression. SLIP just solves the problem of framing arbitrary
//! sized data streams!
//!
//! ## Examples
//!
//! SLIP can be used to both encode and decode streams of bytes:
//!
//! ### Encoding
//!
//! The SLIP encoder can process multiple input slices before ending a packet:
//!
//! ```
//! use serial_line_ip::Encoder;
//!
//! const INPUT_1: &[u8] = &[0x01, 0x02, 0x03];
//! const INPUT_2: &[u8] = &[0x04, 0x05, 0x06];
//! const EXPECTED: &[u8] = &[0xc0,
//!     0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
//!     0xc0
//! ];
//! let mut output: [u8; 32] = [0; 32];
//!
//! let mut slip = Encoder::new();
//!
//! let mut totals = slip.encode(INPUT_1, &mut output).unwrap();
//! let expected_bytes_written = 1 + INPUT_1.len();
//! assert_eq!(expected_bytes_written, totals.written);
//!
//! totals += slip.encode(INPUT_2, &mut output[totals.written..]).unwrap();
//! let expected_bytes_written = expected_bytes_written + INPUT_2.len();
//! assert_eq!(expected_bytes_written, totals.written);
//!
//! totals += slip.finish(&mut output[totals.written..]).unwrap();
//! assert_eq!(expected_bytes_written + 1, totals.written);
//! assert_eq!(EXPECTED, &output[..totals.written]);
//! ```
//!
//! ### Decoding
//!
//! Since the length and number of packets in a data stream (byte slice)
//! is unknown upfront, the length of the input bytes processed, output slice
//! and an indication if the end of a packet was reached, is provided by the
//! decoder:
//!
//! ```
//! use serial_line_ip::Decoder;
//!
//! const SLIP_ENCODED: [u8; 7] = [
//!     0xc0,
//!     0x01, 0x02, 0x03, 0x04, 0x05,
//!     0xc0
//! ];
//! const DATA: [u8; 5] = [0x01, 0x02, 0x03, 0x04, 0x05];
//!
//! let mut output: [u8; 32] = [0; 32];
//! let mut slip = Decoder::new();
//!
//! let (input_bytes_processed, output_slice, is_end_of_packet) =
//!     slip.decode(&SLIP_ENCODED, &mut output).unwrap();
//!
//! assert_eq!(SLIP_ENCODED.len(), input_bytes_processed);
//! assert_eq!(&DATA, output_slice);
//! assert_eq!(true, is_end_of_packet);
//! ```

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

mod decoder;
mod encoder;
mod error;

pub use decoder::Decoder;
pub use encoder::{EncodeTotals, Encoder};
pub use error::{Error, Result};

/// Frame end
const END: u8 = 0xC0;

/// Frame escape
const ESC: u8 = 0xDB;

/// Transposed frame end
const ESC_END: u8 = 0xDC;

/// Transposed frame escape
const ESC_ESC: u8 = 0xDD;
