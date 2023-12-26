pub trait Authanticator {
    fn auth(username: &str, password: &str) -> Result<u32, crate::Error>;
}

// A dummy authanticator that always rejects the login attempt.
impl Authanticator for () {
    fn auth(
        _username: &str,
        _password: &str,
    ) -> Result<u32, crate::Error> {
        Err(crate::Error::InvalidUsernameOrPassword)
    }
}
