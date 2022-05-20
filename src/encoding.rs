const Max_value = (1 << 60) - 1

struct Encoder {
    buf: Vec<u8>,
    h: i32,
    t: i32,
    bp: i32,
    bytes: Vec<u8>,
    b: Vec<u8>,
}

impl Encoder {
    fn new() -> Self {
        Encoder {
            buf: Vec::<u8>::new(),
            h: 0,
            t: 0,
            bp: 0,
            bytes: Vec::<u8>::new(),
            b: Vec::<u8>::new(),
        }
    }

    fn set_values(&mut self, Vec<u8>) {
        
    }
}