pub mod iee4882;
pub mod measure;
pub mod output;

macro_rules! query_digit_operation {
    ($op:ident, $query:literal, $out:ident) => {
        pub(crate) struct $op {}
        impl Operation for $op {
            type Out = $out;

            fn command(&self) -> Cow<'_, str> {
                Cow::Borrowed($query)
            }

            fn parse_line(&self, line: &str) -> Result<Self::Out, Error> {
                line.parse().map_err(|_| Error::UnexpectedData)
            }
        }
    };
}

pub(crate) use query_digit_operation;

pub(crate) fn parse_bool(data: &str) -> Option<bool> {
    match data {
        "ON" | "1" => Some(true),
        "OFF" | "0" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
fn test_states<O>(op: O, states: &[(&str, Result<O::Out, crate::Error>)])
where
    O: crate::Operation,
{
    use crate::Error;

    for s in states {
        let out = op.parse_line(s.0);
        match (&s.1, &out) {
            (Ok(expected), Ok(out)) => assert_eq!(expected, out, "Input: {}", s.0),
            (Err(Error::UnexpectedData), Err(Error::UnexpectedData)) => (),
            (expected, out) => panic!("Expected: {:?} got {:?} for {}", expected, out, s.0),
        }
    }
}
