pub const HANDSHAKE: [u8; 12] = *b"BANDIT-START";
pub const HEADER_SIZE: usize = 13;

pub fn header(len: usize) -> [u8; HEADER_SIZE] {
    let len_bytes = len.to_le_bytes();

    [
        b'B',
        b'H',
        b'E',
        b'A',
        b'D',
        len_bytes[0],
        len_bytes[1],
        len_bytes[2],
        len_bytes[3],
        len_bytes[4],
        len_bytes[5],
        len_bytes[6],
        len_bytes[7],
    ]
}
