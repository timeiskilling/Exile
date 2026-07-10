use cpal::traits::HostTrait;
use exile_error::{CpalDeviceError, ExileError};

pub fn list_input_devices(host: &cpal::Host) -> Result<cpal::Device, ExileError> {
    let default_input = host.default_input_device();
    let default_name = default_input.as_ref().map(|d| d.to_string());

    println!("Available input devices:");
    for device in host.input_devices().map_err(|e| ExileError::DeviceError {
        error: CpalDeviceError::Kind { kind: e.kind() },
    })? {
        let name = device.to_string();
        let mark = if Some(&name) == default_name.as_ref() {
            "*"
        } else {
            " "
        };
        println!("{}{}", mark, name);
    }

    let device = default_input.ok_or(ExileError::DeviceError {
        error: CpalDeviceError::Kind {
            kind: cpal::ErrorKind::DeviceNotAvailable,
        },
    })?;

    Ok(device)
}
