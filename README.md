# owon-spe support crate

Helper crate for the SCPI protocol as spoken by the Owon SPE power supplies;
Tested on an Owon SPE 3102

```rust
use owon_spe::SPE;

fn main() {
  let mut spe = SPE::from_serialport("/dev/ttyUSB0").unwrap();
  let info = spe.idn().unwrap();
  println!("Model: {}, Serial: {}, FW: {}", info.model, info.serial, info.fw);
}
```
