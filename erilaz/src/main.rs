use libruned::start;
use libruned::error::Error;


#[tokio::main]
async fn main() -> Result<(), Error> {
    start(Default::default()).await?;
    Err(Error::UnexpectedShutdown(
        "No futures left to process. This should not happen!".into(),
    ))
}
