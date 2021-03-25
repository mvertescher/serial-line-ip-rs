use super::*;

/// SLIP encoder context
#[derive(Clone)]
pub struct Encoder {
    /// Just keep track of whether we have encoded the header yet
    header_written: bool,
}

/// The return type of `encode` that holds the bytes read and byte written after
/// the encode operation.
pub struct EncodeTotals {
    /// Number of bytes that were read from the input buffer.
    pub read: usize,
    /// Number of bytes that were written to the output buffer.
    pub written: usize,
}

impl Encoder {
    /// Create a new context for SLIP encoding
    pub fn new() -> Self {
        Encoder {
            header_written: false,
        }
    }

    /// Encode a buffer into a SLIP stream and returns the number of input bytes read
    /// and output bytes written.
    pub fn encode(&mut self, input: &[u8], output: &mut [u8]) -> Result<EncodeTotals> {
        let mut out_byte = 0;
        if !self.header_written {
            if output.len() < 1 {
                return Err(Error::NoOutputSpaceForHeader);
            }

            output[out_byte] = END;
            out_byte = 1;
            self.header_written = true;
        }

        let mut in_byte = 0;
        while in_byte < input.len() {
            match input[in_byte] {
                ESC => {
                    if (output.len() - out_byte) < 2 {
                        break;
                    }
                    output[out_byte] = ESC;
                    output[out_byte + 1] = ESC_ESC;
                    out_byte += 2;
                }
                END => {
                    if (output.len() - out_byte) < 2 {
                        break;
                    }
                    output[out_byte] = ESC;
                    output[out_byte + 1] = ESC_END;
                    out_byte += 2;
                }
                _ => {
                    if (output.len() - out_byte) < 1 {
                        break;
                    }
                    output[out_byte] = input[in_byte];
                    out_byte += 1;
                }
            }
            in_byte += 1;
        }

        Ok(EncodeTotals {
            read: in_byte,
            written: out_byte,
        })
    }

    /// Finish encoding the current packet and return the number of output bytes written.
    pub fn finish(self, output: &mut [u8]) -> Result<EncodeTotals> {
        if output.len() < 1 {
            return Err(Error::NoOutputSpaceForEndByte);
        }
        output[0] = END;

        Ok(EncodeTotals {
            read: 0,
            written: 1,
        })
    }
}

impl core::ops::AddAssign for EncodeTotals {
    fn add_assign(&mut self, other: EncodeTotals) {
        *self = EncodeTotals {
            read: self.read + other.read,
            written: self.written + other.written,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_encode() {
        const EXPECTED: [u8; 2] = [0xc0, 0xc0];
        let mut output: [u8; 32] = [0; 32];

        let mut slip = Encoder::new();
        let mut totals = slip.encode(&[0; 0], &mut output).unwrap();
        assert_eq!(0, totals.read);
        assert_eq!(1, totals.written);
        totals += slip.finish(&mut output[totals.written..]).unwrap();
        assert_eq!(0, totals.read);
        assert_eq!(2, totals.written);
        assert_eq!(&EXPECTED, &output[..totals.written]);
    }

    #[test]
    fn encode_esc_esc_sequence() {
        const INPUT: [u8; 3] = [0x01, ESC, 0x03];
        const EXPECTED: [u8; 6] = [0xc0, 0x01, ESC, ESC_ESC, 0x03, 0xc0];
        let mut output: [u8; 32] = [0; 32];

        let mut slip = Encoder::new();
        let mut totals = slip.encode(&INPUT, &mut output).unwrap();
        assert_eq!(2 + INPUT.len(), totals.written);
        totals += slip.finish(&mut output[totals.written..]).unwrap();
        assert_eq!(INPUT.len(), totals.read);
        assert_eq!(3 + INPUT.len(), totals.written);
        assert_eq!(&EXPECTED, &output[..totals.written]);
    }
    #[test]
    fn encode_end_esc_sequence() {
        const INPUT: [u8; 3] = [0x01, END, 0x03];
        const EXPECTED: [u8; 6] = [0xc0, 0x01, ESC, ESC_END, 0x03, 0xc0];
        let mut output: [u8; 32] = [0; 32];

        let mut slip = Encoder::new();
        let mut totals = slip.encode(&INPUT, &mut output).unwrap();
        assert_eq!(2 + INPUT.len(), totals.written);
        totals += slip.finish(&mut output[totals.written..]).unwrap();
        assert_eq!(INPUT.len(), totals.read);
        assert_eq!(3 + INPUT.len(), totals.written);
        assert_eq!(&EXPECTED, &output[..totals.written]);
    }

    #[test]
    fn multi_part_encode() {
        const INPUT_1: [u8; 4] = [0x01, 0x02, 0x03, ESC];
        const INPUT_2: [u8; 4] = [0x05, END, 0x07, 0x08];
        const INPUT_3: [u8; 4] = [0x09, 0x0a, ESC, 0x0c];
        const EXPECTED: &[u8] = &[
            0xc0, 0x01, 0x02, 0x03, ESC, ESC_ESC, 0x05, ESC, ESC_END, 0x07, 0x08, 0x09, 0x0a, ESC,
            ESC_ESC, 0x0c, 0xc0,
        ];
        let mut output: [u8; 32] = [0; 32];

        let mut slip = Encoder::new();
        let mut final_totals = EncodeTotals {
            read: 0,
            written: 0,
        };

        let totals = slip.encode(&INPUT_1, &mut output).unwrap();
        assert_eq!(INPUT_1.len(), totals.read);
        assert_eq!(1 + INPUT_1.len() + 1, totals.written);
        final_totals += totals;

        let totals = slip
            .encode(&INPUT_2, &mut output[final_totals.written..])
            .unwrap();
        assert_eq!(INPUT_2.len(), totals.read);
        assert_eq!(INPUT_2.len() + 1, totals.written);
        final_totals += totals;

        let totals = slip
            .encode(&INPUT_3, &mut output[final_totals.written..])
            .unwrap();
        assert_eq!(INPUT_3.len(), totals.read);
        assert_eq!(INPUT_3.len() + 1, totals.written);
        final_totals += totals;

        let totals = slip.finish(&mut output[final_totals.written..]).unwrap();
        assert_eq!(0, totals.read);
        assert_eq!(1, totals.written);
        final_totals += totals;

        assert_eq!(EXPECTED, &output[..final_totals.written]);
    }
}
