use super::*;

/// SLIP decode context
pub struct Decoder {
    header_found: bool,
    esc_seq: [u8; 4],
    esc_seq_len: usize,
}

impl Decoder {
    /// Create a new context for SLIP decoding
    pub fn new() -> Self {
        Decoder {
            header_found: false,
            esc_seq: [0; 4],
            esc_seq_len: 0,
        }
    }

    /// SLIP decode the input slice into the output slice.
    ///
    /// This returns the number of bytes processed, an output slice and an indication of
    /// the end of the packet.
    pub fn decode<'a>(&mut self, input: &'a [u8], output: &'a mut [u8])
        -> Result<(usize, &'a [u8], bool)>
    {
        let input_len = input.len();
        let mut stream = input;
        if !self.header_found {
            stream = self.decode_header(stream)?;
        }
        let res = self.decode_stream(stream, output)?;

        Ok((input_len - res.0.len(), res.1, res.2))
    }

    /// Either process the header successfully or return an error
    fn decode_header<'a>(&mut self, input: &'a [u8]) -> Result<&'a [u8]> {
        if input.len() < 1 {
            // TODO: decode partial headers! For now, just error out...
            return Err(Error::BadHeaderDecode);
        }

        if input[0] != END {
            return Err(Error::BadHeaderDecode);
        }
        self.header_found = true;

        Ok(&input[1..])
    }

    /// Core stream processing
    fn decode_stream<'a>(&mut self, input: &'a [u8], output: &'a mut [u8])
        -> Result<(&'a [u8], &'a [u8], bool)>
    {
        let mut in_byte = 0;
        let mut out_byte = 0;
        let mut end = false;

        loop {
            if in_byte == input.len() || out_byte == output.len() {
                break;
            }

            if self.esc_seq_len > 0 {
                match input[in_byte] {
                    ESC_END => {
                        output[out_byte] = END
                    }
                    ESC_ESC => {
                        output[out_byte] = ESC
                    }
                    _ => return Err(Error::BadEscapeSequenceDecode),
                }
                out_byte += 1;
                self.esc_sequence_empty();
            } else {
                match input[in_byte] {
                    ESC => {
                        self.esc_sequence_push(ESC);
                    }
                    END => {
                        in_byte += 1;
                        end = true;
                        break;
                    }
                    _ => {
                        output[out_byte] = input[in_byte];
                        out_byte += 1;
                    }
                }
            }
            in_byte += 1;
        }

        Ok((&input[in_byte..], &output[..out_byte], end))
    }

    /// Push a byte onto the escape sequence
    fn esc_sequence_push(&mut self, byte: u8) {
        self.esc_seq[self.esc_seq_len] = byte;
        self.esc_seq_len += 1;
    }

    /// Reset the escape sequence
    fn esc_sequence_empty(&mut self) {
        self.esc_seq_len = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_decode() {
        const INPUT: [u8; 2] = [0xc0, 0xc0];
        let mut output: [u8; 32] = [0; 32];

        let mut slip = Decoder::new();
        let res = slip.decode(&INPUT, &mut output).unwrap();
        assert_eq!(INPUT.len(), res.0);
        assert_eq!(&[0;0], res.1);
        assert_eq!(true, res.2);
    }

    #[test]
    fn simple_decode() {
        const INPUT: [u8; 7] = [0xc0, 0x01, 0x02, 0x03, 0x04, 0x05, 0xc0];
        const DATA: [u8; 5] = [0x01, 0x02, 0x03, 0x04, 0x05];
        let mut output: [u8; 32] = [0; 32];

        let mut slip = Decoder::new();
        let res = slip.decode(&INPUT, &mut output).unwrap();
        assert_eq!(INPUT.len(), res.0);
        assert_eq!(&DATA, res.1);
        assert_eq!(true, res.2);
    }

    /// Ensure that [ESC, ESC_END] -> [END]
    #[test]
    fn decode_esc_then_esc_end_sequence() {
        const INPUT: [u8; 6] = [0xc0, 0x01, 0xdb, 0xdc, 0x03, 0xc0];
        const DATA: [u8; 3] = [0x01, 0xc0, 0x03];
        let mut output: [u8; 200] = [0; 200];

        let mut slip = Decoder::new();
        let res = slip.decode(&INPUT, &mut output).unwrap();
        assert_eq!(INPUT.len(), res.0);
        assert_eq!(&DATA, res.1);
        assert_eq!(true, res.2);
    }

    /// Ensure that [ESC, ESC_ESC] -> [ESC]
    #[test]
    fn decode_esc_then_esc_esc_sequence() {
        const INPUT: [u8; 6] = [0xc0, 0x01, 0xdb, 0xdd, 0x03, 0xc0];
        const DATA: [u8; 3] = [0x01, 0xdb, 0x03];
        let mut output: [u8; 200] = [0; 200];

        let mut slip = Decoder::new();
        let res = slip.decode(&INPUT, &mut output).unwrap();
        assert_eq!(INPUT.len(), res.0);
        assert_eq!(&DATA, res.1);
        assert_eq!(true, res.2);
    }

    #[test]
    fn multi_part_decode() {
        const INPUT_1: [u8; 6] = [0xc0, 0x01, 0x02, 0x03, 0x04, 0x05];
        const INPUT_2: [u8; 6] = [0x05, 0x06, 0x07, 0x08, 0x09, 0xc0];
        const DATA_1: [u8; 5] = [0x01, 0x02, 0x03, 0x04, 0x05];
        const DATA_2: [u8; 5] = [0x05, 0x06, 0x07, 0x08, 0x09];
        let mut output: [u8; 200] = [0; 200];

        let mut slip = Decoder::new();
        let mut offset = 0;
        {
            let res = slip.decode(&INPUT_1, &mut output[offset..]).unwrap();
            assert_eq!(INPUT_1.len(), res.0);
            assert_eq!(&DATA_1, res.1);
            assert_eq!(false, res.2);
            offset += res.1.len();
        }
        {
            let res = slip.decode(&INPUT_2, &mut output[offset..]).unwrap();
            assert_eq!(INPUT_2.len(), res.0);
            assert_eq!(&DATA_2, res.1);
            assert_eq!(true, res.2);
            offset += res.1.len();
        }
        assert_eq!(10, offset);
    }
}
