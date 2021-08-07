## M27 KVM switch control

A KVM control library for the GIGABYTE M27Q monitor.

### Rationale

It's possible to switch to all inputs _except_ the USB-C input via [ddcutil](https://www.ddcutil.com/). This tool uses the usb billboard device to control the KVM switch and source functionality.

### Installation

- Install udev rules from `udev/`.
- Install this package via cargo: `cargo install m27q-kvm --force`. (`--force` to always get the latest version)

### Platform support

- Linux - Tested and working
- MacOS - Tested and working
- Windows - Untested