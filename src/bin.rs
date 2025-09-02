use log::{debug, error, info, trace, warn};

extern crate cec_rs;
use arrayvec::ArrayVec;
use cec_rs::{
    CecCommand, CecConnection, CecConnectionCfgBuilder, CecDatapacket, CecDeviceType,
    CecDeviceTypeVec, CecKeypress, CecLogMessage, CecPowerStatus, CecUserControlCode,
};

use std::convert::TryFrom;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::{channel, Sender};

use std::sync::{Arc, Mutex};
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
    let app_config = CONFIG.get().expect("Config not available");
    let command: Option<Command> = match keypress.keycode {
        CecUserControlCode::VolumeUp => Some(app_config.vol_up_command.new_command()),
        CecUserControlCode::VolumeDown => Some(app_config.vol_down_command.new_command()),
        CecUserControlCode::Mute => {
            if keypress.duration.is_zero() {
                // Filter duplicate events
                None
            } else {
                app_config.mute_command.as_ref().map(|c| c.new_command())
            }
        }
        _ => None,
    };
    if let Some(mut command) = command {
        debug!(
            "Executing command {:?} {:?}",
            command.get_program(),
            command.get_args()
        );
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
            let user_control_code = CecUserControlCode::from_repr(command.parameters.0[0] as u32);
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
        cec_rs::CecOpcode::ReportPowerStatus => {
            if !command.parameters.0.is_empty() {
                let power_status = CecPowerStatus::from_repr(command.parameters.0[0] as _);
                if let Some(power_status) = power_status {
                    if matches!(command.initiator, cec_rs::CecLogicalAddress::Tv) {
                        on_tv_power_status_changed(power_status);
                    }
                }
            }
        }
        _ => {}
    };
}

fn on_tv_power_status_changed(power_status: CecPowerStatus) {
    info!("TV power status changed: {:?}", power_status);

    let app_config = CONFIG.get().expect("Config not available");

    match power_status {
        CecPowerStatus::On => {
            debug!("TV is ON");
        }
        CecPowerStatus::Standby => {
            debug!("TV is standby");
        }
        CecPowerStatus::InTransitionStandbyToOn => {
            info!("TV turned ON - calling tv_turned_on_command");
            if let Some(cmd) = &app_config.tv_turned_on_command {
                let mut command = cmd.new_command();
                debug!(
                    "Executing tv_turned_on_command command: {:?} {:?}",
                    command.get_program(),
                    command.get_args()
                );
                if let Err(e) = command.output() {
                    error!("Failed to execute tv_turned_on_command: {:?}", e);
                }
            }
        }
        CecPowerStatus::InTransitionOnToStandby => {
            info!("TV turned OFF - calling tv_turned_off_command");
            if let Some(cmd) = &app_config.tv_turned_off_command {
                let mut command = cmd.new_command();
                debug!(
                    "Executing tv_turned_off_command command: {:?} {:?}",
                    command.get_program(),
                    command.get_args()
                );
                if let Err(e) = command.output() {
                    error!("Failed to execute tv_turned_off_command: {:?}", e);
                }
            }
        }
        CecPowerStatus::Unknown => {
            warn!("Unknown TV power status received");
        }
    }
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

    let mut connection_config_builder = CecConnectionCfgBuilder::default()
        .device_name(app_config.device_name.clone())
        .key_press_callback(Box::new(on_key_press))
        .command_received_callback(Box::new(move |command| {
            on_command_received(sender.clone(), command)
        }))
        .log_message_callback(Box::new(on_log_message))
        .device_types(CecDeviceTypeVec::new(CecDeviceType::AudioSystem));
    if !app_config.hdmi_port.clone().is_empty() {
        connection_config_builder = connection_config_builder.port(app_config.hdmi_port.clone());
    }
    let connection_config = connection_config_builder
        .build()
        .expect("Could not construct config");

    let connection: Arc<Mutex<CecConnection>> =
        Arc::new(Mutex::new(connection_config.open().unwrap_or_else(|_| {
            panic!(
                "Adapter open failed, port {:?}",
                app_config.hdmi_port.clone()
            )
        })));

    // Start power status polling thread
    if app_config.tv_turned_on_command.is_some() || app_config.tv_turned_off_command.is_some() {
        let connection_clone = connection.clone();
        let poll_interval = Duration::from_millis(app_config.power_poll_interval_ms);

        std::thread::spawn(move || {
            loop {
                std::thread::sleep(poll_interval);

                // Request power status from TV (logical address 0)
                let power_request = CecCommand {
                    ack: true,
                    destination: cec_rs::CecLogicalAddress::Tv,
                    eom: true,
                    initiator: cec_rs::CecLogicalAddress::Audiosystem, // Our address
                    transmit_timeout: Duration::from_millis(1000),
                    parameters: CecDatapacket(ArrayVec::new()), // No parameters
                    opcode_set: true,
                    opcode: cec_rs::CecOpcode::GiveDevicePowerStatus,
                };
                if let Err(e) = connection_clone.lock().unwrap().transmit(power_request) {
                    debug!("Failed to request TV power status: {:?}", e);
                }
            }
        });
    }
    loop {
        if let Ok(command) = receiver.recv() {
            match connection.lock().unwrap().transmit(command.clone()) {
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
