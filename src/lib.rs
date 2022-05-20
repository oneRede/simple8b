mod simple8b;

use crate::simple8b::Encoder;

#[test]
fn test_create_encoder() {
    let encoder = Encoder {
        buf: [1; 240],
        h: 0,
        t: 0,
        bp: 0,
        bytes: [1; 1920],
        b: [1; 8],
    };
    println!("{:?}", encoder.buf);
}