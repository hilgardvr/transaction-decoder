use std::io::Read;


fn read_compact_size(transaction_bytes: &mut &[u8]) -> u64 {
    let mut compact_size = [0_u8; 1];
    transaction_bytes.read(&mut compact_size).unwrap();
    match compact_size[0] {
        0..=252 => compact_size[0] as u64,
        253 => {
            let mut buffer = [0; 2];
            transaction_bytes.read(&mut buffer).unwrap();
            u16::from_le_bytes(buffer) as u64
        }, 
        254 => {
            let mut buffer = [0; 4];
            transaction_bytes.read(&mut buffer).unwrap();
            u32::from_le_bytes(buffer) as u64
        },
        255 => {
            let mut buffer = [0; 8];
            transaction_bytes.read(&mut buffer).unwrap();
            u64::from_le_bytes(buffer)
        }
    }
}

fn read_u32(transaction_bytes: &mut &[u8]) -> u32 {
    let mut buffer = [0_u8; 4];
    transaction_bytes.read(&mut buffer).unwrap();
    u32::from_le_bytes(buffer)
}

fn read_txid(transaction_bytes: &mut &[u8]) -> [u8; 32] {
    let mut buffer = [0; 32];
    transaction_bytes.read(&mut buffer).unwrap();
    buffer.reverse();
    buffer
}

fn read_script(transaction_bytes: &mut &[u8]) -> Vec<u8> {
    let script_size = read_compact_size(transaction_bytes);
    let mut buffer = vec![0_u8; script_size as usize];
    transaction_bytes.read(&mut buffer).unwrap();
    buffer
}

fn main() {
    //let transaction_hex = "010000000242d5c1d6f730";
    let transaction_hex = "01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff6403059d05e4b883e5bda9e7a59ee4bb99e9b1bcfabe6d6df3c53c3d9db8c2488121f2445e2665083387680896b4f9a69e0d3fd63334ac6510000000f09f909f4d696e656420627920756e6f00000000000000000000000000000000000000000000000000d0f80100015a7d7995000000001976a914c825a1ecf2a6830c4401620c3a16f1995057c2ab88acdb63af34";
    let transaction_bytes = hex::decode(transaction_hex).unwrap();
    let mut bytes_slice = transaction_bytes.as_slice();
    let version = read_u32(&mut bytes_slice);
    let input_count = read_compact_size(&mut bytes_slice);
    for _ in 0..input_count {
        let txid = read_txid(&mut bytes_slice);
        let output_index = read_u32(&mut bytes_slice);
        let script_size = read_compact_size(&mut bytes_slice);
        let script_sig = read_script(&mut bytes_slice);
        let sequence = read_u32(&mut bytes_slice);
        println!("sequence: {}", sequence)
    }
    println!("Version: {}, compact size: {}", version, input_count)
}

#[cfg(test)]
mod test {
    use crate::read_u32;

    use super::read_compact_size;

    #[test]
    fn test_read_compact_size() {
        let mut bytes = [1_u8].as_slice();
        let count = read_compact_size(&mut bytes);
        assert_eq!(count, 1_u64);

        let mut bytes = [253_u8, 0, 1].as_slice();
        let count = read_compact_size(&mut bytes);
        assert_eq!(count, 256_u64);

        let mut bytes = [254_u8, 0, 0, 0, 1].as_slice();
        let count = read_compact_size(&mut bytes);
        assert_eq!(count, 256_u64.pow(3));

        let mut bytes = [255_u8, 0, 0, 0, 0, 0, 0, 0, 1].as_slice();
        let count = read_compact_size(&mut bytes);
        assert_eq!(count, 256_u64.pow(7));

        let big_tx = "01000000fd204e";
        let hex = hex::decode(big_tx).unwrap();
        let mut sl = hex.as_slice();
        let version = read_u32(&mut sl);
        let count = read_compact_size(&mut sl);
        assert_eq!(version, 1);
        assert_eq!(count, 20000)
    }
}

