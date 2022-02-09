use log::{debug, error, info, trace, warn};

extern crate cec_rs;
use cec_rs::{
    CecCommand, CecConnection, CecConnectionCfgBuilder, CecDeviceType, CecDeviceTypeVec,
    CecKeypress, CecLogMessage, CecUserControlCode,
};
use std::process::Command;
use std::sync::mpsc::{channel, Sender};

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

fn on_command_received(sender: Sender<CecCommand>, command: CecCommand) {
    trace!(
        "onCommandReceived:  opcode: {:?}, initiator: {:?}",
        command.opcode,
        command.initiator
    );
    match command.opcode {
        cec_rs::CecOpcode::GiveSystemAudioModeStatus => {
            // report audio mode
        }
        cec_rs::CecOpcode::GiveAudioStatus => {
            // report volume
        }
        _ => {}
    };
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
    let (sender, receiver) = channel();

    let cfg = CecConnectionCfgBuilder::default()
        .port("RPI".into())
        .device_name("Hifiberry".into())
        .key_press_callback(Box::new(on_key_press))
        .command_received_callback(Box::new(move |command| {
            on_command_received(sender, command) // TODO: Command to implement copy?
        }))
        .log_message_callback(Box::new(on_log_message))
        .device_types(CecDeviceTypeVec::new(CecDeviceType::AudioSystem))
        .build()
        .unwrap();
    let connection: CecConnection = cfg.open().unwrap();

    trace!("Active source: {:?}", connection.get_active_source());

    loop {
        if let Ok(command) = receiver.recv() {
            if let Err(e) = connection.transmit(command.into()) {
                // FIXME: fix cec-rs signature
                // warn!("Could not send command {}: {}", command.opcode, e) // FIXME:display
            }
        } else {
            break;
        }
    }

    // TODO: handle alsa vol changes
}
