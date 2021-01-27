use crate::rune::v1::rune::Rune;
use crate::server::error::Error;

pub async fn create_model(_name: &str) -> Result<(), Error> {
    Ok(())
}

pub async fn configure_model() -> Result<(), Error> {
    Ok(())
}

pub async fn destroy_model() -> Result<(), Error> {
    Ok(())
}

pub async fn add_rune(_name: &str, _rune: &Rune) -> Result<(), Error> {
    Ok(())
}

pub async fn remove_rune(_name: &str) -> Result<(), Error> {
    Ok(())
}
