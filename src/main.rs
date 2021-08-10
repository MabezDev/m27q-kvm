//! M27 KVM control
//!
//! A simple tool to manage the GIGABYTE M27Q KVM interface
//! Switching to USB-C input is not supported via DDC, but it is exposed via the monitors usb billboard device.
//!
//! All USB writes have been reverse engineered from the OSD side kick tool by analyzing the USB packets over wireshark.

use std::{fmt::Display, str::FromStr, thread::sleep, time::Duration};

use rusb::{Context, Device, DeviceDescriptor, DeviceHandle, UsbContext};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tray_item::TrayItem;

pub const M27Q_VID: u16 = 0x2109;
pub const M27Q_PID: u16 = 0x8883;

const MENU_SEPARATOR: &str = "------------";

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter)]
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
            _ => return Err("Invalid KVM input - valid inputs are HDMI1,HDMI2,DP"),
        })
    }
}

struct Args {
    switch_back_input: Option<KvmInput>,
    run_trigger: bool,
}

fn main() {
    let mut pargs = pico_args::Arguments::from_env();

    let tray: bool = pargs.contains("--tray");
    let run_trigger = pargs.contains("--kvm-trigger");

    let args = Args {
        switch_back_input: pargs.opt_value_from_str("--switch-back-input").unwrap(),
        run_trigger,
    };

    if tray {
        launch_tray();
    } else {
        let cmds = cmd_queue_from_args(&args);
        exec(cmds).unwrap();
    }
}

pub struct Command {
    rtype: u8,
    request: u8,
    value: u16,
    index: u16,
    buf: Vec<u8>,
}

fn cmd_queue_from_args(args: &Args) -> Vec<Command> {
    let mut cmds = Vec::new();
    
    if let Some(input) = args.switch_back_input {
        println!(
            "Input switch supplied, writing {} to to kvm switch back",
            input
        );
        cmds.push(Command {
            rtype: 0x40,
            request: 178,
            value: 0,
            index: 0,
            buf: vec![0x6e, 0x51, 0x84, 0x03, 0xe0, 0x6b, input as u8],
        })
    }
    if args.run_trigger {
        println!("Triggering KVM switch...");
        cmds.push(Command {
            rtype: 0x40,
            request: 178,
            value: 0,
            index: 0,
            buf: vec![0x6e, 0x51, 0x84, 0x03, 0xe0, 0x69, 0x01],
        });
    }

    cmds
}

fn exec(cmds: Vec<Command>) -> rusb::Result<()> {
    match Context::new() {
        Ok(mut context) => match open_device(&mut context, M27Q_VID, M27Q_PID) {
            Some((_device, _device_desc, handle)) => {
                println!("Succesfully opened m27q connection!");

                for cmd in cmds {
                    handle.write_control(
                        cmd.rtype,
                        cmd.request,
                        cmd.value,
                        cmd.index,
                        &cmd.buf,
                        Duration::from_secs(1),
                    )?;
                    sleep(Duration::from_millis(50));
                }

                println!("Success!");
            }
            None => println!("could not find m27q"),
        },
        Err(e) => panic!("could not initialize libusb: {}", e),
    }

    Ok(())
}

fn launch_tray() {
    let mut tray = TrayItem::new("M27Q", "").unwrap();

    tray.add_menu_item("KVM Switch", move || {
        let cmds = cmd_queue_from_args(&Args {
            run_trigger: true,
            switch_back_input: None,
        });
        exec(cmds).unwrap();
    })
    .unwrap();

    tray.add_label(MENU_SEPARATOR).unwrap();

    for input in KvmInput::iter() {
        tray.add_menu_item(&format!("{}", input), move || {
            let cmds = cmd_queue_from_args(&Args {
                run_trigger: true,
                switch_back_input: Some(input),
            });
            exec(cmds).unwrap();
        })
        .unwrap();
    }

    tray.add_label(MENU_SEPARATOR).unwrap();

    let inner = tray.inner_mut();
    inner.add_quit_item("Quit");
    inner.display();
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
                }
            }
        }
    }

    None
}
