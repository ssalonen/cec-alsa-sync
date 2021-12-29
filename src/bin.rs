use log::{debug, error, info, trace, warn};

extern crate cec_rs;
use cec_rs::{
    CecCommand, CecConnectionCfgBuilder, CecDeviceType, CecDeviceTypeVec, CecKeypress,
    CecLogMessage, CecUserControlCode,
};
use std::process::Command;

use std::{thread, time};

fn on_key_press(keypress: CecKeypress) {
    trace!(
        "onKeyPress: {:?}, keycode: {:?}, duration: {:?}",
        keypress,
        keypress.keycode,
        keypress.duration
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
}

fn on_command_received(command: CecCommand) {
    trace!(
        "onCommandReceived:  opcode: {:?}, initiator: {:?}",
        command.opcode,
        command.initiator
    );
}

fn on_log_message(log_message: CecLogMessage) {
    match log_message.level {
        cec_rs::CecLogLevel::All => trace!("cec log: {:?}", log_message.message),
        cec_rs::CecLogLevel::Debug | cec_rs::CecLogLevel::Traffic => {
            debug!("cec log: {:?}", log_message.message)
        }
        cec_rs::CecLogLevel::Notice => info!("cec log: {:?}", log_message.message),
        cec_rs::CecLogLevel::Warning => warn!("cec log: {:?}", log_message.message),
        cec_rs::CecLogLevel::Error => error!("cec log: {:?}", log_message.message),
    }
}

pub fn main() {
    env_logger::init();

    let cfg = CecConnectionCfgBuilder::default()
        .port("RPI".into())
        .device_name("Hifiberry".into())
        .key_press_callback(Box::new(on_key_press))
        .command_received_callback(Box::new(on_command_received))
        .log_message_callback(Box::new(on_log_message))
        .device_types(CecDeviceTypeVec::new(CecDeviceType::AudioSystem))
        .build()
        .unwrap();
    let connection = cfg.open().unwrap();
    trace!("Active source: {:?}", connection.get_active_source());

    thread::sleep(time::Duration::from_secs(99_999_999));
    // TODO: handle alsa vol changes
}
