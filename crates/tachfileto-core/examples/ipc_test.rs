use serde_json;
use std::path::PathBuf;
use tachfileto_core::ipc::protocol::{
    BoundingBox, ExtractEvidencePayload, IpcMessage, MessageType,
};
use tachfileto_core::ipc::IpcManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("🚀 TachFileTo IPC Integration Test");
    eprintln!("{}", "=".repeat(60));

    // 1. Create IPC Manager
    let manager = IpcManager::new("python".to_string());

    // 2. Start Python worker
    eprintln!("\n📡 Starting Python worker...");
    manager.start().await?;

    // Wait for worker to initialization
    eprintln!("⏳ Waiting for worker initialization...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // 3. Send handshake
    eprintln!("\n🤝 Sending handshake...");
    match manager.handshake().await {
        Ok(response) => {
            eprintln!("✅ Handshake successful!");
            eprintln!("📄 Response: {}", serde_json::to_string_pretty(&response)?);
        }
        Err(e) => {
            eprintln!("❌ Handshake failed: {}", e);
            manager.stop().await?;
            return Err(e.into());
        }
    }

    // 4. Test Evidence Extraction (File Not Found Scenario)
    eprintln!("\n🖼️ Testing Evidence Extraction...");

    let evi_payload = ExtractEvidencePayload {
        file_path: PathBuf::from("non_existent_file.pdf"),
        page_index: 0,
        bbox: BoundingBox {
            x: 10.0,
            y: 10.0,
            width: 100.0,
            height: 50.0,
            unit: "pt".to_string(),
        },
        dpi: 72,
        output_format: Some("jpeg".to_string()),
        quality: Some(80),
    };

    let msg = IpcMessage::new(
        MessageType::CmdExtractEvidence,
        serde_json::to_value(evi_payload)?,
    );

    // We expect an error because file doesn't exist, but getting the error means the worker processed it!
    match manager.send_message(&msg).await {
        Ok(res) => {
            eprintln!(
                "❓ Unexpected success (should fail for missing file): {:?}",
                res
            );
        }
        Err(e) => {
            eprintln!(
                "✅ Received expected error from worker (File Not Found): {}",
                e
            );
            // Verify it's the correct error
            if e.contains("E-FILE-001")
                || e.contains("No such file")
                || e.contains("failed to open")
            {
                eprintln!("   (Error code verified)");
            } else {
                eprintln!("   (Warning: Error message different than expected: {})", e);
            }
        }
    }

    // 5. Stop worker
    eprintln!("\n🛑 Stopping worker...");
    manager.stop().await?;

    eprintln!("\n✅ Test completed successfully!");

    Ok(())
}
