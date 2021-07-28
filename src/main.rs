use std::time::Duration;

use rusb::{
    Context, Device, DeviceDescriptor, DeviceHandle, UsbContext,
};

pub const M27Q_VID: u16 = 0x2109;
pub const M27Q_PID: u16 = 0x8883;

fn main() {
    match Context::new() {
        Ok(mut context) => match open_device(&mut context, M27Q_VID, M27Q_PID) {
            Some((_device, _device_desc, handle)) => {
                println!("Succesfully opened m27q connection!");
                println!("Triggering KVM switch...");
                handle.write_control(0x40, 178, 0, 0, &[0x6e, 0x51, 0x84, 0x03, 0xe0, 0x69, 0x01], Duration::from_secs(3)).unwrap();
                println!("Success!");
            }
            None => println!("could not find m27q"),
        },
        Err(e) => panic!("could not initialize libusb: {}", e),
    }
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
