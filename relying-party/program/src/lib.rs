//! Record program
#![deny(missing_docs)]

mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod borsh_utils;

// Export current SDK types for downstream users building with a different SDK version
pub use solana_program;

solana_program::declare_id!("9fH5EdMT9ovEGvvd8NUhxT1XfyotiQvz3aHWNgWxyHZd");
