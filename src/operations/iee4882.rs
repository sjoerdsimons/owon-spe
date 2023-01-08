use crate::{Error, Operation};
use std::borrow::Cow;

pub(crate) struct Idn {}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdnOutput {
    pub model: String,
    pub serial: String,
    pub fw: String,
}

impl Operation for Idn {
    type Out = IdnOutput;

    fn command(&self) -> Cow<'_, str> {
        Cow::Borrowed("*IDN?")
    }

    fn parse_line(&self, line: &str) -> Result<Self::Out, Error> {
        let mut parts = line.split(',');
        let _vendor = parts.next().ok_or(Error::UnexpectedData)?;
        let mut parts = parts.map(|d| d.to_owned());
        Ok(IdnOutput {
            model: parts.next().ok_or(Error::UnexpectedData)?,
            serial: parts.next().ok_or(Error::UnexpectedData)?,
            fw: parts.next().ok_or(Error::UnexpectedData)?,
        })
    }
}
