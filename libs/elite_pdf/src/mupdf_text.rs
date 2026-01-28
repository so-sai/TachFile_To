// =========================================================================
// MISSION 008: STRUCTURED TEXT EXTRACTION
// =========================================================================
// Red Team Analysis: Binary-compatible FFI structs for MuPDF 1.27.0
// Platform: Windows x64
// Safety: Opaque pointers + #[repr(C)] for binary compatibility

use std::ffi::{c_float, c_int};

// =========================================================================
// 1. GEOMETRY STRUCTS (Already defined in lib.rs but duplicated for safety)
// =========================================================================

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FzRect {
    pub x0: c_float,
    pub y0: c_float,
    pub x1: c_float,
    pub y1: c_float,
}

impl FzRect {
    pub fn width(&self) -> f32 {
        self.x1 - self.x0
    }
    pub fn height(&self) -> f32 {
        self.y1 - self.y0
    }
}

// =========================================================================
// 2. TEXT STRUCTURE HIERARCHY (Page -> Block -> Line -> Char)
// =========================================================================

/// Individual character with font metadata
/// CRITICAL: size field is used for heading detection
#[repr(C)]
#[derive(Debug)]
pub struct FzStextChar {
    pub c: c_int,                    // Unicode character
    pub origin: [c_float; 2],        // Position (x, y)
    pub quad: [c_float; 4],          // Direction vectors
    pub size: c_float,               // Font size (HEADING DETECTION KEY)
    pub font: *mut std::ffi::c_void, // Opaque font pointer
    pub next: *mut FzStextChar,      // Linked list
}

/// Line of text (collection of characters)
#[repr(C)]
#[derive(Debug)]
pub struct FzStextLine {
    pub wmode: c_int,                 // Writing mode (0=horizontal)
    pub dir: [c_float; 2],            // Text direction
    pub bbox: FzRect,                 // Bounding box
    pub first_char: *mut FzStextChar, // First character in line
    pub last_char: *mut FzStextChar,  // Last character in line
    pub prev: *mut FzStextLine,       // Previous line
    pub next: *mut FzStextLine,       // Next line
}

/// Block container (Text or Image)
/// WARNING: Contains C union - handle with care
#[repr(C)]
#[derive(Debug)]
pub struct FzStextBlock {
    pub type_: c_int, // 0=Text, 1=Image
    pub bbox: FzRect, // Bounding box
    // C union: { text: *mut FzStextBlockText, image: *mut FzStextBlockImage }
    // We use raw bytes and cast when type_ == 0
    pub u: [*mut std::ffi::c_void; 2], // Union storage
    pub prev: *mut FzStextBlock,       // Previous block
    pub next: *mut FzStextBlock,       // Next block
}

/// Text-specific block (when block.type_ == 0)
#[repr(C)]
#[derive(Debug)]
pub struct FzStextBlockText {
    pub type_: c_int,                 // Should be 0
    pub bbox: FzRect,                 // Bounding box
    pub first_line: *mut FzStextLine, // First line in block
    pub last_line: *mut FzStextLine,  // Last line in block
    pub prev: *mut FzStextBlock,      // Previous block
    pub next: *mut FzStextBlock,      // Next block
}

/// Complete structured text page
/// SAFETY: All pointers are owned by this struct
#[repr(C)]
#[derive(Debug)]
pub struct FzStextPage {
    pub first_block: *mut FzStextBlock, // First block in page
    pub last_block: *mut FzStextBlock,  // Last block in page
}

// =========================================================================
// 3. FFI FUNCTION DECLARATIONS
// =========================================================================

unsafe extern "C" {
    // Extract structured text from page
    fn fz_new_stext_page_from_page(
        ctx: *mut crate::fz_context,
        page: *mut crate::fz_page,
        options: *const std::ffi::c_char,
    ) -> *mut FzStextPage;

    // Cleanup structured text
    fn fz_drop_stext_page(ctx: *mut crate::fz_context, stext: *mut FzStextPage);
}

// =========================================================================
// 4. SAFETY WRAPPERS (Mission 008 Implementation)
// =========================================================================

/// Safe wrapper for structured text page with lifetime tethering
pub struct EliteTextPage<'a> {
    ctx: &'a crate::EliteContext,
    page: &'a crate::ElitePage<'a>,
    inner: *mut FzStextPage,
}

impl<'a> EliteTextPage<'a> {
    /// Extract structured text from a page
    pub fn from_page(page: &'a crate::ElitePage) -> Result<Self, String> {
        unsafe {
            let stext =
                fz_new_stext_page_from_page(page.ctx.as_ptr(), page.inner, std::ptr::null());

            if stext.is_null() {
                return Err("Failed to extract structured text".to_string());
            }

            Ok(Self {
                ctx: &page.ctx,
                page,
                inner: stext,
            })
        }
    }

    /// Get iterator over text blocks
    pub fn blocks(&self) -> EliteTextBlockIterator {
        EliteTextBlockIterator {
            current: unsafe { (*self.inner).first_block },
            _phantom: std::marker::PhantomData,
        }
    }

    /// Convert to Markdown using 2-Pass Scanning algorithm
    pub fn to_markdown(&self) -> Result<String, String> {
        // Pass 1: Analyze font sizes for heading detection
        let font_histogram = self.analyze_font_sizes();

        // Pass 2: Generate Markdown with semantic structure
        self.generate_markdown(&font_histogram)
    }

    /// Pass 1: Build histogram of font sizes
    fn analyze_font_sizes(&self) -> std::collections::HashMap<u32, usize> {
        let mut histogram = std::collections::HashMap::new();

        for block in self.blocks() {
            if let BlockType::Text(text_block) = block.block_type() {
                for line in text_block.lines() {
                    for char_info in line.chars() {
                        let size = unsafe { (*char_info).size };
                        let size_int = (size * 10.0) as u32; // Convert to integer for hashing
                        *histogram.entry(size_int).or_insert(0) += 1;
                    }
                }
            }
        }

        histogram
    }

    /// Pass 2: Generate Markdown with heading detection
    fn generate_markdown(
        &self,
        histogram: &std::collections::HashMap<u32, usize>,
    ) -> Result<String, String> {
        let mut markdown = String::new();

        // Find base font size (most common)
        let base_size_int = histogram
            .iter()
            .max_by_key(|&(_, &count)| count)
            .map(|(&size, _)| size)
            .unwrap_or(120); // 12.0 * 10
        let base_size = base_size_int as f32 / 10.0;

        for block in self.blocks() {
            match block.block_type() {
                BlockType::Text(text_block) => {
                    for line in text_block.lines() {
                        let line_text = line.extract_text()?;

                        if line_text.trim().is_empty() {
                            markdown.push('\n');
                            continue;
                        }

                        // Heading detection based on font size
                        let first_char_size = line.first_char_size();
                        if let Some(size) = first_char_size {
                            if size > base_size * 1.5 {
                                markdown.push_str(&format!("# {}\n", line_text));
                            } else if size > base_size * 1.2 {
                                markdown.push_str(&format!("## {}\n", line_text));
                            } else {
                                // Check for indentation (lists/blockquotes)
                                let indent = line.indentation_level();
                                if indent > 20.0 {
                                    // 20 points threshold
                                    markdown.push_str(&format!("> {}\n", line_text));
                                } else if indent > 10.0 {
                                    markdown.push_str(&format!("- {}\n", line_text));
                                } else {
                                    markdown.push_str(&format!("{}\n", line_text));
                                }
                            }
                        } else {
                            markdown.push_str(&format!("{}\n", line_text));
                        }
                    }
                    markdown.push('\n'); // Block separator
                }
                BlockType::Image(_) => {
                    markdown.push_str("[IMAGE PLACEHOLDER]\n\n");
                }
                BlockType::Unknown => {
                    markdown.push_str("[UNKNOWN BLOCK TYPE]\n\n");
                }
            }
        }

        Ok(markdown)
    }
}

impl<'a> Drop for EliteTextPage<'a> {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                fz_drop_stext_page(self.ctx.as_ptr(), self.inner);
            }
            self.inner = std::ptr::null_mut();
        }
    }
}

// =========================================================================
// 5. ITERATORS (Zero-Copy Design)
// =========================================================================

pub struct EliteTextBlockIterator<'a> {
    current: *mut FzStextBlock,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Iterator for EliteTextBlockIterator<'a> {
    type Item = EliteTextBlock<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            return None;
        }

        let block = EliteTextBlock {
            inner: self.current,
            _phantom: std::marker::PhantomData,
        };

        unsafe {
            self.current = (*self.current).next;
        }

        Some(block)
    }
}

pub struct EliteTextBlock<'a> {
    inner: *mut FzStextBlock,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> EliteTextBlock<'a> {
    pub fn block_type(&self) -> BlockType<'a> {
        unsafe {
            match (*self.inner).type_ {
                0 => {
                    // Cast to text block
                    let text_block = self.inner as *mut FzStextBlockText;
                    BlockType::Text(EliteTextBlockText {
                        inner: text_block,
                        _phantom: std::marker::PhantomData,
                    })
                }
                1 => BlockType::Image(EliteTextBlockImage {
                    inner: self.inner,
                    _phantom: std::marker::PhantomData,
                }),
                _ => BlockType::Unknown,
            }
        }
    }
}

pub enum BlockType<'a> {
    Text(EliteTextBlockText<'a>),
    Image(EliteTextBlockImage<'a>),
    Unknown,
}

pub struct EliteTextBlockText<'a> {
    inner: *mut FzStextBlockText,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> EliteTextBlockText<'a> {
    pub fn lines(&self) -> EliteTextLineIterator<'a> {
        EliteTextLineIterator {
            current: unsafe { (*self.inner).first_line },
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct EliteTextBlockImage<'a> {
    inner: *mut FzStextBlock,
    _phantom: std::marker::PhantomData<&'a ()>,
}

pub struct EliteTextLineIterator<'a> {
    current: *mut FzStextLine,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Iterator for EliteTextLineIterator<'a> {
    type Item = EliteTextLine<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            return None;
        }

        let line = EliteTextLine {
            inner: self.current,
            _phantom: std::marker::PhantomData,
        };

        unsafe {
            self.current = (*self.current).next;
        }

        Some(line)
    }
}

pub struct EliteTextLine<'a> {
    inner: *mut FzStextLine,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> EliteTextLine<'a> {
    pub fn chars(&self) -> EliteTextCharIterator<'a> {
        EliteTextCharIterator {
            current: unsafe { (*self.inner).first_char },
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn extract_text(&self) -> Result<String, String> {
        let mut raw_bytes = Vec::new();

        // Step 1: Collect raw UTF-8 bytes from MuPDF
        for char_info in self.chars() {
            let unicode_val = unsafe { (*char_info).c };

            if let Some(c) = char::from_u32(unicode_val as u32) {
                // Encode to UTF-8 bytes first
                let mut buf = [0u8; 4];
                let utf8_bytes = c.encode_utf8(&mut buf);
                raw_bytes.extend_from_slice(utf8_bytes.as_bytes());
            }
        }

        // Step 2: Apply Mission 009 SIMD Sanitizer
        let clean_bytes = crate::sanitizer::SimdSanitizer::fast_clean(&raw_bytes);

        // Step 3: Convert back to String (guaranteed valid UTF-8)
        String::from_utf8(clean_bytes).map_err(|e| format!("UTF-8 conversion failed: {}", e))
    }

    pub fn first_char_size(&self) -> Option<f32> {
        let first_char = unsafe { (*self.inner).first_char };
        if first_char.is_null() {
            return None;
        }

        Some(unsafe { (*first_char).size })
    }

    pub fn indentation_level(&self) -> f32 {
        let bbox = unsafe { (*self.inner).bbox };
        bbox.x0 // Left margin position
    }
}

pub struct EliteTextCharIterator<'a> {
    current: *mut FzStextChar,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Iterator for EliteTextCharIterator<'a> {
    type Item = *mut FzStextChar;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            return None;
        }

        let char_info = self.current;

        unsafe {
            self.current = (*self.current).next;
        }

        Some(char_info)
    }
}
