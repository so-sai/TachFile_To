use elite_pdf::EliteDocument;
use std::path::PathBuf;

fn get_test_pdf_path() -> String {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Current dir is libs/elite_pdf
    // We want to go up two levels to the workspace root
    d.pop(); // libs
    d.pop(); // app root
    
    d.push("docs");
    d.push("tests");
    d.push("pdf");
    d.push("254-cp.signed.pdf");
    
    let s = d.to_string_lossy().into_owned();
    if !d.exists() {
        // Fallback for direct workspace run or specific placement
        println!("WARNING: Test file not found at calculated path: {}", s);
        // Try relative path if running from root
        let rel = PathBuf::from("docs/tests/pdf/254-cp.signed.pdf");
        if rel.exists() {
            return rel.to_string_lossy().into_owned();
        }
        panic!("Test file not found at: {}", s);
    }
    s
}

#[test]
fn test_document_page_count() {
    let path = get_test_pdf_path();
    println!("Testing with PDF: {}", path);
    
    let doc = EliteDocument::new(path).expect("Failed to open document");
    let count = doc.page_count().expect("Failed to count pages");
    
    println!("Page Count: {}", count);
    assert_eq!(count, 112, "Page count should be 112 for 254-cp.signed.pdf");
}

#[test]
fn test_extract_all_text_no_panic() {
    let path = get_test_pdf_path();
    let doc = EliteDocument::new(path).expect("Failed to open document");
    
    let texts = doc.extract_all_text().expect("Failed to extract text");
    
    println!("Extracted {} pages of text placeholders", texts.len());
    assert_eq!(texts.len(), 112, "Should extract text for all 112 pages");
    
    if !texts.is_empty() {
        assert!(texts[0].starts_with("Page 1 extracted"));
    }
}
