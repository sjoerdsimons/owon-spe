use super::{parse_bool, query_digit_operation};
use crate::{Error, Operation};
use std::borrow::Cow;

pub(crate) struct Output {}
impl Operation for Output {
    type Out = bool;

    fn command(&self) -> Cow<'_, str> {
        Cow::Borrowed("OUTP?")
    }

    fn parse_line(&self, line: &str) -> Result<Self::Out, Error> {
        parse_bool(line).ok_or(Error::UnexpectedData)
    }
}

query_digit_operation!(Volt, "VOLT?", f32);
query_digit_operation!(VoltLimit, "VOLT:LIM?", f32);
query_digit_operation!(Current, "CURR?", f32);
query_digit_operation!(CurrentLimit, "CURR:LIM?", f32);

#[cfg(test)]
mod test {
    use super::*;
    use crate::operations::test_states;

    #[test]
    fn test_output() {
        test_states(
            Output {},
            &[
                ("ON", Ok(true)),
                ("1", Ok(true)),
                ("OFF", Ok(false)),
                ("0", Ok(false)),
            ],
        );
    }

    #[test]
    fn test_volt() {
        test_states(Volt {}, &[("1.000", Ok(1.000)), ("2.5", Ok(2.5))]);
    }
}
