const SHOW_USAGE: &str = "usage hash-pwd <password>";

fn main() -> Result<(), String> {
    let pwd = std::env::args().nth(1).ok_or_else(|| SHOW_USAGE.to_owned())?;
    let hashed = bcrypt::hash(pwd, bcrypt::DEFAULT_COST).map_err(|e| e.to_string())?;
    println!("{}", hashed);
    Ok(())
}
