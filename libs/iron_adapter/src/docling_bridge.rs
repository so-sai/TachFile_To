use iron_table::contract::{TableTruth, TableRow, TableCell, CellValue, BoundingBox, EncodingStatus, TableSchema, ColumnDef, DataType, ExtractionMeta};
use crate::gatekeeper::EncodingGatekeeper;
use thiserror::Error;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::BufReader;
use serde::Deserialize;
use std::collections::BTreeMap;
use chrono::Local;

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Validation error: {0}")]
    Validation(String),
}

// --- Docling V2/V3 Schema Definitions ---

#[derive(Debug, Deserialize)]
struct DoclingDocument {
    pages: Vec<DoclingPage>,
}

#[derive(Debug, Deserialize)]
struct DoclingPage {
    page_no: u32,
    size: DoclingSize,
    tables: Vec<DoclingTable>,
}

#[derive(Debug, Deserialize)]
struct DoclingSize {
    width: f64,
    height: f64,
}

#[derive(Debug, Deserialize)]
struct DoclingTable {
    #[serde(rename = "self_ref")]
    _self_ref: String,
    cells: Vec<DoclingCell>,
}

#[derive(Debug, Deserialize)]
struct DoclingCell {
    row_index: usize,
    col_index: usize,
    content: DoclingContent,
    #[serde(rename = "box_2d")]
    bbox: [f64; 4], // [x0, y0, x1, y1]
    confidence: f64,
}

#[derive(Debug, Deserialize)]
struct DoclingContent {
    text: String,
}

// --- Bridge Implementation ---

pub struct DoclingBridge;

impl DoclingBridge {
    /// Ingests a Docling JSON file and converts it into a vector of valid TableTruths.
    pub fn ingest(path: &Path) -> Result<Vec<TableTruth>, BridgeError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let doc: DoclingDocument = serde_json::from_reader(reader)?;
        
        let mut truths = Vec::new();

        for page in doc.pages {
            for (idx, table) in page.tables.into_iter().enumerate() {
                let truth = Self::map_table(table, page.page_no, path, idx, &page.size)?;
                truths.push(truth);
            }
        }

        Ok(truths)
    }

    fn map_table(
        docling_table: DoclingTable, 
        page_no: u32, 
        source_path: &Path, 
        table_idx: usize,
        page_size: &DoclingSize
    ) -> Result<TableTruth, BridgeError> {
        let mut cells_by_row: BTreeMap<usize, Vec<DoclingCell>> = BTreeMap::new();
        let mut max_col = 0;
        let mut max_row = 0;

        // 1. Group cells and find dimensions
        for cell in docling_table.cells {
            if cell.col_index > max_col { max_col = cell.col_index; }
            if cell.row_index > max_row { max_row = cell.row_index; }
            cells_by_row.entry(cell.row_index).or_default().push(cell);
        }

        let col_count = max_col + 1;
        let row_count = max_row + 1;

        // 2. Build Rows & Cells
        let mut table_rows = Vec::new();
        
        for r in 0..row_count {
            let mut row_cells = Vec::new();
            if let Some(d_cells) = cells_by_row.get(&r) {
                // Determine row-level cells. 
                // Note: Docling might have sparse cells. 
                // IronTable expects dense cells? `TableTruth` validation checks row.cells.len() == schema.col_count
                
                // We need to fill holes with Null/Empty cells to ensure density
                let mut d_cell_map: BTreeMap<usize, &DoclingCell> = BTreeMap::new();
                for c in d_cells {
                    d_cell_map.insert(c.col_index, c);
                }

                for c in 0..col_count {
                    if let Some(dc) = d_cell_map.get(&c) {
                        // Scan for encoding
                        let (enc_status, enc_evidence) = EncodingGatekeeper::scan(&dc.content.text);
                        
                        // Numeric detection heuristic
                        let value = if let Ok(i) = dc.content.text.parse::<i64>() {
                            CellValue::Int(i)
                        } else if let Ok(f) = dc.content.text.replace(',', "").parse::<f64>() {
                            CellValue::Float(f)
                        } else {
                            CellValue::Text(dc.content.text.clone())
                        };

                        let cell = TableCell {
                            row_idx: dc.row_index,
                            col_idx: dc.col_index,
                            value,
                            bbox: BoundingBox {
                                x: dc.bbox[0] as f32,
                                y: dc.bbox[1] as f32,
                                width: (dc.bbox[2] - dc.bbox[0]) as f32,
                                height: (dc.bbox[3] - dc.bbox[1]) as f32,
                                page: page_no,
                            },
                            confidence: dc.confidence as f32,
                            source_text: dc.content.text.clone(),
                            encoding_status: enc_status,
                            encoding_evidence: enc_evidence,
                        };
                        row_cells.push(cell);
                    } else {
                        // Fill missing cell
                        row_cells.push(TableCell {
                            row_idx: r,
                            col_idx: c,
                            value: CellValue::Null,
                            bbox: BoundingBox { x:0.0, y:0.0, width:0.0, height:0.0, page: page_no },
                            confidence: 0.0,
                            source_text: String::new(),
                            encoding_status: EncodingStatus::Clean,
                            encoding_evidence: None,
                        });
                    }
                }
            } else {
                 // Empty row
                 for c in 0..col_count {
                    row_cells.push(TableCell {
                        row_idx: r,
                        col_idx: c,
                        value: CellValue::Null,
                        bbox: BoundingBox { x:0.0, y:0.0, width:0.0, height:0.0, page: page_no },
                        confidence: 0.0,
                        source_text: String::new(),
                        encoding_status: EncodingStatus::Clean,
                        encoding_evidence: None,
                    });
                 }
            }
            table_rows.push(TableRow { row_idx: r, cells: row_cells });
        }

        // 3. Build Schema
        // Default to Generic Columns for now since we haven't implemented header detection logic from the user
        let mut columns = Vec::new();
        for i in 0..col_count {
            columns.push(ColumnDef {
                name: format!("Column {}", i + 1),
                dtype: DataType::Utf8, // Bridge deals in strings
                unit: None,
                nullable: true,
                is_critical: false,
            });
        }

        let schema = TableSchema {
            columns,
            row_count,
            col_count,
        };

        // 4. Construct TableTruth
        Ok(TableTruth {
            table_id: format!("{}-{}-{}", source_path.file_stem().unwrap().to_string_lossy(), page_no, table_idx),
            source_file: PathBuf::from(source_path),
            source_page: page_no,
            schema,
            rows: table_rows,
            extraction_meta: ExtractionMeta {
                tool_version: "Docling V2".to_string(),
                timestamp: Local::now().to_rfc3339(),
                confidence_score: 1.0, // Aggregate could be calc'd but 1.0 for the container
            },
            bbox: BoundingBox { // Page-level or Table-level union bbox? Using placeholder.
                x: 0.0, y: 0.0, width: page_size.width as f32, height: page_size.height as f32, page: page_no
            },
        })
    }
}

// --- Project Assembler ---

pub struct ProjectAssembler;

impl ProjectAssembler {
    /// Assembles a ProjectTruth from a collection of TableTruths.
    /// This is where we would implement logic to link Summary tables to Detail tables.
    /// For Mission 024, we provide a basic pass-through or simple aggregation.
    pub fn assemble(_tables: Vec<TableTruth>) -> Result<iron_table::contract::ProjectTruth, BridgeError> {
        // TODO: Implement actual assembly logic (grouping, validation)
        // For now, return a dummy ProjectTruth to satisfy the interface.
        // Real implementation requires detailed business rules (Mission 023).
        
        use iron_table::contract::{ProjectTruth, ProjectStatus, Financials, DeviationSummary, SystemMetrics};

        Ok(ProjectTruth {
            project_name: "Docling Import".to_string(),
            last_updated: Local::now().to_rfc3339(),
            data_source: "Docling V2".to_string(),
            project_status: ProjectStatus::Safe,
            status_reason: "Imported from Docling".to_string(),
            financials: Financials {
                total_cost: 0.0,
                total_paid: 0.0,
                remaining: 0.0,
            },
            deviation: DeviationSummary {
                percentage: 0.0,
                absolute: 0.0,
            },
            top_risks: vec![],
            pending_actions: vec![],
            metrics: SystemMetrics {
                table_count: _tables.len(),
                row_count: _tables.iter().map(|t| t.rows.len()).sum(),
                processing_time_ms: 0,
            },
        })
    }
}
