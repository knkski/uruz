use crate::error::Error;
use crate::traits::Cloud;

struct AWS {}

impl Cloud for AWS {
    fn name(&self) -> String {
        "AWS".into()
    }
    fn create(&self) -> Result<(), Error> {
        Ok(())
    }
}
