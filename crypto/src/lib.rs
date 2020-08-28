//! This crate contains cipher algorithms used between the Conquer
//! Online game client and server, it Defines generalized methods for ciphers
//! used by `TcpServerActor` and `TcpServerListener` for encrypting and
//! decrypting data to and from the game client.

mod rc5;
pub use rc5::TQRC5;

mod tq_cipher;
pub use tq_cipher::TQCipher;

/// Defines generalized methods for ciphers used by
/// `TcpServerActor` and `TcpServerListener` for encrypting and decrypting
/// data to and from the game client.
/// Can be used to switch between ciphers easily for
/// seperate states of the game client connection.
pub trait Cipher {
    /// Generates keys using key derivation variables.
    fn generate_keys(&self, key1: u32, key2: u32);
    /// Decrypts data from the client.
    ///
    /// * `src` - Source span that requires decrypting.
    /// * `dst` - Destination span to contain the decrypted result.
    fn decrypt(&self, src: &[u8], dst: &mut [u8]);

    /// Encrypts data to send to the client.
    ///
    /// * `src` - Source span that requires encrypting.
    /// * `dst` - Destination span to contain the encrypted result.
    fn encrypt(&self, src: &[u8], dst: &mut [u8]);
}
