use std::io::{Read, Write};

use clap::{Parser, Subcommand};
use owon_spe::SPE;

#[derive(Debug, Subcommand)]
enum OutputCommand {
    On,
    Off,
    Volt { volt: f32 },
    VoltLimit { volt: f32 },
    Current { current: f32 },
    CurrentLimit { current: f32 },
}

#[derive(Debug, Subcommand)]
enum MeasureCommand {
    Volt,
    Current,
    Power,
    All,
    AllInfo,
}

#[derive(Debug, Subcommand)]
enum Command {
    Idn,
    Reset,
    Output {
        #[command(subcommand)]
        command: Option<OutputCommand>,
    },
    Measure {
        #[command(subcommand)]
        command: Option<MeasureCommand>,
    },
}

#[derive(clap::Parser)]
struct Opts {
    port: String,
    #[command(subcommand)]
    command: Command,
}

fn measure<T: Read + Write>(mut spe: SPE<T>, cmd: Option<MeasureCommand>) -> anyhow::Result<()> {
    match cmd {
        Some(MeasureCommand::Volt) => println!("Power: {}", spe.measure_volt()?),
        Some(MeasureCommand::Current) => println!("Power: {}", spe.measure_current()?),
        Some(MeasureCommand::Power) => println!("Power: {}", spe.measure_power()?),
        Some(MeasureCommand::All) => {
            let o = spe.measure_all()?;
            println!("Volt: {}", o.volt);
            println!("Current: {}", o.current);
        }
        Some(MeasureCommand::AllInfo) | None => {
            let o = spe.measure_all_info()?;
            println!("Volt: {}", o.volt);
            println!("Current: {}", o.current);
            println!("Power: {}", o.power);
            println!("Overvoltage: {}", o.over_voltage);
            println!("Overcurrent: {}", o.over_current);
            println!("OverTemperature: {}", o.over_temperature);
            println!("Mode: {}", o.mode);
        }
    }
    Ok(())
}

fn output<T: Read + Write>(mut spe: SPE<T>, cmd: Option<OutputCommand>) -> anyhow::Result<()> {
    if let Some(cmd) = cmd {
        match cmd {
            OutputCommand::On => spe.enable_output()?,
            OutputCommand::Off => spe.disable_output()?,
            OutputCommand::Volt { volt } => spe.set_volt(volt)?,
            OutputCommand::VoltLimit { volt } => spe.set_volt_limit(volt)?,
            OutputCommand::Current { current } => spe.set_current(current)?,
            OutputCommand::CurrentLimit { current } => spe.set_current_limit(current)?,
        }
    } else {
        println!("Output: {}", spe.output()?);
        println!("Voltage: {}", spe.volt()?);
        println!("Voltage Limit: {}", spe.volt_limit()?);
        println!("Current: {}", spe.current()?);
        println!("Current Limit: {}", spe.current_limit()?);
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let opts = Opts::parse();

    let mut spe = SPE::from_serialport(&opts.port)?;
    match opts.command {
        Command::Idn => {
            let idn = spe.idn()?;
            println!(
                "Model: {}, Serial: {}, Firmware: {}",
                idn.model, idn.serial, idn.fw
            )
        }
        Command::Reset => spe.reset()?,
        Command::Output { command } => output(spe, command)?,
        Command::Measure { command } => measure(spe, command)?,
    }

    Ok(())
}
