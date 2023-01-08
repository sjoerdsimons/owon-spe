use clap::Parser;
use owon_spe::AsyncSPE;
use tokio_serial::SerialPortBuilderExt;

#[derive(clap::Parser)]
struct Opts {
    port: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let opts = Opts::parse();

    let port = tokio_serial::new(&opts.port, 115200).open_native_async()?;
    let port = async_compat::Compat::new(port);
    let mut spe = AsyncSPE::new(port);

    loop {
        let a = spe.measure_all_info().await?;
        println!(
            "V: {} A: {} P: {}, OV: {}, OC: {}, OT: {}, M: {}",
            a.volt, a.current, a.power, a.over_voltage, a.over_current, a.over_temperature, a.mode
        );
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
    }
}
