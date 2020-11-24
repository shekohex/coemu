use runestick::{ContextError, Module};
use std::io;
use tokio::fs;

/// Construct the `fs` module.
pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::new(&["fs"]);
    module.async_function(&["read"], read)?;
    module.async_function(&["read_to_string"], read_to_string)?;
    Ok(module)
}

async fn read_to_string(path: &str) -> io::Result<String> {
    fs::read_to_string(path).await
}

async fn read(path: &str) -> io::Result<Vec<u8>> { fs::read(path).await }
