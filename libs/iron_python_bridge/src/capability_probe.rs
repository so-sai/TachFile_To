/*
 * CAPABILITY PROBE - MISSION 014
 * ===============================
 * 
 * Non-extension based file identification (Magic Sniffing).
 */

use serde::{Serialize, Deserialize};
use std::path::Path;
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ExtractionLane {
    Pdf,
    Excel,
    Word,
    Markdown,
    Unknown,
}

pub struct CapabilityProbe;

impl CapabilityProbe {
    /// Detect the extraction lane based on file content (magic bytes)
    pub fn detect_lane<P: AsRef<Path>>(path: P) -> Result<ExtractionLane> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path)?;
        let mut buffer = [0; 4096]; // First 4KB
        let n = file.read(&mut buffer)?;
        
        if n < 4 {
            return Ok(ExtractionLane::Unknown);
        }

        // 1. PDF Sniff: %PDF-
        if &buffer[0..4] == b"%PDF" {
            return Ok(ExtractionLane::Pdf);
        }

        // 2. ZIP based (Office/OpenXML/etc): PK\x03\x04
        if &buffer[0..2] == b"PK" {
            // For now, assume Word if it's a ZIP magic. 
            // In a real implementation we would inspect the central directory for word/ or xl/
            return Ok(ExtractionLane::Word);
        }

        // 3. Markdown Sniff: Check if it's UTF-8 and starts with # or has typical MD markers
        // This is a heuristic.
        let content = String::from_utf8_lossy(&buffer[..n]);
        if content.starts_with('#') || content.contains("\n# ") || content.contains("**") {
            return Ok(ExtractionLane::Markdown);
        }

        Ok(ExtractionLane::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_lane_detection_accuracy() {
        // 1. PDF Magic: %PDF-
        let mut pdf_file = NamedTempFile::new().unwrap();
        pdf_file.write_all(b"%PDF-1.7\n% \xFB\xFC\xFD\xFE\n").unwrap();
        
        // 2. Office Magic (ZIP based): PK\x03\x04
        // Word/Excel are ZIPs. Sniffing needs deeper inspection for specific XMLs, 
        // but for RED PHASE we at least check zip.
        let mut zip_file = NamedTempFile::new().unwrap();
        zip_file.write_all(b"PK\x03\x04\n").unwrap();

        // 3. Markdown: Just text, maybe a header
        let mut md_file = NamedTempFile::new().unwrap();
        md_file.write_all(b"# Document Header\nContent here").unwrap();

        // Test Sniffing (Should FAIL or be Unknown in RED PHASE)
        assert_eq!(CapabilityProbe::detect_lane(pdf_file.path()).unwrap(), ExtractionLane::Pdf, "Should sniff PDF magic");
        assert_eq!(CapabilityProbe::detect_lane(zip_file.path()).unwrap(), ExtractionLane::Word, "Should sniff Office magic (Word)");
        assert_eq!(CapabilityProbe::detect_lane(md_file.path()).unwrap(), ExtractionLane::Markdown, "Should sniff Markdown text");
    }
}
