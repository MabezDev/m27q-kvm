//! M27 KVM control
//! 
//! A simple tool to manage the GIGABYTE M27Q KVM interface
//! Switching to USB-C input is not supported via DDC, but it is exposed via the monitors usb billboard device.
//! 
//! All USB writes have been reverse engineered from the OSD side kick tool by analyzing the USB packets over wireshark. 



use std::{fmt::Display, str::FromStr, thread::sleep, time::Duration};

use rusb::{
    Context, Device, DeviceDescriptor, DeviceHandle, UsbContext,
};

pub const M27Q_VID: u16 = 0x2109;
pub const M27Q_PID: u16 = 0x8883;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KvmInput {
    Hdmi1 = 0x00,
    Hdmi2 = 0x01,
    DisplayPort = 0x02,
}

impl Display for KvmInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KvmInput::Hdmi1 => write!(f, "HDMI1"),
            KvmInput::Hdmi2 => write!(f, "HDMI2"),
            KvmInput::DisplayPort => write!(f, "DP"),
        }
    }
}

impl FromStr for KvmInput {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "HDMI1" => KvmInput::Hdmi1,
            "HDMI2" => KvmInput::Hdmi2,
            "DP" => KvmInput::DisplayPort,
            _ => return Err("Invalid KVM input - valid inputs are HDMI1,HDMI2,DP")
        })
    }
}

struct Args {
    switch_back_input: Option<KvmInput>,
    tray: bool,
    run_trigger: bool
}

fn main() {
    let mut pargs = pico_args::Arguments::from_env();

    let tray: bool = pargs.value_from_str("--tray").unwrap();
    let run_trigger = !tray && pargs.value_from_str("--kvm-trigger").expect("Nothing to do!");
    
    let args = Args {
        switch_back_input: pargs.opt_value_from_str("--switch-back-input").unwrap(),
        tray,
        run_trigger,
    };

    match Context::new() {
        Ok(mut context) => match open_device(&mut context, M27Q_VID, M27Q_PID) {
            Some((_device, _device_desc, handle)) => {
                println!("Succesfully opened m27q connection!");
                if let Some(input) = args.switch_back_input {
                    println!("Input switch supplied, writing {} to to kvm switch back", input);
                    handle.write_control(0x40, 178, 0, 0, &[0x6e, 0x51, 0x84, 0x03, 0xe0, 0x6b, input as u8], Duration::from_secs(3)).unwrap();
                    sleep(Duration::from_millis(50))
                }
                if args.tray {
                    launch_tray(&args);
                }
                if args.run_trigger {
                    println!("Triggering KVM switch...");
                    handle.write_control(0x40, 178, 0, 0, &[0x6e, 0x51, 0x84, 0x03, 0xe0, 0x69, 0x01], Duration::from_secs(3)).unwrap();
                }

                println!("Success!");
            }
            None => println!("could not find m27q"),
        },
        Err(e) => panic!("could not initialize libusb: {}", e),
    }
}

fn launch_tray(args: &Args) {

}

fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<(Device<T>, DeviceDescriptor, DeviceHandle<T>)> {
    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None,
    };

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };
        println!("Found device: {:?}", device);
        
        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            match device.open() {
                Ok(handle) => return Some((device, device_desc, handle)),
                Err(e) => {
                    println!("Failed to open device: {:?}", e);   
                    return None;
                },
            }
        }
    }

    None
}
