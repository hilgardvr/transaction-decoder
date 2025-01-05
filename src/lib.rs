use transaction::{Decodable, Transaction};
use std::error::Error;
mod transaction;

pub fn decode(transaction_hex: String) -> Result<String, Box<dyn Error>> {
    let transaction_bytes = hex::decode(transaction_hex).map_err(|e| format!("Hex decode error: {}", e))?;
    let transaction = Transaction::consensus_decode(&mut transaction_bytes.as_slice())?;
    let json_inputs = serde_json::to_string_pretty(&transaction)?;
    //println!("transaction: {}", json_inputs);
    Ok(json_inputs)
}

//#[cfg(test)]
//mod test {
//    use crate::read_u32;
//    use super::Error;
//
//    use super::read_compact_size;
//
//    #[test]
//    fn test_read_compact_size() -> Result<(), Box<dyn Error>> {
//        let mut bytes = [1_u8].as_slice();
//        let count = read_compact_size(&mut bytes)?;
//        assert_eq!(count, 1_u64);
//
//        let mut bytes = [253_u8, 0, 1].as_slice();
//        let count = read_compact_size(&mut bytes)?;
//        assert_eq!(count, 256_u64);
//
//        let mut bytes = [254_u8, 0, 0, 0, 1].as_slice();
//        let count = read_compact_size(&mut bytes)?;
//        assert_eq!(count, 256_u64.pow(3));
//
//        let mut bytes = [255_u8, 0, 0, 0, 0, 0, 0, 0, 1].as_slice();
//        let count = read_compact_size(&mut bytes)?;
//        assert_eq!(count, 256_u64.pow(7));
//
//        let big_tx = "01000000fd204e";
//        let hex = hex::decode(big_tx)?;
//        let mut sl = hex.as_slice();
//        let version = read_u32(&mut sl)?;
//        let count = read_compact_size(&mut sl)?;
//        assert_eq!(version, 1);
//        assert_eq!(count, 20000);
//        Ok(())
//    }
//}
