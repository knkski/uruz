use crate::rune::v1::rune::Rune;
use crate::server::error::Error;

pub async fn create_model(_name: &str) -> Result<(), Error> {
    unimplemented!()
}

pub async fn configure_model() -> Result<(), Error> {
    unimplemented!()
}

pub async fn destroy_model() -> Result<(), Error> {
    unimplemented!()
}

pub async fn add_rune(_name: &str, _rune: &Rune) -> Result<(), Error> {
    unimplemented!()
}

pub async fn configure_rune(_name: &str, _attr: &str, _val: &str) -> Result<(), Error> {
    unimplemented!()
}

pub async fn remove_rune(_name: &str) -> Result<(), Error> {
    unimplemented!()
}
