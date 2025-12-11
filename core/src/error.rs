use thiserror::Error;

#[derive(Error, Debug)]
pub enum ZenithError {
    #[error("Arrow error: {0}")]
    ArrowError(#[from] arrow::error::ArrowError),

    #[error("WASM error: {0}")]
    WasmError(#[from] anyhow::Error),

    #[error("Buffer full")]
    BufferFull,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ZenithError>;
