use super::*;

/// SLIP encoder context
pub struct Encoder {
    /// Just keep track of whether we have encoded the header yet
    header_written: bool,
}

impl Encoder {
    /// Create a new context for SLIP encoding
    pub fn new() -> Self {
        Encoder {
            header_written: false,
        }
    }

    /// Encode a buffer into a SLIP stream and returns the number of output bytes written.
    pub fn encode(&mut self, input: &[u8], output: &mut [u8]) -> Result<usize> {
        let mut out_byte = 0;
        if !self.header_written {
            if output.len() < HEADER.len() {
                return Err(Error::NoOutputSpaceForHeader);
            }

            for i in 0..HEADER.len() {
                output[i] = HEADER[i];
            }
            out_byte = HEADER.len();
            self.header_written = true;
        }

        for i in 0..input.len() {
            match input[i] {
                ESC => {
                    if (output.len() - out_byte) < 2 {
                        return Err(Error::NoOutputSpaceForEscEscapeSequence);
                    }
                    output[out_byte] = ESC;
                    output[out_byte + 1] = ESC_ESC;
                    out_byte += 2;
                }
                END => {
                    if (output.len() - out_byte) < 2 {
                        return Err(Error::NoOutputSpaceForEndEscapeSequence);
                    }
                    output[out_byte] = ESC;
                    output[out_byte + 1] = ESC_END;
                    out_byte += 2;
                }
                _ => {
                    if (output.len() - out_byte) < 1 {
                        return Err(Error::NoOutputSpaceForInputData);
                    }
                    output[out_byte] = input[i];
                    out_byte += 1;
                }
            }
        }

        Ok(out_byte)
    }

    /// Finish encoding the current packet and return the number of output bytes written.
    pub fn finish(self, output: &mut [u8]) -> Result<usize> {
        if output.len() < 1 {
            return Err(Error::NoOutputSpaceForEndByte);
        }
        output[0] = END;

        Ok(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_encode() {
        const EXPECTED: [u8; 5] = [0xc0, 0xdb, 0xdc, 0xdd, 0xc0];
        let mut output: [u8; 32] = [0; 32];

        let mut slip = Encoder::new();
        let mut bytes_written = slip.encode(&[0;0], &mut output).unwrap();
        assert_eq!(4, bytes_written);
        bytes_written += slip.finish(&mut output[bytes_written..]).unwrap();
        assert_eq!(5, bytes_written);
        assert_eq!(&EXPECTED, &output[..bytes_written]);
    }

    #[test]
    fn encode_esc_esc_sequence() {
        const INPUT: [u8; 3] = [0x01, ESC, 0x03];
        const EXPECTED: [u8; 9] = [0xc0, 0xdb, 0xdc, 0xdd, 0x01, ESC, ESC_ESC, 0x03, 0xc0];
        let mut output: [u8; 32] = [0; 32];

        let mut slip = Encoder::new();
        let mut bytes_written = slip.encode(&INPUT, &mut output).unwrap();
        assert_eq!(5 + INPUT.len(), bytes_written);
        bytes_written += slip.finish(&mut output[bytes_written..]).unwrap();
        assert_eq!(6 + INPUT.len(), bytes_written);
        assert_eq!(&EXPECTED, &output[..bytes_written]);
    }

    #[test]
    fn encode_end_esc_sequence() {
        const INPUT: [u8; 3] = [0x01, END, 0x03];
        const EXPECTED: [u8; 9] = [0xc0, 0xdb, 0xdc, 0xdd, 0x01, ESC, ESC_END, 0x03, 0xc0];
        let mut output: [u8; 32] = [0; 32];

        let mut slip = Encoder::new();
        let mut bytes_written = slip.encode(&INPUT, &mut output).unwrap();
        assert_eq!(5 + INPUT.len(), bytes_written);
        bytes_written += slip.finish(&mut output[bytes_written..]).unwrap();
        assert_eq!(6 + INPUT.len(), bytes_written);
        assert_eq!(&EXPECTED, &output[..bytes_written]);
    }

    #[test]
    fn multi_part_encode() {
        const INPUT_1: [u8; 4] = [0x01, 0x02, 0x03, ESC];
        const INPUT_2: [u8; 4] = [0x05, END, 0x07, 0x08];
        const INPUT_3: [u8; 4] = [0x09, 0x0a, ESC, 0x0c];
        const EXPECTED: &[u8] = &[
            0xc0, 0xdb, 0xdc, 0xdd, 0x01, 0x02, 0x03, ESC,
            ESC_ESC, 0x05, ESC, ESC_END, 0x07, 0x08, 0x09, 0x0a,
            ESC, ESC_ESC, 0x0c, 0xc0
        ];
        let mut output: [u8; 32] = [0; 32];

        let mut slip = Encoder::new();
        let mut bytes_written = slip.encode(&INPUT_1, &mut output).unwrap();
        let expected_bytes_written = 4 + INPUT_1.len() + 1;
        assert_eq!(expected_bytes_written, bytes_written);

        bytes_written += slip.encode(&INPUT_2, &mut output[bytes_written..]).unwrap();
        let expected_bytes_written = expected_bytes_written + INPUT_2.len() + 1;
        assert_eq!(expected_bytes_written, bytes_written);

        bytes_written += slip.encode(&INPUT_3, &mut output[bytes_written..]).unwrap();
        let expected_bytes_written = expected_bytes_written + INPUT_3.len() + 1;
        assert_eq!(expected_bytes_written, bytes_written);

        bytes_written += slip.finish(&mut output[bytes_written..]).unwrap();
        assert_eq!(expected_bytes_written + 1, bytes_written);
        assert_eq!(EXPECTED, &output[..bytes_written]);
    }
}
