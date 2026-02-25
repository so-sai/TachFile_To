use crate::ast::node::{Node, Section};
use crate::ast::sink::AstSink;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread;

/// The Streaming Ingestor provides a bounded channel architecture
/// for backpressured memory management during large PDF parsing.
pub struct StreamingIngestor<W: AstSink + Send + 'static> {
    pub capacity: usize,
    sink: Option<W>,
}

impl<W: AstSink + Send + 'static> StreamingIngestor<W> {
    pub fn new(capacity: usize, sink: W) -> Self {
        Self {
            capacity,
            sink: Some(sink),
        }
    }

    /// Spawns the processing thread and returns a channel sender for producer use.
    pub fn start(&mut self) -> SyncSender<Vec<Node>> {
        let (tx, rx): (SyncSender<Vec<Node>>, Receiver<Vec<Node>>) = sync_channel(self.capacity);

        let mut attached_sink = self
            .sink
            .take()
            .expect("Sink already attached to a running thread");

        // Spawn a dedicated consumer thread to avoid Deadlocks with Rayon
        thread::spawn(move || {
            let mut section_counter = 0;

            while let Ok(nodes) = rx.recv() {
                for node in nodes {
                    attached_sink.push_node(node);
                }

                // Simulate periodic section finalization per page/chunk
                section_counter += 1;
                let title = format!("Streaming Section {}", section_counter);

                // If it fails, log and continue, but in real use we'd bubble via oneshot
                let _ = attached_sink.finalize_section(title, 2);
            }

            let _ = attached_sink.flush();
        });

        tx
    }
}
