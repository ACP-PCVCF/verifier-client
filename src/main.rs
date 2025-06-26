use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub mod receiptverifier {
    tonic::include_proto!("receipt_verifier");
}

use receiptverifier::{
    receipt_verifier_service_client::ReceiptVerifierServiceClient,
    BytesChunk,
};

const CHUNK_SIZE_BYTES: usize = 3 * 1024 * 1024; // 3MB Chunks

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_addr = "http://127.0.0.1:50051";
    //let server_addr = "http://0.0.0.0:50051";
    let file_path = "receipt_output.json";
    //let file_path = "receipt_output.bin";

    let mut client = ReceiptVerifierServiceClient::connect(server_addr).await?;
    println!("Verbunden mit gRPC Server auf {}", server_addr);

    let path = Path::new(file_path);
    let mut file = File::open(path).await?;
    let file_size = file.metadata().await?.len();
    println!("Lese Datei: {} (Größe: {} bytes)", file_path, file_size);

    let stream = async_stream::stream! {
        let mut buffer = vec![0u8; CHUNK_SIZE_BYTES];
        loop {
            match file.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => {
                    let data_to_send = buffer[..n].to_vec();
                    println!("Sende Chunk mit {} Bytes", data_to_send.len());
                    yield BytesChunk { data: data_to_send };
                }
                Err(e) => {
                    eprintln!("Fehler beim Lesen der Datei: {}", e);
                    break;
                }
            }
        }
    };

    println!("Starte Stream zum Server...");
    match client.verify_receipt_stream(tonic::Request::new(stream)).await {
        Ok(response) => {
            let resp_inner = response.into_inner();
            println!("gRPC Antwort erhalten:");
            println!("  Valid: {}", resp_inner.valid);
            println!("  Message: {}", resp_inner.message);
            if let Some(val) = resp_inner.journal_value {
                println!("  Journal Value: {}", val);
            }
        }
        Err(e) => {
            eprintln!("gRPC Fehler: {:?}", e);
        }
    }

    Ok(())
}