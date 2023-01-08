use super::{parse_bool, query_digit_operation};
use crate::{Error, Operation};
use std::borrow::Cow;

query_digit_operation!(Volt, "MEAS:VOLT?", f32);
query_digit_operation!(Current, "MEAS:Current?", f32);
query_digit_operation!(Power, "MEAS:Power?", f32);

fn pop_f32<'a, I: Iterator<Item = &'a str>>(mut i: I) -> Result<f32, Error> {
    i.next()
        .ok_or(Error::UnexpectedData)?
        .parse()
        .or(Err(Error::UnexpectedData))
}

fn pop_bool<'a, I: Iterator<Item = &'a str>>(mut i: I) -> Result<bool, Error> {
    let b = i.next().ok_or(Error::UnexpectedData)?;
    parse_bool(b).ok_or(Error::UnexpectedData)
}

pub(crate) struct MeasureAll {}
#[derive(Debug, Clone, PartialEq)]
pub struct MeasureAllOutput {
    pub volt: f32,
    pub current: f32,
}

impl Operation for MeasureAll {
    type Out = MeasureAllOutput;

    fn command(&self) -> Cow<'_, str> {
        Cow::Borrowed("MEAS:ALL?")
    }

    fn parse_line(&self, line: &str) -> Result<Self::Out, Error> {
        let mut parts = line.split(',');
        Ok(MeasureAllOutput {
            volt: pop_f32(&mut parts)?,
            current: pop_f32(&mut parts)?,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Mode {
    Standby = 0,
    ConstantVoltage = 1,
    ConstantCurrent = 2,
    Failed = 3,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Standby => f.write_str("Standby"),
            Mode::ConstantVoltage => f.write_str("Constant Voltage"),
            Mode::ConstantCurrent => f.write_str("Constant Current"),
            Mode::Failed => f.write_str("Failed"),
        }
    }
}

pub(crate) struct MeasureAllInfo {}
#[derive(Debug, Clone, PartialEq)]
pub struct MeasureAllInfoOutput {
    pub volt: f32,
    pub current: f32,
    pub power: f32,
    pub over_voltage: bool,
    pub over_current: bool,
    pub over_temperature: bool,
    pub mode: Mode,
}

impl Operation for MeasureAllInfo {
    type Out = MeasureAllInfoOutput;

    fn command(&self) -> Cow<'_, str> {
        Cow::Borrowed("MEAS:ALL:INFO?")
    }

    fn parse_line(&self, line: &str) -> Result<Self::Out, Error> {
        let mut parts = line.split(',');
        Ok(MeasureAllInfoOutput {
            volt: pop_f32(&mut parts)?,
            current: pop_f32(&mut parts)?,
            power: pop_f32(&mut parts)?,
            over_voltage: pop_bool(&mut parts)?,
            over_current: pop_bool(&mut parts)?,
            over_temperature: pop_bool(&mut parts)?,
            mode: parts
                .next()
                .and_then(|mode| {
                    let m = mode.parse().ok()?;
                    match m {
                        0 => Some(Mode::Standby),
                        1 => Some(Mode::ConstantVoltage),
                        2 => Some(Mode::ConstantCurrent),
                        3 => Some(Mode::Failed),
                        _ => None,
                    }
                })
                .ok_or(Error::UnexpectedData)?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::operations::test_states;

    #[test]
    fn test_measure_all() {
        test_states(
            MeasureAll {},
            &[(
                "1.000,2.000",
                Ok(MeasureAllOutput {
                    volt: 1.0,
                    current: 2.0,
                }),
            )],
        );
    }

    #[test]
    fn test_measure_all_info() {
        test_states(
            MeasureAllInfo {},
            &[(
                "4.987,0.514,2.560,OFF,OFF,OFF,1",
                Ok(MeasureAllInfoOutput {
                    volt: 4.987,
                    current: 0.514,
                    power: 2.560,
                    over_voltage: false,
                    over_current: false,
                    over_temperature: false,
                    mode: Mode::ConstantVoltage,
                }),
            )],
        );
    }
}
