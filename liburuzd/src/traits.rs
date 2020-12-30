use crate::error::Error;

pub trait Cloud {
    fn name(&self) -> String;
    fn create(&self) -> Result<(), Error>;
}

// pub trait Translator {
//     type Cloud: Cloud;
//     type Output;

//     fn translate(&self, rune: &Rune) -> Result<Self::Output, Error>;
// }

// pub trait Applier {
//     type Cloud: Cloud;
//     type Input;
//     fn apply(&self, input: Self::Input) -> Result<(), Error>;
// }
