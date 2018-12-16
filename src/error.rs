
/// Type alias for handling SLIP errors.
pub type Result<T> = core::result::Result<T, self::Error>;

/// Errors encountered by SLIP.
#[derive(Debug)]
pub enum Error {
    // Encoder errors
    /// The encoder does not have enough space to write the SLIP header.
    NoOutputSpaceForHeader,
    /// The encoder does not have enough space to write an `ESC, ESC_ESC` sequence.
    NoOutputSpaceForEscEscapeSequence,
    /// The encoder does not have enough space to write an `ESC, END_ESC` sequence.
    NoOutputSpaceForEndEscapeSequence,
    /// The encoder does not have enough space to write input data into the output buffer.
    NoOutputSpaceForInputData,
    /// The encoder does not have enough space to write the final SLIP end byte.
    NoOutputSpaceForEndByte,

    // Decoder errors
    /// The decoder cannot process the SLIP header.
    BadHeaderDecode,
    /// The decoder cannot process the SLIP escape sequence.
    BadEscapeSequenceDecode,
}



