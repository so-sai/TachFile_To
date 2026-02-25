use super::node::Node;

/// The AstSink defines the streaming interface for building the Document AST.
///
/// Implementing this trait allows the system to process a document stream,
/// accumulating nodes, and flushing them to a persistent sink (like a Markdown file)
/// on section boundaries to keep RAM usage strictly bounded.
pub trait AstSink {
    /// Pushes a single parsed node into the current active buffer.
    fn push_node(&mut self, node: Node);

    /// Called when a structural boundary (like a new H1/H2) is encountered.
    /// The implementation must flush the current section to disk and release it from RAM.
    fn finalize_section(&mut self, title: String, level: u8) -> Result<(), String>;

    /// Flushes any remaining nodes at the end of the document stream.
    fn flush(&mut self) -> Result<(), String>;
}
