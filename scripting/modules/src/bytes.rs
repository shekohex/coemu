use bytes::Buf;
use runestick::{ContextError, Module};

#[derive(Debug, Clone)]
struct Bytes {
    inner: bytes::Bytes,
}

runestick::impl_external!(Bytes);

impl Bytes {
    pub fn new() -> Self {
        Self {
            inner: bytes::Bytes::new(),
        }
    }
}

impl Buf for Bytes {
    fn remaining(&self) -> usize { self.inner.remaining() }

    fn bytes(&self) -> &[u8] { self.inner.bytes() }

    fn advance(&mut self, cnt: usize) { self.inner.advance(cnt); }
}

/// Construct the `bytes` module.
pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::new(&["bytes"]);
    module.ty(&["Bytes"]).build::<Bytes>()?;

    module.function(&["Bytes", "new"], Bytes::new)?;

    module.inst_fn("get_u8", Bytes::get_u8)?;
    module.inst_fn("get_i8", Bytes::get_i8)?;

    Ok(module)
}
