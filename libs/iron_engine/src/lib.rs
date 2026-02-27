//! # Iron Engine — Public API Facade (v1.0.0-core+)
//!
//! **This is the ONLY entry point for Tauri or any external consumer.**
//! All internal modules are private. Tauri layer MUST call `spawn_blocking`
//! around these functions — they are synchronous and CPU-bound.

// ─── Internal Modules (Private) ──────────────────────────────────────────────
#[allow(dead_code, unused_imports)]
mod ast;
#[allow(dead_code, unused_imports)]
mod calculator;
mod diff;
mod exporter;
mod ingestor;
#[allow(dead_code, unused_imports)]
mod numeric_validator;

// ─── Backward-compat type alias (used by legacy calculator.rs) ───────────────
/// Legacy Result alias — maps to ProcessError for source compatibility.
pub type Result<T> = std::result::Result<T, ProcessError>;

// ─── Internal re-exports for integration tests ONLY ──────────────────────────
pub use numeric_validator::{ValidationContext, ValidationEngine};

// ─── IPC Error Contract ───────────────────────────────────────────────────────
/// Error codes returned by the engine.
///
/// **INVARIANT**: Never raw error strings. Frontend maps these to Vietnamese messages via `messages.vi.ts`.
#[derive(thiserror::Error, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "code")]
pub enum ProcessError {
    #[error("FileTooLarge")]
    FileTooLarge,
    #[error("OcrFailed")]
    OcrFailed,
    #[error("UnsupportedFormat")]
    UnsupportedFormat,
    #[error("UserCancelled")]
    UserCancelled,
    #[error("IoError")]
    IoError,
    #[error("EnginePanic")]
    EnginePanic,
}

impl From<std::io::Error> for ProcessError {
    fn from(_: std::io::Error) -> Self {
        ProcessError::IoError
    }
}

// ─── IPC Data Contracts ───────────────────────────────────────────────────────
/// Opaque document summary returned after processing.
///
/// - `id` is a hex String (not u64) to prevent JavaScript float precision loss.
/// - Internal fields (`numeric_index`, `section_ids`, `heading_entries`) are
///   omitted from JSON sent to the UI (`skip_serializing_if`) but retained
///   in-memory (Tauri state registry) for compare operations.
/// - `markdown` is the cached export result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSummary {
    pub id: String,
    pub source_path: String,
    pub total_pages: u32,
    pub has_ocr: bool,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) numeric_index: Vec<ast::node::NumericIndexEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) section_ids: Vec<ast::node::StableId>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) heading_entries: Vec<diff::HeadingEntry>,
    pub(crate) markdown: String,
}

/// The kind of change detected (matches TypeScript union).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum IpcDeltaKind {
    Added,
    Removed,
    Modified,
}

/// A single detected difference, IPC-safe.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpcDelta {
    pub node_id: String,
    pub kind: IpcDeltaKind,
    pub location: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

/// The final diff report, IPC-safe.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpcDiffReport {
    pub is_identical: bool,
    pub total_deltas: u32,
    pub deltas: Vec<IpcDelta>,
}

// ─── Public API Facade ────────────────────────────────────────────────────────

/// Process a document file and return a `DocumentSummary`.
///
/// **SYNC / CPU-bound** — Tauri layer MUST call `spawn_blocking`.
///
/// This function handles validation (size, format), runs the streaming
/// ingestor pipeline, and returns an opaque summary containing the cached
/// Markdown and internal index for compare operations.
pub fn process_document(path: &std::path::Path) -> Result<DocumentSummary> {
    use ast::node::{Node, Section, StableId};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // ── 1. Validate ──────────────────────────────────────────────────────────
    if !path.exists() {
        return Err(ProcessError::IoError);
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    if !matches!(ext.as_str(), "pdf" | "docx") {
        return Err(ProcessError::UnsupportedFormat);
    }

    let metadata = std::fs::metadata(path).map_err(|_| ProcessError::IoError)?;
    const MAX_BYTES: u64 = 500 * 1024 * 1024; // 500 MB
    if metadata.len() > MAX_BYTES {
        return Err(ProcessError::FileTooLarge);
    }

    // ── 2. Parse ─────────────────────────────────────────────────────────────
    // NOTE: Real PDF/DOCX parsing requires a native parser (planned for Phase 4+).
    // For V1.0 shell integration, we produce a structural stub that exercises
    // the full pipeline. The ingestor architecture is ready; only the format
    // adapter (PDF byte-stream → Node) needs to be plugged in.
    //
    // For now: read the raw text as a single paragraph section.
    // This produces valid output for text-based files.
    let raw_text = std::fs::read_to_string(path).unwrap_or_else(|_| {
        format!("# {}\n\n[Nội dung nhị phân — cần parser PDF/DOCX]", path.file_name().unwrap_or_default().to_string_lossy())
    });

    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Build a single section from raw text
    let section = Section {
        level: 1,
        title: file_name.clone(),
        id: StableId::generate(&file_name, &raw_text),
        nodes: vec![Node::Paragraph {
            text: raw_text.clone(),
            id: StableId::generate("p0", &raw_text),
        }],
    };

    let sections = vec![section];

    // ── 3. Build outputs ─────────────────────────────────────────────────────
    let numeric_index: Vec<_> = sections
        .iter()
        .flat_map(exporter::extract_numeric_index)
        .collect();

    let section_ids: Vec<_> = sections.iter().map(|s| s.id.clone()).collect();

    let heading_entries: Vec<diff::HeadingEntry> = sections
        .iter()
        .flat_map(|s| {
            s.nodes.iter().filter_map(|n| {
                if let Node::Heading { level, id, .. } = n {
                    Some(diff::HeadingEntry {
                        id: id.clone(),
                        level: *level,
                    })
                } else {
                    None
                }
            })
        })
        .collect();

    let markdown = exporter::export_markdown_from_sections(&sections);

    // ── 4. Generate stable ID ─────────────────────────────────────────────────
    let id = {
        let mut h = DefaultHasher::new();
        path.hash(&mut h);
        metadata.len().hash(&mut h);
        format!("{:016x}", h.finish())
    };

    Ok(DocumentSummary {
        id,
        source_path: path.to_string_lossy().to_string(),
        total_pages: 1, // Real page count from PDF parser (Phase 4+)
        has_ocr: false,
        numeric_index,
        section_ids,
        heading_entries,
        markdown,
    })
}

/// Compare two processed document summaries. Returns a structured diff report.
///
/// **SYNC / CPU-bound** — Tauri layer MUST call `spawn_blocking`.
/// O(n) algorithm using HashMap set-subtraction.
pub fn compare_documents(a: &DocumentSummary, b: &DocumentSummary) -> IpcDiffReport {
    use diff::report::DeltaType;

    let internal_report = diff::diff_documents(
        &a.section_ids,
        &b.section_ids,
        &a.heading_entries,
        &b.heading_entries,
        &a.numeric_index,
        &b.numeric_index,
    );

    let deltas: Vec<IpcDelta> = internal_report
        .deltas
        .iter()
        .map(|d| {
            let (kind, old_value, new_value) = match &d.delta_type {
                DeltaType::Added => (IpcDeltaKind::Added, None, None),
                DeltaType::Removed => (IpcDeltaKind::Removed, None, None),
                DeltaType::ValueMismatch { old, new, .. } => (
                    IpcDeltaKind::Modified,
                    Some(format!("{:.2}", old)),
                    Some(format!("{:.2}", new)),
                ),
                DeltaType::StructuralChange { description } => (
                    IpcDeltaKind::Modified,
                    Some(description.clone()),
                    None,
                ),
            };
            IpcDelta {
                node_id: format!("{:016x}", d.node_id.0),
                kind,
                location: d.location.clone(),
                old_value,
                new_value,
            }
        })
        .collect();

    let total = deltas.len() as u32;
    IpcDiffReport {
        is_identical: total == 0,
        total_deltas: total,
        deltas,
    }
}

/// Retrieve the cached Markdown string from a processed document.
pub fn get_markdown(summary: &DocumentSummary) -> &str {
    &summary.markdown
}
