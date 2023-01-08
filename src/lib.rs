#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(feature = "async")]
use futures::{AsyncBufReadExt, AsyncWriteExt};
use log::debug;
use std::{
    borrow::Cow,
    io::{BufRead, BufReader},
};
use thiserror::Error;

pub mod operations;
use operations::iee4882::IdnOutput;
use operations::measure::{MeasureAllInfoOutput, MeasureAllOutput};

macro_rules! query_method {
    (sync, $method: ident, $op:expr, $out:ident) => {
        pub fn $method(&mut self) -> Result<$out, Error> {
            self.run_operation($op)
        }
    };
    (async, $method: ident, $op:expr, $out:ident) => {
        pub async fn $method(&mut self) -> Result<$out, Error> {
            self.run_operation($op).await
        }
    };
}

macro_rules! set_method {
    (sync, $method: ident, $param: ident, $set:literal, $ty:ident) => {
        pub fn $method(&mut self, $param: $ty) -> Result<(), Error> {
            self.write_command(&format!($set, $param))
        }
    };

    (async, $method: ident, $param: ident, $set:literal, $ty:ident) => {
        pub async fn $method(&mut self, $param: $ty) -> Result<(), Error> {
            self.write_command(&format!($set, $param)).await
        }
    };
}

macro_rules! write_method {
    (sync, $method: ident, $set:literal) => {
        pub fn $method(&mut self) -> Result<(), Error> {
            self.write_command($set)
        }
    };

    (async, $method: ident, $set:literal) => {
        pub async fn $method(&mut self) -> Result<(), Error> {
            self.write_command($set).await
        }
    };
}

macro_rules! implement_spe {
    ($sync_or_async:ident) => {
        query_method!($sync_or_async, idn, operations::iee4882::Idn {}, IdnOutput);
        write_method!($sync_or_async, reset, "*RST");

        write_method!($sync_or_async, enable_output, "OUTP ON");
        write_method!($sync_or_async, disable_output, "OUTP OFF");
        query_method!($sync_or_async, output, operations::output::Output {}, bool);

        set_method!($sync_or_async, set_volt, volt, "VOLT {}", f32);
        query_method!($sync_or_async, volt, operations::output::Volt {}, f32);

        set_method!($sync_or_async, set_volt_limit, volt, "VOLT:LIMIT {}", f32);
        query_method!(
            $sync_or_async,
            volt_limit,
            operations::output::VoltLimit {},
            f32
        );

        set_method!($sync_or_async, set_current, current, "CURR {}", f32);
        query_method!($sync_or_async, current, operations::output::Current {}, f32);

        set_method!(
            $sync_or_async,
            set_current_limit,
            current,
            "CURR:LIMIT {}",
            f32
        );
        query_method!(
            $sync_or_async,
            current_limit,
            operations::output::CurrentLimit {},
            f32
        );

        query_method!(
            $sync_or_async,
            measure_volt,
            operations::measure::Volt {},
            f32
        );
        query_method!(
            $sync_or_async,
            measure_current,
            operations::measure::Current {},
            f32
        );
        query_method!(
            $sync_or_async,
            measure_power,
            operations::measure::Power {},
            f32
        );

        query_method!(
            $sync_or_async,
            measure_all,
            operations::measure::MeasureAll {},
            MeasureAllOutput
        );

        query_method!(
            $sync_or_async,
            measure_all_info,
            operations::measure::MeasureAllInfo {},
            MeasureAllInfoOutput
        );
    };
}

#[derive(Debug)]
pub struct SPE<T>
where
    T: std::io::Write + std::io::Read,
{
    transport: BufReader<T>,
}

impl<T> SPE<T>
where
    T: std::io::Read + std::io::Write,
{
    pub fn new(transport: T) -> Self {
        Self {
            transport: BufReader::new(transport),
        }
    }

    fn write_command(&mut self, cmd: &str) -> Result<(), Error> {
        debug!("out: {}", cmd);
        let writer = self.transport.get_mut();
        writer.write_all(cmd.as_bytes())?;
        writer.write_all(b"\r\n")?;
        writer.flush()?;
        Ok(())
    }

    fn run_operation<Op>(&mut self, operation: Op) -> Result<Op::Out, Error>
    where
        Op: Operation,
    {
        self.write_command(&operation.command())?;

        let mut out: String = String::new();
        self.transport.read_line(&mut out)?;
        let trimmed = out.trim_end();

        debug!("in: {}", trimmed);
        operation.parse_line(trimmed)
    }

    implement_spe!(sync);
}

#[cfg(feature = "serialport")]
#[cfg_attr(docsrs, doc(cfg(feature = "serialport")))]
mod s {
    use super::*;

    #[cfg(unix)]
    type SerialportNative = serialport::TTYPort;
    #[cfg(windows)]
    type SerialportNative = serialport::COMPort;

    impl SPE<SerialportNative> {
        pub fn from_serialport(path: &str) -> Result<Self, serialport::Error> {
            debug!("Opening: {}", path);
            let port = serialport::new(path, 115_200)
                .timeout(std::time::Duration::from_secs(5))
                .open_native()?;
            Ok(Self::new(port))
        }
    }
}

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
mod a {
    use super::*;
    #[derive(Debug)]
    #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
    pub struct AsyncSPE<T>
    where
        T: futures::io::AsyncWrite + futures::io::AsyncRead + Unpin,
    {
        transport: futures::io::BufReader<T>,
    }

    impl<T> AsyncSPE<T>
    where
        T: futures::io::AsyncWrite + futures::io::AsyncRead + Unpin,
    {
        pub fn new(transport: T) -> Self {
            Self {
                transport: futures::io::BufReader::new(transport),
            }
        }

        async fn write_command(&mut self, cmd: &str) -> Result<(), Error> {
            debug!("out: {}", cmd);
            let writer = self.transport.get_mut();
            writer.write_all(cmd.as_bytes()).await?;
            writer.write_all(b"\r\n").await?;
            writer.flush().await?;
            Ok(())
        }

        async fn run_operation<Op>(&mut self, operation: Op) -> Result<Op::Out, Error>
        where
            Op: Operation,
        {
            self.write_command(&operation.command()).await?;

            let mut out: String = String::new();
            self.transport.read_line(&mut out).await?;
            let trimmed = out.trim_end();

            debug!("in: {}", trimmed);
            operation.parse_line(trimmed)
        }

        implement_spe!(async);
    }
}

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub use a::AsyncSPE;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unexpected data")]
    UnexpectedData,
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
}

trait Operation {
    type Out: PartialEq + std::fmt::Debug;

    fn command(&self) -> Cow<'_, str>;
    fn has_output(&self) -> bool {
        true
    }
    fn parse_line(&self, line: &str) -> Result<Self::Out, Error>;
}
