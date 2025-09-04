use std::{ffi::CString, process::Command};

use once_cell::sync::OnceCell;
use serde::{de, Deserialize, Deserializer};

pub trait CreatesCommand {
    fn new_command(&self) -> Command;
}

#[derive(Debug)]
pub struct CommandTemplate(Vec<String>);

impl From<Vec<&str>> for CommandTemplate {
    fn from(values: Vec<&str>) -> Self {
        CommandTemplate(values.iter().cloned().map(String::from).collect())
    }
}

impl CreatesCommand for CommandTemplate {
    fn new_command(&self) -> Command {
        let args = self.0.clone();
        let mut cmd = Command::new(args.first().expect("Missing program"));
        cmd.args(&args[1..]);
        cmd
    }
}

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    #[serde(default = "default_hdmi_port")]
    pub hdmi_port: CString,
    #[serde(default = "default_device_name")]
    pub device_name: String,
    #[serde(
        default = "default_volume_up",
        deserialize_with = "command_template_from_args"
    )]
    pub vol_up_command: CommandTemplate,
    #[serde(
        default = "default_volume_down",
        deserialize_with = "command_template_from_args"
    )]
    pub vol_down_command: CommandTemplate,
    #[serde(default, deserialize_with = "command_template_option_from_args")]
    pub mute_command: Option<CommandTemplate>,
    #[serde(default, deserialize_with = "command_template_option_from_args")]
    pub tv_turned_on_command: Option<CommandTemplate>,
    #[serde(default, deserialize_with = "command_template_option_from_args")]
    pub tv_turned_off_command: Option<CommandTemplate>,

    #[serde(default = "default_power_poll_interval_ms")]
    pub power_poll_interval_ms: u64,
}

fn command_template_from_args<'de, D>(deserializer: D) -> Result<CommandTemplate, D::Error>
where
    D: Deserializer<'de>,
{
    let args: Vec<String> = de::Deserialize::deserialize(deserializer)?;
    if args.is_empty() {
        return Err(de::Error::custom("app missing"));
    }
    Ok(CommandTemplate(args))
}

fn command_template_option_from_args<'de, D>(
    deserializer: D,
) -> Result<Option<CommandTemplate>, D::Error>
where
    D: Deserializer<'de>,
{
    let args: Option<Vec<String>> = de::Deserialize::deserialize(deserializer)?;

    if let Some(ref args) = args {
        if args.is_empty() {
            return Ok(None);
        }
    }
    Ok(args.map(CommandTemplate))
}

fn default_volume_up() -> CommandTemplate {
    vec!["amixer", "set", "DSPVolume", "1%+"].into()
}

fn default_volume_down() -> CommandTemplate {
    vec!["amixer", "set", "DSPVolume", "1%-"].into()
}

fn default_device_name() -> String {
    "Hifiberry".to_owned()
}
fn default_hdmi_port() -> CString {
    CString::new("").unwrap()
}

const fn default_power_poll_interval_ms() -> u64 {
    5000
}

pub static CONFIG: OnceCell<AppConfig> = OnceCell::new();
