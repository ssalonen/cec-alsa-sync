use libcec_sys::bindings::{
    cec_command, cec_datapacket, cec_device_type, cec_device_type_list, cec_keypress,
    cec_logical_address, cec_opcode, cec_user_control_code, libcec_clear_configuration,
    libcec_close, libcec_configuration, libcec_connection_t, libcec_destroy,
    libcec_enable_callbacks, libcec_initialise, libcec_open, libcec_transmit, libcec_version,
    ICECCallbacks,
};
use std::process::Command;

use libcec_sys::CecConnection;
use std::{thread, time};

#[repr(C)]
struct CallbackHandle {
    // have conn here? for use in callbacks?
}

extern "C" fn onKeyPress(
    callbackHandle: *mut ::std::os::raw::c_void, /*CallbackHandle,*/
    keypress_raw: *const cec_keypress,
) {
    let keypress: cec_keypress;
    unsafe {
        if keypress_raw as usize == 0 {
            return;
        }
        keypress = *keypress_raw;
    }
    println!(
        "onKeyPress: {:?}, keycode: {:?}, duration: {:?}",
        keypress_raw, keypress.keycode, keypress.duration
    );
    let vol_delta: Option<&str> = match keypress.keycode {
        cec_user_control_code::VOLUME_UP => Some("1%+"),
        cec_user_control_code::VOLUME_DOWN => Some("1%-"),
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

extern "C" fn onCommandReceived(
    callbackHandle: *mut ::std::os::raw::c_void, /*CallbackHandle,*/
    command_raw: *const cec_command,
) {
    let command: cec_command;
    unsafe {
        if command_raw as usize == 0 {
            return;
        }
        command = *command_raw;
    }

    println!(
        "onCommandReceived: {:?}, opcode: {:?}, initiator: {:?}",
        command_raw, command.opcode, command.initiator
    );
    //=> callback kuuntele GIVE_AUDIO_STATUS, transmit REPORT_AUDIO_STATUS
}

static mut CALLBACKS: ICECCallbacks = ICECCallbacks {
    logMessage: Option::None,
    keyPress: Option::Some(onKeyPress),
    commandReceived: Option::Some(onCommandReceived),
    configurationChanged: Option::None,
    alert: Option::None,
    menuStateChanged: Option::None,
    sourceActivated: Option::None,
};

pub fn main() {
    let types = vec![cec_device_type::AUDIO_SYSTEM];

    // TODO: Pin<callbacks> or have it static?
    let config: libcec_configuration;
    unsafe {
        config = libcec_configuration::new(false, types.into(), &mut CALLBACKS);
    }

    let connection = CecConnection::new(config).expect("CecConnection failed");
    connection
        .open(&std::ffi::CString::new("RPI").unwrap(), 1000)
        .expect("CecConnection failed");    
    thread::sleep(time::Duration::from_secs(99_999_999));
}
