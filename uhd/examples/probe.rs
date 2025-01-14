extern crate uhd;

use std::error::Error;

use num_complex::Complex32;
use uhd::{Usrp, TuneRequest, ReceiveStreamer, StreamArgs};

fn main() -> Result<(), Box<dyn Error>> {
    let found_usrps = Usrp::find("")?;

    for address in found_usrps {
        println!("Opening {}", address);
        match probe_one_usrp(&address) {
            Ok(_) => {}
            Err(e) => eprintln!("{}", e),
        }
    }

    Ok(())
}

fn probe_one_usrp(address: &str) -> Result<(), Box<dyn Error>> {
    let usrp = Usrp::open(address)?;
    let num_mboards = usrp.get_num_motherboards()?;
    for board in 0..num_mboards {
        println!(
            "Motherboard {}, name: {}",
            board,
            usrp.get_motherboard_name(board)?
        );
        if let Ok(rate) = usrp.get_master_clock_rate(board) {
            println!("Master clock rate {}", rate);
        }
        if let Ok(eeprom) = usrp.get_motherboard_eeprom(board) {
            let keys = [
                "hardware",
                "revision",
                "revision_compat",
                "product",
                "serial",
                "name",
                "mac-addr",
                "mac-addr0",
                "mac-addr1",
                "ip-addr",
                "ip-addr0",
                "ip-addr1",
                "ip-addr2",
                "ip-addr3",
                "subnet",
                "subnet0",
                "subnet1",
                "subnet2",
                "subnet3",
                "gateway",
            ];
            for key in &keys {
                if let Ok(Some(value)) = eeprom.get(key) {
                    println!("Motherboard EEPROM[{}] = {}", key, value);
                }
            }
        }
        if let Ok(eeprom) = usrp.get_daughter_board_eeprom("rx", "A", board) {
            println!("Daughter RX {:?}", eeprom);
        }
        if let Ok(eeprom) = usrp.get_daughter_board_eeprom("tx", "A", board) {
            println!("Daughter TX {:?}", eeprom);
        }
        if let Ok(eeprom) = usrp.get_daughter_board_eeprom("gdb", "A", board) {
            println!("Daughter GDB {:?}", eeprom);
        }
        if let Ok(banks) = usrp.get_gpio_banks(board) {
            println!("GPIO banks {:?}", banks);
        }
    }

    for channel in 0..usrp.get_num_tx_channels()? {
        println!("Transmit channel {}:", channel);
        if let Ok(antennas) = usrp.get_tx_antennas(channel) {
            println!("TX antennas {:?}", antennas);
        }
        if let Ok(range) = usrp.get_fe_tx_freq_range(channel) {
            println!("Front-end TX frequency ranges: {:?}", range);
        }
        if let Ok(gain) = usrp.get_normalized_tx_gain(channel) {
            println!("Normalized TX gain {}", gain);
        }
    }
    for channel in 0..usrp.get_num_rx_channels()? {
        println!("Receive channel {}:", channel);
        if let Ok(antennas) = usrp.get_rx_antennas(channel) {
            println!("RX antennas {:?}", antennas);
        }
        if let Ok(range) = usrp.get_fe_rx_freq_range(channel) {
            println!("Front-end RX frequency ranges: {:?}", range);
        }
        if let Ok(gain) = usrp.get_normalized_rx_gain(channel) {
            println!("Normalized RX gain {}", gain);
        }
        if let Ok(names) = usrp.get_rx_gain_names(channel) {
            for name in names {
                let range = usrp.get_rx_gain_range(channel, &name)?;
                let current = usrp.get_rx_gain(channel, &name)?;
                println!(
                    "Gain element {}: range {:?}, current {}",
                    name, range, current
                );
            }
        }
        println!("Local oscillators: {:?}", usrp.get_rx_lo_names(channel)?);
    }

    let channel = 0;
    println!("Antennas: {:?}", usrp.get_rx_antennas(channel).unwrap());
    usrp.set_rx_antenna("RX2", channel).unwrap();

    usrp.set_rx_frequency(&TuneRequest::with_frequency(2446500000.0), channel).unwrap();
    usrp.set_rx_sample_rate(15360000.0, channel).unwrap();
    println!("Gain name: {:?}", usrp.get_rx_gain_names(channel).unwrap());
    usrp.set_rx_gain(20.0, channel, "PGA").unwrap();

    // usrp.set_rx_antenna(antenna, channel)

    // usrp.get_rx_st
    let mut stream = usrp.get_rx_stream::<Complex32>(&StreamArgs::builder().channels(vec![channel]).build()).unwrap();
    let mut buff = [Complex32::default(); 2400 / 16];
    for i in 0..(10){
        stream.receive_simple(&mut buff).unwrap();
        println!("Recived: {:?}", &buff[..16]);
    }

    Ok(())
}
