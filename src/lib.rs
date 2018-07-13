//! Serial Line Internet Protocol (SLIP)
//!
//! See RFC 1055 for more information

/// Slip special character definitions
enum SlipChar {
    End    = 0xC0,
    Esc    = 0xDB,
    EscEnd = 0xDC,
    EscEsc = 0xDD,
}

/// Slip header definition
static HEADER: &'static [u8] = &[SlipChar::End as u8, SlipChar::Esc as u8,
    SlipChar::EscEnd as u8, SlipChar::EscEsc as u8];

/// Fully slip encode the input buffer to the output buffer
/// This will panic if the output buffer is less than 5 bytes
pub fn encode(input: &[u8], output: &mut [u8]) -> Result<usize, ()> {
    for i in 0..4 {
        output[i] = HEADER[i];
    }

    let mut offset = 4;
    for i in 0..input.len() {
        match input[i] {
            0xDB => {
                output[offset] = SlipChar::Esc as u8;
                output[offset+1] = SlipChar::EscEsc as u8;
                offset += 1;
            },
            0xC0 => {
                output[offset] = SlipChar::Esc as u8;
                output[offset+1] = SlipChar::EscEnd as u8;
                offset += 1;
            },
            _ => {
                output[offset] = input[i];
            },
        }
        offset += 1;
    }

    output[offset] = SlipChar::End as u8;
    offset += 1;

    Ok(offset)
}

/// Slip decode context structure
pub struct Slip {
    esc_seq: Vec<u8>,
}

impl Slip {
    /// Create a new context for slip decoding
    pub fn new() -> Self {
        Slip {
            esc_seq: Vec::with_capacity(4),
        }
    }

    /// Byte by byte slip decode
    pub fn decode<'a>(&mut self, input: &'a [u8], output: &'a mut [u8]) -> Result<(usize, &'a [u8]), ()> {
        let mut in_byte = 0;
        let mut out_byte = 0;
        for i in 0..input.len() {
            if out_byte > output.len() {
                in_byte = i;
                break;
            }

            if self.esc_seq.len() > 0 {
                if self.esc_seq[0] == SlipChar::End as u8 {
                    if input[i] == HEADER[self.esc_seq.len()] {
                        self.esc_seq.push(input[i]);
                        if self.esc_seq.len() == HEADER.len() {
                            self.esc_seq.drain(..);
                            in_byte = i + 1;
                            break;
                        }
                    } else {
                        self.esc_seq.pop();
                        continue;
                    }
                } else {
                    match input[i] {
                        0xDC => {
                            output[out_byte] = SlipChar::End as u8
                        },
                        0xDD => {
                            output[out_byte] = SlipChar::Esc as u8
                        },
                        _ => {
                            panic!("bad escape character");
                        },
                    }
                    self.esc_seq.drain(..);
                }
            } else {
                match input[i] {
                    0xDB => {
                        self.esc_seq.push(SlipChar::Esc as u8);
                    },
                    0xC0 => {
                        self.esc_seq.push(SlipChar::End as u8);
                        in_byte = i + 1;
                        break;
                    },
                    _ => {
                        output[out_byte] = input[i];
                        out_byte = out_byte + 1;
                    },
                }
            }
            in_byte = i + 1;
        }
        Ok((in_byte, &output[..out_byte]))
    }

    /// Decode a full SLIP encoded packet
    pub fn decode_packet<'a>(&mut self, input: &'a [u8], output: &'a mut [u8]) -> Result<(usize, &'a [u8]), ()> {
        let mut offset = 0;
        {
            let context = self.decode(&input[offset..], output).unwrap();
            offset = context.0;
        }
        {
            let context = self.decode(&input[offset..], output).unwrap();
            offset = offset + context.0;
        }

        let context = self.decode(&input[offset..], output).unwrap();
        offset = offset + context.0;

        Ok((offset, context.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_encode() {
        let mut output: [u8; 200] = [0; 200];
        let bytes_written = encode(&[0;0], &mut output).unwrap();
        let expected = [0xc0, 0xdb, 0xdc, 0xdd, 0xc0];
        assert_eq!(&expected, &output[..bytes_written]);
    }

    #[test]
    fn empty_decode() {
        let slipped = vec![0xc0, 0xdb, 0xdc, 0xdd, 0xc0];
        let mut output: [u8; 200] = [0; 200];

        let input = slipped.as_slice();
        let mut offset = 0;
        let mut slip = Slip::new();
        {
            let context = slip.decode(&input[offset..], &mut output).unwrap();
            assert_eq!(1, context.0);
            assert_eq!(&[0;0], context.1);
            offset = context.0;
        }
        {
            let context = slip.decode(&input[offset..], &mut output).unwrap();
            assert_eq!(3, context.0);
            assert_eq!(&[0;0], context.1);
            offset = offset + context.0;
        }
        {
            let context = slip.decode(&input[offset..], &mut output).unwrap();
            assert_eq!(1, context.0);
            assert_eq!(&[0;0], context.1);
            offset = offset + context.0;
        }
        assert_eq!(slipped.len(), offset);
    }
}
