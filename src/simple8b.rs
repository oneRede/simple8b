use byteorder::{BigEndian, ByteOrder};

#[allow(dead_code)]
const MAX_VALUE: u64 = (1 << 60) - 1;
const BUF_SIZE: usize = 240;

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct Encoder {
    pub buf: [u64; BUF_SIZE],
    pub h: usize,
    pub t: usize,
    pub bp: usize,
    pub bytes: [u8; BUF_SIZE * 8],
    pub b: [u8; 8],
}

#[allow(dead_code)]
impl Encoder {
    pub fn new() -> Self {
        Encoder {
            buf: [0; BUF_SIZE],
            h: 0,
            t: 0,
            bp: 0,
            bytes: [0; BUF_SIZE * 8],
            b: [0; 8],
        }
    }

    pub fn set_values(&mut self, v: [u64; BUF_SIZE]) {
        self.buf = v;
        self.t = self.buf.len();
        self.h = 0;
        self.bytes = [0; BUF_SIZE * 8]
    }

    pub fn reset(&mut self) {
        self.h = 0;
        self.t = 0;
        self.bp = 0;

        self.buf = [0u64; BUF_SIZE];
        self.bytes = [0u8; BUF_SIZE * 8];
        self.b = [0u8; 8];
    }

    pub fn write(&mut self, v: u64) {
        if self.t >= self.buf.len() {
            return;
        }

        if self.t >= self.buf.len() {
            self.buf = shift(self.buf, self.h);
            self.t -= self.h;
            self.h = 0;
        }
        self.buf[self.t] = v;
        self.t += 1;
    }

    pub fn flush(&mut self) {
        if self.t == 0 {
            return;
        }
        let mut ht: [u64; BUF_SIZE] = [0; BUF_SIZE];
        for i in 0..(self.t - self.h) {
            ht[i] = self.buf[self.h + i]
        }
        let (encoded, n) = encode(ht);
        BigEndian::write_u64(&mut self.b, encoded);

        if self.bp + 8 < self.bytes.len() {
            array_append(&mut self.bytes, self.b, self.bp);
            self.bp += 8
        }
        self.h += n;
        if self.h == self.t {
            self.h = 0;
            self.t = 0;
        }

        return ();
    }

    pub fn bytes(&self) -> &[u8] {
        return &self.bytes[..self.bp];
    }
}

#[allow(dead_code)]
pub struct Decoder {
    pub bytes: [u8; BUF_SIZE * 8],
    pub buf: [u64; BUF_SIZE],
    pub i: usize,
    pub n: usize,
}

#[allow(dead_code)]
impl Decoder {
    pub fn new(bytes: [u8; BUF_SIZE * 8]) -> Self {
        Decoder {
            bytes: bytes,
            buf: [0; BUF_SIZE],
            i: 0,
            n: 0,
        }
    }

    // Error due to the fix array
    pub fn next(&mut self) -> bool {
        self.i += 1;

        if self.i >= self.n {
            self.read();
        }

        return self.bytes.len() >= 8 || (self.i < self.n);
    }

    fn set_bytes(&mut self, bytes: [u8; BUF_SIZE * 8]) {
        self.bytes = bytes;
        self.i = 0;
        self.n = 0;
    }

    fn read(&self) -> u64 {
        return self.buf[self.i];
    }
}

#[allow(dead_code)]
struct Packing {
    n: usize,
    bit: usize,
    unpack: fn(u64, &mut [u64]),
    pack: fn(&[u64]) -> u64,
}

#[allow(dead_code)]
const SELECTOR: [Packing; 16] = [
    Packing {
        n: 240,
        bit: 0,
        unpack: unpack240,
        pack: pack240,
    },
    Packing {
        n: 120,
        bit: 0,
        unpack: unpack120,
        pack: pack120,
    },
    Packing {
        n: 60,
        bit: 1,
        unpack: unpack60,
        pack: pack60,
    },
    Packing {
        n: 30,
        bit: 2,
        unpack: unpack30,
        pack: pack30,
    },
    Packing {
        n: 20,
        bit: 3,
        unpack: unpack20,
        pack: pack20,
    },
    Packing {
        n: 15,
        bit: 4,
        unpack: unpack15,
        pack: pack15,
    },
    Packing {
        n: 12,
        bit: 5,
        unpack: unpack12,
        pack: pack12,
    },
    Packing {
        n: 10,
        bit: 6,
        unpack: unpack10,
        pack: pack10,
    },
    Packing {
        n: 8,
        bit: 7,
        unpack: unpack8,
        pack: pack8,
    },
    Packing {
        n: 7,
        bit: 8,
        unpack: unpack7,
        pack: pack7,
    },
    Packing {
        n: 6,
        bit: 10,
        unpack: unpack6,
        pack: pack6,
    },
    Packing {
        n: 5,
        bit: 12,
        unpack: unpack5,
        pack: pack5,
    },
    Packing {
        n: 4,
        bit: 15,
        unpack: unpack4,
        pack: pack4,
    },
    Packing {
        n: 3,
        bit: 20,
        unpack: unpack3,
        pack: pack3,
    },
    Packing {
        n: 2,
        bit: 30,
        unpack: unpack2,
        pack: pack2,
    },
    Packing {
        n: 1,
        bit: 60,
        unpack: unpack1,
        pack: pack1,
    },
];

fn count(v: u64) -> usize {
	let sel = v >> 60;
	if sel >= 16 {
		return 0
	}
	return SELECTOR[sel as usize].n
}

#[allow(dead_code)]
fn decode(dst: &mut [u64], v: u64) -> usize {
	let sel = v >> 60;
	if sel >= 16 {
		return 0
	}
	let unpack = SELECTOR[sel as usize].unpack;
    unpack(v, dst);
	return SELECTOR[sel as usize].n
}

#[allow(dead_code)]
fn decode_all(dst: &mut [u64], src: [u64; BUF_SIZE]) -> usize {
	let mut j = 0;
	for i in 0..src.len() {
        let v = src[i];
		let sel = v >> 60;
		if sel >= 16 {
			return 0
		}
        let unpack = SELECTOR[sel as usize].unpack;
        unpack(v, &mut dst[j..]);
        j += SELECTOR[sel as usize].n;
	}
	return j
}

fn array_append(src: &mut [u8; BUF_SIZE * 8], b: [u8; 8], index: usize) {
    for i in index..(index + 8) {
        src[i] = b[i - index]
    }
}

fn shift(mut array: [u64; BUF_SIZE], index: usize) -> [u64; BUF_SIZE] {
    for i in index..4 {
        array[i - index] = array[i]
    }
    array
}

fn can_pack(src: &[u64], n: usize, bits: usize) -> bool {
    if src.len() < n {
        return false;
    }

    let mut end = src.len();
    if n < end {
        end = n;
    }

    if bits == 0 {
        for v in src {
            if *v != 1 {
                return false;
            }
        }
        return true;
    }

    let max = ((1 << (bits as u64)) - 1) as u64;
    for i in 0..end {
        if src[i] > max {
            return false;
        }
    }
    true
}

#[allow(dead_code)]
fn pack240(src: &[u64]) -> u64 {
    return 0;
}

#[allow(dead_code)]
fn pack120(src: &[u64]) -> u64 {
    return 0;
}

#[allow(dead_code)]
fn pack60(src: &[u64]) -> u64 {
    return 2 << 60
        | src[0]
        | src[1] << 1
        | src[2] << 2
        | src[3] << 3
        | src[4] << 4
        | src[5] << 5
        | src[6] << 6
        | src[7] << 7
        | src[8] << 8
        | src[9] << 9
        | src[10] << 10
        | src[11] << 11
        | src[12] << 12
        | src[13] << 13
        | src[14] << 14
        | src[15] << 15
        | src[16] << 16
        | src[17] << 17
        | src[18] << 18
        | src[19] << 19
        | src[20] << 20
        | src[21] << 21
        | src[22] << 22
        | src[23] << 23
        | src[24] << 24
        | src[25] << 25
        | src[26] << 26
        | src[27] << 27
        | src[28] << 28
        | src[29] << 29
        | src[30] << 30
        | src[31] << 31
        | src[32] << 32
        | src[33] << 33
        | src[34] << 34
        | src[35] << 35
        | src[36] << 36
        | src[37] << 37
        | src[38] << 38
        | src[39] << 39
        | src[40] << 40
        | src[41] << 41
        | src[42] << 42
        | src[43] << 43
        | src[44] << 44
        | src[45] << 45
        | src[46] << 46
        | src[47] << 47
        | src[48] << 48
        | src[49] << 49
        | src[50] << 50
        | src[51] << 51
        | src[52] << 52
        | src[53] << 53
        | src[54] << 54
        | src[55] << 55
        | src[56] << 56
        | src[57] << 57
        | src[58] << 58
        | src[59] << 59;
}

fn pack30(src: &[u64]) -> u64 {
    return 3 << 60
        | src[0]
        | src[1] << 2
        | src[2] << 4
        | src[3] << 6
        | src[4] << 8
        | src[5] << 10
        | src[6] << 12
        | src[7] << 14
        | src[8] << 16
        | src[9] << 18
        | src[10] << 20
        | src[11] << 22
        | src[12] << 24
        | src[13] << 26
        | src[14] << 28
        | src[15] << 30
        | src[16] << 32
        | src[17] << 34
        | src[18] << 36
        | src[19] << 38
        | src[20] << 40
        | src[21] << 42
        | src[22] << 44
        | src[23] << 46
        | src[24] << 48
        | src[25] << 50
        | src[26] << 52
        | src[27] << 54
        | src[28] << 56
        | src[29] << 58;
}

fn pack20(src: &[u64]) -> u64 {
    return 4 << 60
        | src[0]
        | src[1] << 3
        | src[2] << 6
        | src[3] << 9
        | src[4] << 12
        | src[5] << 15
        | src[6] << 18
        | src[7] << 21
        | src[8] << 24
        | src[9] << 27
        | src[10] << 30
        | src[11] << 33
        | src[12] << 36
        | src[13] << 39
        | src[14] << 42
        | src[15] << 45
        | src[16] << 48
        | src[17] << 51
        | src[18] << 54
        | src[19] << 57;
}

fn pack15(src: &[u64]) -> u64 {
    return 5 << 60
        | src[0]
        | src[1] << 4
        | src[2] << 8
        | src[3] << 12
        | src[4] << 16
        | src[5] << 20
        | src[6] << 24
        | src[7] << 28
        | src[8] << 32
        | src[9] << 36
        | src[10] << 40
        | src[11] << 44
        | src[12] << 48
        | src[13] << 52
        | src[14] << 56;
}

fn pack12(src: &[u64]) -> u64 {
    return 6 << 60
        | src[0]
        | src[1] << 5
        | src[2] << 10
        | src[3] << 15
        | src[4] << 20
        | src[5] << 25
        | src[6] << 30
        | src[7] << 35
        | src[8] << 40
        | src[9] << 45
        | src[10] << 50
        | src[11] << 55;
}

fn pack10(src: &[u64]) -> u64 {
    return 7 << 60
        | src[0]
        | src[1] << 6
        | src[2] << 12
        | src[3] << 18
        | src[4] << 24
        | src[5] << 30
        | src[6] << 36
        | src[7] << 42
        | src[8] << 48
        | src[9] << 54;
}

fn pack8(src: &[u64]) -> u64 {
    return 8 << 60
        | src[0]
        | src[1] << 7
        | src[2] << 14
        | src[3] << 21
        | src[4] << 28
        | src[5] << 35
        | src[6] << 42
        | src[7] << 49;
}

fn pack7(src: &[u64]) -> u64 {
    return 9 << 60
        | src[0]
        | src[1] << 8
        | src[2] << 16
        | src[3] << 24
        | src[4] << 32
        | src[5] << 40
        | src[6] << 48;
}

fn pack6(src: &[u64]) -> u64 {
    return 10 << 60
        | src[0]
        | src[1] << 10
        | src[2] << 20
        | src[3] << 30
        | src[4] << 40
        | src[5] << 50;
}

fn pack5(src: &[u64]) -> u64 {
    return 11 << 60 | src[0] | src[1] << 12 | src[2] << 24 | src[3] << 36 | src[4] << 48;
}

fn pack4(src: &[u64]) -> u64 {
    return 12 << 60 | src[0] | src[1] << 15 | src[2] << 30 | src[3] << 45;
}

fn pack3(src: &[u64]) -> u64 {
    return 13 << 60 | src[0] | src[1] << 20 | src[2] << 40;
}

fn pack2(src: &[u64]) -> u64 {
    return 14 << 60 | src[0] | src[1] << 30;
}

fn pack1(src: &[u64]) -> u64 {
    return 15 << 60 | src[0];
}

fn encode(src: [u64; BUF_SIZE]) -> (u64, usize) {
    if can_pack(&src, 240, 0) {
        return (0, 240);
    } else if can_pack(&src, 120, 0) {
        return (1 << 60, 120);
    } else if can_pack(&src, 60, 1) {
        return (pack60(&src[..60]), 60);
    } else if can_pack(&src, 30, 2) {
        return (pack30(&src[..30]), 30);
    } else if can_pack(&src, 20, 3) {
        return (pack20(&src[..20]), 20);
    } else if can_pack(&src, 15, 4) {
        return (pack15(&src[..15]), 15);
    } else if can_pack(&src, 12, 5) {
        return (pack12(&src[..12]), 12);
    } else if can_pack(&src, 10, 6) {
        return (pack10(&src[..10]), 10);
    } else if can_pack(&src, 8, 7) {
        return (pack8(&src[..8]), 8);
    } else if can_pack(&src, 7, 8) {
        return (pack7(&src[..7]), 7);
    } else if can_pack(&src, 6, 10) {
        return (pack6(&src[..6]), 6);
    } else if can_pack(&src, 5, 12) {
        return (pack5(&src[..5]), 5);
    } else if can_pack(&src, 4, 15) {
        return (pack4(&src[..4]), 4);
    } else if can_pack(&src, 3, 20) {
        return (pack3(&src[..3]), 3);
    } else if can_pack(&src, 2, 30) {
        return (pack2(&src[..2]), 2);
    } else if can_pack(&src, 1, 60) {
        return (pack1(&src[..1]), 1);
    } else {
        if src.len() > 0 {
            return (0, 0);
        }
        return (0, 0);
    }
}

fn encode_all(src: [u64; BUF_SIZE]) -> [u64; BUF_SIZE] {
	let mut i = 0;

	// Re-use the input slice and write encoded values back in place
	let mut dst = src;
	let mut j = 0;

	loop {
		if i >= src.len() {
			break
		}
		let remaining = &src[i..];

		if can_pack(remaining, 240, 0) {
			dst[j] = 0;
			i += 240;
		} else if can_pack(remaining, 120, 0) {
			dst[j] = 1 << 60;
			i += 120;
		} else if can_pack(remaining, 60, 1) {
			dst[j] = pack60(&src[i..i+60]);
			i += 60
		} else if can_pack(remaining, 30, 2) {
			dst[j] = pack30(&src[i..i+30]);
			i += 30
		} else if can_pack(remaining, 20, 3) {
			dst[j] = pack20(&src[i..i+20]);
			i += 20
		} else if can_pack(remaining, 15, 4) {
			dst[j] = pack15(&src[i..i+15]);
			i += 15
		} else if can_pack(remaining, 12, 5) {
			dst[j] = pack12(&src[i..i+12]);
			i += 12
		} else if can_pack(remaining, 10, 6) {
			dst[j] = pack10(&src[i..i+10]);
			i += 10
		} else if can_pack(remaining, 8, 7) {
			dst[j] = pack8(&src[i..i+8]);
			i += 8
		} else if can_pack(remaining, 7, 8) {
			dst[j] = pack7(&src[i..i+7]);
			i += 7
		} else if can_pack(remaining, 6, 10) {
			dst[j] = pack6(&src[i..i+6]);
			i += 6
		} else if can_pack(remaining, 5, 12) {
			dst[j] = pack5(&src[i..i+5]);
			i += 5
		} else if can_pack(remaining, 4, 15) {
			dst[j] = pack4(&src[i..i+4]);
			i += 4
		} else if can_pack(remaining, 3, 20) {
			dst[j] = pack3(&src[i..i+3]);
			i += 3
		} else if can_pack(remaining, 2, 30) {
			dst[j] = pack2(&src[i..i+2]);
			i += 2
		} else if can_pack(remaining, 1, 60) {
			dst[j] = pack1(&src[i..i+1]);
			i += 1
		} else {
			return [0; BUF_SIZE]
		}
		j += 1
	}
	return dst
}

#[allow(dead_code)]
fn unpack240(v: u64, dst: &mut [u64]) {
    for i in 0..BUF_SIZE {
        dst[i] = 1
    }
}

#[allow(dead_code)]
fn unpack120(v: u64, dst: &mut [u64]) {
    for i in 0..BUF_SIZE {
        dst[i] = 1
    }
}

#[allow(dead_code)]
fn unpack60(v: u64, dst: &mut [u64]) {
    dst[0] = v & 1;
    dst[1] = (v >> 1) & 1;
    dst[2] = (v >> 2) & 1;
    dst[3] = (v >> 3) & 1;
    dst[4] = (v >> 4) & 1;
    dst[5] = (v >> 5) & 1;
    dst[6] = (v >> 6) & 1;
    dst[7] = (v >> 7) & 1;
    dst[8] = (v >> 8) & 1;
    dst[9] = (v >> 9) & 1;
    dst[10] = (v >> 10) & 1;
    dst[11] = (v >> 11) & 1;
    dst[12] = (v >> 12) & 1;
    dst[13] = (v >> 13) & 1;
    dst[14] = (v >> 14) & 1;
    dst[15] = (v >> 15) & 1;
    dst[16] = (v >> 16) & 1;
    dst[17] = (v >> 17) & 1;
    dst[18] = (v >> 18) & 1;
    dst[19] = (v >> 19) & 1;
    dst[20] = (v >> 20) & 1;
    dst[21] = (v >> 21) & 1;
    dst[22] = (v >> 22) & 1;
    dst[23] = (v >> 23) & 1;
    dst[24] = (v >> 24) & 1;
    dst[25] = (v >> 25) & 1;
    dst[26] = (v >> 26) & 1;
    dst[27] = (v >> 27) & 1;
    dst[28] = (v >> 28) & 1;
    dst[29] = (v >> 29) & 1;
    dst[30] = (v >> 30) & 1;
    dst[31] = (v >> 31) & 1;
    dst[32] = (v >> 32) & 1;
    dst[33] = (v >> 33) & 1;
    dst[34] = (v >> 34) & 1;
    dst[35] = (v >> 35) & 1;
    dst[36] = (v >> 36) & 1;
    dst[37] = (v >> 37) & 1;
    dst[38] = (v >> 38) & 1;
    dst[39] = (v >> 39) & 1;
    dst[40] = (v >> 40) & 1;
    dst[41] = (v >> 41) & 1;
    dst[42] = (v >> 42) & 1;
    dst[43] = (v >> 43) & 1;
    dst[44] = (v >> 44) & 1;
    dst[45] = (v >> 45) & 1;
    dst[46] = (v >> 46) & 1;
    dst[47] = (v >> 47) & 1;
    dst[48] = (v >> 48) & 1;
    dst[49] = (v >> 49) & 1;
    dst[50] = (v >> 50) & 1;
    dst[51] = (v >> 51) & 1;
    dst[52] = (v >> 52) & 1;
    dst[53] = (v >> 53) & 1;
    dst[54] = (v >> 54) & 1;
    dst[55] = (v >> 55) & 1;
    dst[56] = (v >> 56) & 1;
    dst[57] = (v >> 57) & 1;
    dst[58] = (v >> 58) & 1;
    dst[59] = (v >> 59) & 1;
}

fn unpack30(v: u64, dst: &mut [u64]) {
    dst[0] = v & 3;
    dst[1] = (v >> 2) & 3;
    dst[2] = (v >> 4) & 3;
    dst[3] = (v >> 6) & 3;
    dst[4] = (v >> 8) & 3;
    dst[5] = (v >> 10) & 3;
    dst[6] = (v >> 12) & 3;
    dst[7] = (v >> 14) & 3;
    dst[8] = (v >> 16) & 3;
    dst[9] = (v >> 18) & 3;
    dst[10] = (v >> 20) & 3;
    dst[11] = (v >> 22) & 3;
    dst[12] = (v >> 24) & 3;
    dst[13] = (v >> 26) & 3;
    dst[14] = (v >> 28) & 3;
    dst[15] = (v >> 30) & 3;
    dst[16] = (v >> 32) & 3;
    dst[17] = (v >> 34) & 3;
    dst[18] = (v >> 36) & 3;
    dst[19] = (v >> 38) & 3;
    dst[20] = (v >> 40) & 3;
    dst[21] = (v >> 42) & 3;
    dst[22] = (v >> 44) & 3;
    dst[23] = (v >> 46) & 3;
    dst[24] = (v >> 48) & 3;
    dst[25] = (v >> 50) & 3;
    dst[26] = (v >> 52) & 3;
    dst[27] = (v >> 54) & 3;
    dst[28] = (v >> 56) & 3;
    dst[29] = (v >> 58) & 3;
}

fn unpack20(v: u64, dst: &mut [u64]) {
    dst[0] = v & 7;
    dst[1] = (v >> 3) & 7;
    dst[2] = (v >> 6) & 7;
    dst[3] = (v >> 9) & 7;
    dst[4] = (v >> 12) & 7;
    dst[5] = (v >> 15) & 7;
    dst[6] = (v >> 18) & 7;
    dst[7] = (v >> 21) & 7;
    dst[8] = (v >> 24) & 7;
    dst[9] = (v >> 27) & 7;
    dst[10] = (v >> 30) & 7;
    dst[11] = (v >> 33) & 7;
    dst[12] = (v >> 36) & 7;
    dst[13] = (v >> 39) & 7;
    dst[14] = (v >> 42) & 7;
    dst[15] = (v >> 45) & 7;
    dst[16] = (v >> 48) & 7;
    dst[17] = (v >> 51) & 7;
    dst[18] = (v >> 54) & 7;
    dst[19] = (v >> 57) & 7;
}

fn unpack15(v: u64, dst: &mut [u64]) {
    dst[0] = v & 15;
    dst[1] = (v >> 4) & 15;
    dst[2] = (v >> 8) & 15;
    dst[3] = (v >> 12) & 15;
    dst[4] = (v >> 16) & 15;
    dst[5] = (v >> 20) & 15;
    dst[6] = (v >> 24) & 15;
    dst[7] = (v >> 28) & 15;
    dst[8] = (v >> 32) & 15;
    dst[9] = (v >> 36) & 15;
    dst[10] = (v >> 40) & 15;
    dst[11] = (v >> 44) & 15;
    dst[12] = (v >> 48) & 15;
    dst[13] = (v >> 52) & 15;
    dst[14] = (v >> 56) & 15;
}

fn unpack12(v: u64, dst: &mut [u64]) {
    dst[0] = v & 31;
    dst[1] = (v >> 5) & 31;
    dst[2] = (v >> 10) & 31;
    dst[3] = (v >> 15) & 31;
    dst[4] = (v >> 20) & 31;
    dst[5] = (v >> 25) & 31;
    dst[6] = (v >> 30) & 31;
    dst[7] = (v >> 35) & 31;
    dst[8] = (v >> 40) & 31;
    dst[9] = (v >> 45) & 31;
    dst[10] = (v >> 50) & 31;
    dst[11] = (v >> 55) & 31;
}

fn unpack10(v: u64, dst: &mut [u64]) {
    dst[0] = v & 63;
    dst[1] = (v >> 6) & 63;
    dst[2] = (v >> 12) & 63;
    dst[3] = (v >> 18) & 63;
    dst[4] = (v >> 24) & 63;
    dst[5] = (v >> 30) & 63;
    dst[6] = (v >> 36) & 63;
    dst[7] = (v >> 42) & 63;
    dst[8] = (v >> 48) & 63;
    dst[9] = (v >> 54) & 63;
}

fn unpack8(v: u64, dst: &mut [u64]) {
    dst[0] = v & 127;
    dst[1] = (v >> 7) & 127;
    dst[2] = (v >> 14) & 127;
    dst[3] = (v >> 21) & 127;
    dst[4] = (v >> 28) & 127;
    dst[5] = (v >> 35) & 127;
    dst[6] = (v >> 42) & 127;
    dst[7] = (v >> 49) & 127;
}

fn unpack7(v: u64, dst: &mut [u64]) {
    dst[0] = v & 255;
    dst[1] = (v >> 8) & 255;
    dst[2] = (v >> 16) & 255;
    dst[3] = (v >> 24) & 255;
    dst[4] = (v >> 32) & 255;
    dst[5] = (v >> 40) & 255;
    dst[6] = (v >> 48) & 255;
}

fn unpack6(v: u64, dst: &mut [u64]) {
    dst[0] = v & 1023;
    dst[1] = (v >> 10) & 1023;
    dst[2] = (v >> 20) & 1023;
    dst[3] = (v >> 30) & 1023;
    dst[4] = (v >> 40) & 1023;
    dst[5] = (v >> 50) & 1023;
}

fn unpack5(v: u64, dst: &mut [u64]) {
    dst[0] = v & 4095;
    dst[1] = (v >> 12) & 4095;
    dst[2] = (v >> 24) & 4095;
    dst[3] = (v >> 36) & 4095;
    dst[4] = (v >> 48) & 4095;
}

fn unpack4(v: u64, dst: &mut [u64]) {
    dst[0] = v & 32767;
    dst[1] = (v >> 15) & 32767;
    dst[2] = (v >> 30) & 32767;
    dst[3] = (v >> 45) & 32767;
}

fn unpack3(v: u64, dst: &mut [u64]) {
    dst[0] = v & 1048575;
    dst[1] = (v >> 20) & 1048575;
    dst[2] = (v >> 40) & 1048575;
}

fn unpack2(v: u64, dst: &mut [u64]) {
    dst[0] = v & 1073741823;
    dst[1] = (v >> 30) & 1073741823;
}

fn unpack1(v: u64, dst: &mut [u64]) {
    dst[0] = v & 1152921504606846975;
}
