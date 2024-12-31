use std::{io::Read};
// use std::fmt;
use serde::{Serialize, Serializer};

#[derive(Debug, Serialize)]
struct Input {
    txid: String, //[u8; 32],
    output_index: u32,
    script_sig: String, //Vec<u8>,
    sequence: u32
}

#[derive(Debug, Serialize)]
struct Transaction {
    version: u32, 
    inputs: Vec<Input>,
    outputs: Vec<Output>
}

#[derive(Debug, Serialize)]
struct Output {
    #[serde(serialize_with = "as_btc")]
    amount: Amount,
    script_pubkey: String,
}

fn as_btc<S: Serializer, T: BitcoinValue>(t: &T, s: S) -> Result<S::Ok, S::Error> {
    let btc = t.to_btc();
    s.serialize_f64(btc)
}

trait BitcoinValue {
    fn to_btc(&self) -> f64;
}

#[derive(Debug)]
struct Amount(u64);

impl BitcoinValue for Amount {
    fn to_btc(&self) -> f64 {
        self.0 as f64 / 100_000_000.0
    }
}

//impl fmt::Debug for Input {
//    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//        f.debug_struct("Input")
//        .field("txid", &self.txid)
//        .field("output_index", &self.output_index)
//        .field("script_sig", &self.script_sig)
//        .field("sequence", &self.sequence)
//        .finish()
//    }
//}

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

fn read_amout(transaction_bytes: &mut &[u8]) -> Amount {
    let mut buffer = [0; 8];
    transaction_bytes.read(&mut buffer).unwrap();
    Amount(u64::from_le_bytes(buffer))
}

fn read_u32(transaction_bytes: &mut &[u8]) -> u32 {
    let mut buffer = [0_u8; 4];
    transaction_bytes.read(&mut buffer).unwrap();
    u32::from_le_bytes(buffer)
}

// fn read_txid(transaction_bytes: &mut &[u8]) -> [u8; 32] {
//     let mut buffer = [0; 32];
//     transaction_bytes.read(&mut buffer).unwrap();
//     buffer.reverse();
//     buffer
// }

fn read_txid(transaction_bytes: &mut &[u8]) -> String {
    let mut buffer = [0; 32];
    transaction_bytes.read(&mut buffer).unwrap();
    buffer.reverse();
    hex::encode(buffer)
}
// fn read_script(transaction_bytes: &mut &[u8]) -> Vec<u8> {
//     let script_size = read_compact_size(transaction_bytes);
//     let mut buffer = vec![0_u8; script_size as usize];
//     transaction_bytes.read(&mut buffer).unwrap();
//     buffer
// }

fn read_script(transaction_bytes: &mut &[u8]) -> String {
    let script_size = read_compact_size(transaction_bytes);
    let mut buffer = vec![0_u8; script_size as usize];
    transaction_bytes.read(&mut buffer).unwrap();
    hex::encode(buffer)
}

fn main() {
    let transaction_hex = "01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff6403059d05e4b883e5bda9e7a59ee4bb99e9b1bcfabe6d6df3c53c3d9db8c2488121f2445e2665083387680896b4f9a69e0d3fd63334ac6510000000f09f909f4d696e656420627920756e6f00000000000000000000000000000000000000000000000000d0f80100015a7d7995000000001976a914c825a1ecf2a6830c4401620c3a16f1995057c2ab88acdb63af34";
    let transaction_bytes = hex::decode(transaction_hex).unwrap();
    let mut bytes_slice = transaction_bytes.as_slice();
    let version = read_u32(&mut bytes_slice);
    let input_count = read_compact_size(&mut bytes_slice);
    let mut inputs = vec![];
    for _ in 0..input_count {
        let txid = read_txid(&mut bytes_slice);
        let output_index = read_u32(&mut bytes_slice);
        let script_sig = read_script(&mut bytes_slice);
        let sequence = read_u32(&mut bytes_slice);
        let input = Input {
            txid,
            output_index,
            script_sig,
            sequence
        };
        inputs.push(input);
    }
    let output_count = read_compact_size(&mut bytes_slice);
    let mut outputs = vec![]; 
    for _ in 0..output_count {
        let amount = read_amout(&mut bytes_slice);
        let script_pubkey = read_script(&mut bytes_slice);
        outputs.push(Output { amount, script_pubkey });
    }
    let transaction = Transaction {
        version: version,
        inputs: inputs,
        outputs: outputs
    };
    let json_inputs = serde_json::to_string_pretty(&transaction).unwrap();
    println!("transaction: {}", json_inputs)
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

