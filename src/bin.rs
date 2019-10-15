extern crate cec_rs;
use cec_rs::{
    CecCommand, CecConfiguration, CecConnection, CecDeviceType, CecDeviceTypeVec, CecKeypress,
    CecUserControlCode,
};
use std::process::Command;

use std::{thread, time};

fn on_key_press(keypress: CecKeypress) {
    println!(
        "onKeyPress: {:?}, keycode: {:?}, duration: {:?}",
        keypress, keypress.keycode, keypress.duration
    );
    let vol_delta: Option<&str> = match keypress.keycode {
        CecUserControlCode::VolumeUp => Some("1%+"),
        CecUserControlCode::VolumeDown => Some("1%-"),
        _ => None,
    };
    if let Some(vol_delta) = vol_delta {
        Command::new("amixer")
            .arg("set")
            .arg("DSPVolume")
            .arg(vol_delta)
            .output()
            .expect("Failed to call amixer");
    }
    // TODO: handle vol down, vol up, mute
    // & report new audio status
}

fn on_command_received(command: CecCommand) {
    println!(
        "onCommandReceived:  opcode: {:?}, initiator: {:?}",
        command.opcode, command.initiator
    );
    //=> callback kuuntele GIVE_AUDIO_STATUS, transmit REPORT_AUDIO_STATUS
}

pub fn main() {
    // TODO: Pin<callbacks> or have it static?
    let devices = CecDeviceTypeVec::new(CecDeviceType::AudioSystem);
    let config = CecConfiguration::new("Hifiberry", devices);
    let connection = CecConnection::new(config).expect("CecConnection failed");
    connection
        .open(
            "RPI",
            1000,
            Some(Box::new(on_key_press)),
            Some(Box::new(on_command_received)),
        )
        .expect("CecConnection failed");
    thread::sleep(time::Duration::from_secs(99_999_999));
}
