use log::{debug, error, info, trace, warn};

extern crate cec_rs;
use arrayvec::ArrayVec;
use cec_rs::{
    CecCommand, CecConnection, CecConnectionCfgBuilder, CecDatapacket, CecDeviceType,
    CecDeviceTypeVec, CecKeypress, CecLogMessage, CecUserControlCode,
};

use std::convert::TryFrom;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::{channel, Sender};

use std::time::Duration;

mod config;
use config::CONFIG;

use crate::config::{AppConfig, CreatesCommand};

struct VolumePercent(u8);

impl TryFrom<u8> for VolumePercent {
    type Error = ();
    fn try_from(volume_percent: u8) -> Result<Self, Self::Error> {
        if volume_percent > 100 {
            Err(())
        } else {
            Ok(VolumePercent(volume_percent))
        }
    }
}

impl VolumePercent {
    fn volume_percent(&self) -> u8 {
        self.0
    }
}

fn audio_status_data_packet(mute: bool, volume: VolumePercent) -> CecDatapacket {
    let mut data = ArrayVec::<u8, 64>::new();
    // Audio Status payload is 1 byte
    // Bit 7: Audio Mute Status. 0=Mute off, 1=Mute on
    // Bit 0-6: Audio Volume Status
    //
    // Audio Volume Status:
    // 0x00 <= N <= 0x64 : Playback volume as percentage, number between 0 (0x00) and 100 (0x64)
    // 0x64 <= N <= 0x7E : Reserved
    // 0x7F : Current audio volume is unknown
    let volume_low_7bits = 0b0111_1111 & volume.volume_percent();
    data.push(if mute {
        0b1000_0000 | volume_low_7bits
    } else {
        volume_low_7bits
    });

    CecDatapacket(data)
}

fn bool_data_packet(val: bool) -> CecDatapacket {
    let mut data = ArrayVec::<u8, 64>::new();
    data.push(if val { 1 } else { 0 });
    CecDatapacket(data)
}

fn on_key_press(keypress: CecKeypress) {
    trace!(
        "onKeyPress: {:?}, keycode: {:?}, duration: {:?}",
        keypress,
        keypress.keycode,
        keypress.duration
    );
    if keypress.duration.is_zero() {
        // Filter duplicate events
        return;
    }
    let app_config = CONFIG.get().expect("Config not available");
    let command: Option<Command> = match keypress.keycode {
        CecUserControlCode::VolumeUp => Some(app_config.vol_up_command.new_command()),
        CecUserControlCode::VolumeDown => Some(app_config.vol_down_command.new_command()),
        CecUserControlCode::Mute => app_config.mute_command.as_ref().map(|c| c.new_command()),
        _ => None,
    };
    if let Some(mut command) = command {
        debug!("Executing command {:?} {:?}", command.get_program(), command.get_args());
        command.output().expect("Failed to call amixer");
    }
}

fn on_command_received(sender: Sender<CecCommand>, command: CecCommand) {
    trace!(
        "onCommandReceived:  opcode: {:?}, initiator: {:?}",
        command.opcode,
        command.initiator
    );
    match command.opcode {
        cec_rs::CecOpcode::SystemAudioModeRequest => {
            // SystemAudioModeRequest(physical address of the device that should be active system audio)

            // Note: We set system audio mode no matter what the parameter in the SystemAudioModeRequest command
            // Reply with SetSystemAudioMode
            sender
                .send(CecCommand {
                    ack: true,
                    destination: command.initiator,
                    eom: true,
                    initiator: command.destination,
                    transmit_timeout: Duration::from_millis(500),
                    parameters: bool_data_packet(true),
                    opcode_set: true,
                    opcode: cec_rs::CecOpcode::SetSystemAudioMode,
                })
                .expect("internal channel send failed");
        }
        cec_rs::CecOpcode::GiveSystemAudioModeStatus => {
            // Reply with SystemAudioModeStatus
            sender
                .send(CecCommand {
                    ack: true,
                    destination: command.initiator,
                    eom: true,
                    initiator: command.destination,
                    transmit_timeout: Duration::from_millis(500),
                    parameters: bool_data_packet(true),
                    opcode_set: true,
                    opcode: cec_rs::CecOpcode::SystemAudioModeStatus,
                })
                .expect("internal channel send failed");
        }
        cec_rs::CecOpcode::GiveAudioStatus => {
            sender
                .send(CecCommand {
                    ack: true,
                    destination: command.initiator,
                    eom: true,
                    initiator: command.destination,
                    transmit_timeout: Duration::from_millis(500),
                    parameters: audio_status_data_packet(
                        false,
                        VolumePercent::try_from(50u8).unwrap(),
                    ), // FIXME:real volume
                    opcode_set: true,
                    opcode: cec_rs::CecOpcode::ReportAudioStatus,
                })
                .expect("internal channel send failed");
        }
        cec_rs::CecOpcode::UserControlPressed => {
            let user_control_code = CecUserControlCode::try_from(command.parameters.0[0] as u32);
            if user_control_code
                .map(|cc| {
                    cc == CecUserControlCode::VolumeDown || cc == CecUserControlCode::VolumeUp
                })
                .unwrap_or(false)
            {
                // TODO: Throttle these ones with 500ms to be compliant
                sender
                    .send(CecCommand {
                        ack: true,
                        destination: command.initiator,
                        eom: true,
                        initiator: command.destination,
                        transmit_timeout: Duration::from_millis(500),
                        parameters: audio_status_data_packet(
                            false,
                            VolumePercent::try_from(50u8).unwrap(),
                        ), // FIXME:real volume
                        opcode_set: true,
                        opcode: cec_rs::CecOpcode::ReportAudioStatus,
                    })
                    .expect("internal channel send failed");
            }
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

pub fn main() -> Result<(), &'static str> {
    env_logger::init();
    read_config()?;

    let (sender, receiver) = channel();
    let app_config = CONFIG.get().expect("Config not available");

    let connection_config = CecConnectionCfgBuilder::default()
        .port(app_config.hdmi_port.clone())
        .device_name(app_config.device_name.clone())
        .key_press_callback(Box::new(on_key_press))
        .command_received_callback(Box::new(move |command| {
            on_command_received(sender.clone(), command)
        }))
        .log_message_callback(Box::new(on_log_message))
        .device_types(CecDeviceTypeVec::new(CecDeviceType::AudioSystem))
        .build()
        .unwrap();
    let connection: CecConnection = connection_config.open().unwrap();

    trace!("Active source: {:?}", connection.get_active_source());

    loop {
        if let Ok(command) = receiver.recv() {
            match connection.transmit(command.clone()) {
                Ok(_) => debug!(
                    "Sent command {:?} with parameters {:?}",
                    command.opcode, command.parameters
                ),
                Err(e) => warn!("Could not send command {:?}: {:?}", command.opcode, e),
            };
        } else {
            error!("Shutting down, no more commands");
            break;
        }
    }
    Ok(())
}

fn read_config() -> Result<(), &'static str> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        // Load default config
        let c: AppConfig = toml::from_str("").expect("error while reading config");
        CONFIG.set(c).expect("failed to set config");
        return Ok(());
    }
    let config_path = &args[1];
    let config_file_path = Path::new(config_path);
    let mut config_file = File::open(config_file_path).expect("Failed to open config");
    let mut contents = String::new();
    config_file
        .read_to_string(&mut contents)
        .expect("Failed to read config");
    let c: AppConfig = toml::from_str(&contents).expect("error while reading config");
    CONFIG.set(c).expect("failed to set config");
    Ok(())
}
