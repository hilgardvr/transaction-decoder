use std::{io::{Error as ioError, Read}};
use sha2::{digest::Digest, Sha256};
use transaction::{Amount, Input, Output, Transaction, Txid};
use std::error::Error;
mod transaction;

fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, ioError> {
    let mut compact_size = [0_u8; 1];
    transaction_bytes.read(&mut compact_size)?;
    match compact_size[0] {
        0..=252 => Ok(compact_size[0] as u64),
        253 => {
            let mut buffer = [0; 2];
            transaction_bytes.read(&mut buffer)?;
            Ok(u16::from_le_bytes(buffer) as u64)
        }, 
        254 => {
            let mut buffer = [0; 4];
            transaction_bytes.read(&mut buffer)?;
            Ok(u32::from_le_bytes(buffer) as u64)
        },
        255 => {
            let mut buffer = [0; 8];
            transaction_bytes.read(&mut buffer)?;
            Ok(u64::from_le_bytes(buffer))
        }
    }
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, ioError> {
    let mut buffer = [0; 8];
    transaction_bytes.read(&mut buffer)?;
    Ok(Amount::from_sat(u64::from_le_bytes(buffer)))
}

fn read_u32(transaction_bytes: &mut &[u8]) -> Result<u32, ioError> {
    let mut buffer = [0_u8; 4];
    transaction_bytes.read(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, ioError> {
    let mut buffer = [0; 32];
    transaction_bytes.read(&mut buffer)?;
    Ok(Txid::from_bytes(buffer))
}

fn read_script(transaction_bytes: &mut &[u8]) -> Result<String, ioError> {
    let script_size = read_compact_size(transaction_bytes)?;
    let mut buffer = vec![0_u8; script_size as usize];
    transaction_bytes.read(&mut buffer)?;
    Ok(hex::encode(buffer))
}

fn hash_raw_transaction(raw_transaction: &[u8]) -> Txid {
    let mut hasher = Sha256::new();
    hasher.update(&raw_transaction);
    let hash1 = hasher.finalize();
    let mut hasher = Sha256::new();
    hasher.update(hash1);
    let hash2 = hasher.finalize();
    Txid::from_bytes(hash2.into())
}

fn decode(transaction_hex: String) -> Result<String, Box<dyn Error>> {
    //let transaction_hex = "01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff6403059d05e4b883e5bda9e7a59ee4bb99e9b1bcfabe6d6df3c53c3d9db8c2488121f2445e2665083387680896b4f9a69e0d3fd63334ac6510000000f09f909f4d696e656420627920756e6f00000000000000000000000000000000000000000000000000d0f80100015a7d7995000000001976a914c825a1ecf2a6830c4401620c3a16f1995057c2ab88acdb63af34";
    let transaction_bytes = hex::decode(transaction_hex).map_err(|e| format!("Hex decode error: {}", e))?;
    let mut bytes_slice = transaction_bytes.as_slice();
    let version = read_u32(&mut bytes_slice)?;
    let input_count = read_compact_size(&mut bytes_slice)?;
    let mut inputs = vec![];
    for _ in 0..input_count {
        let txid = read_txid(&mut bytes_slice)?;
        let output_index = read_u32(&mut bytes_slice)?;
        let script_sig = read_script(&mut bytes_slice)?;
        let sequence = read_u32(&mut bytes_slice)?;
        let input = Input {
            txid,
            output_index,
            script_sig,
            sequence
        };
        inputs.push(input);
    }
    let output_count = read_compact_size(&mut bytes_slice)?;
    let mut outputs = vec![]; 
    for _ in 0..output_count {
        let amount = read_amount(&mut bytes_slice)?;
        let script_pubkey = read_script(&mut bytes_slice)?;
        outputs.push(Output { amount, script_pubkey });
    }
    let lock_time = read_u32(&mut bytes_slice)?;
    let transaction_id = hash_raw_transaction(&transaction_bytes);
    let transaction = Transaction {
        version: version,
        inputs: inputs,
        outputs: outputs,
        lock_time: lock_time,
        transaction_id: transaction_id,
    };
    let json_inputs = serde_json::to_string_pretty(&transaction)?;
    println!("transaction: {}", json_inputs);
    Ok(json_inputs)
}

fn main() {
    let transaction_hex = "01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4d04ffff001d0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f757420666f722062616e6b73ffffffff0100f2052a01000000434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac00000000";
    match decode(transaction_hex.to_string()) {
        Ok(json) => println!("decoded: {}", json),
        Err(e) => println!("error: {}", e)
    }
}

#[cfg(test)]
mod test {
    use crate::read_u32;
    use super::Error;

    use super::read_compact_size;

    #[test]
    fn test_read_compact_size() -> Result<(), Box<dyn Error>> {
        let mut bytes = [1_u8].as_slice();
        let count = read_compact_size(&mut bytes)?;
        assert_eq!(count, 1_u64);

        let mut bytes = [253_u8, 0, 1].as_slice();
        let count = read_compact_size(&mut bytes)?;
        assert_eq!(count, 256_u64);

        let mut bytes = [254_u8, 0, 0, 0, 1].as_slice();
        let count = read_compact_size(&mut bytes)?;
        assert_eq!(count, 256_u64.pow(3));

        let mut bytes = [255_u8, 0, 0, 0, 0, 0, 0, 0, 1].as_slice();
        let count = read_compact_size(&mut bytes)?;
        assert_eq!(count, 256_u64.pow(7));

        let big_tx = "01000000fd204e";
        let hex = hex::decode(big_tx)?;
        let mut sl = hex.as_slice();
        let version = read_u32(&mut sl)?;
        let count = read_compact_size(&mut sl)?;
        assert_eq!(version, 1);
        assert_eq!(count, 20000);
        Ok(())
    }
}

