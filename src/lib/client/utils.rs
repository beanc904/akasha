#![allow(dead_code)]

use base64::{Engine, engine::general_purpose};
use rand::Rng;

/// Generate WebSocket handshake key
pub fn generate_websocket_key() -> String {
    // generate 16 byte random number
    let mut rng = rand::rng();
    let mut key = [0u8; 16];
    rng.fill(&mut key);
    // base64 code
    general_purpose::STANDARD.encode(key)
}

// // parse chunked data, https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Transfer-Encoding#examples
// fn decode_chunked(data: &str) -> Result<String> {
//     let mut reader = BufReader::new(Cursor::new(data.as_bytes()));
//     let mut result = Vec::new();

//     loop {
//         let mut line = String::new();
//         reader.read_line(&mut line)?;
//         // parse chunk size (hexadecimal)
//         if let Ok(chunk_size) = usize::from_str_radix(line.trim(), 16) {
//             if chunk_size == 0 {
//                 break;
//             }
//             // read chunk size
//             let mut chunk = vec![0; chunk_size];
//             reader.read_exact(&mut chunk)?;
//             result.extend_from_slice(&chunk);
//             // skip \r\n separator
//             reader.read_line(&mut String::new())?;
//         } else {
//             log::error!("Failed to parse chunk size: {line}");
//             return Err(Error::HttpParseError(format!("Failed to parse chunk size: {line}")));
//         }
//     }
//     let body = String::from_utf8(result)?;
//     Ok(body)
// }
