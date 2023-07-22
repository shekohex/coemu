/// Nop Cipher, does almost no work, Could be useful for testing or for using
/// internally between local servers to act as RPC.
#[derive(Copy, Clone, Debug, Default)]
pub struct NopCipher;

impl crate::Cipher for NopCipher {
    fn generate_keys(&self, _seed: u64) {}

    fn decrypt(&self, src: &[u8], dst: &mut [u8]) { dst.copy_from_slice(src); }

    fn encrypt(&self, src: &[u8], dst: &mut [u8]) { dst.copy_from_slice(src); }
}
