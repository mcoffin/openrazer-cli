extern crate byteorder;
#[macro_use]
extern crate clap;

use clap::{ Arg, App, Error, ErrorKind, SubCommand };
use std::{ fs, io };
use std::path::{ Path, PathBuf };

use byteorder::WriteBytesExt;
use io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceMode {
    Unmanaged,
    Managed,
}

impl Into<[u8; 2]> for DeviceMode {
    fn into(self) -> [u8; 2] {
        use DeviceMode::*;
        let first: u8 = match self {
            Unmanaged => 0x0,
            Managed => 0x3,
        };
        [first, 0x0]
    }
}

impl Default for DeviceMode {
    fn default() -> DeviceMode {
        DeviceMode::Unmanaged
    }
}

#[derive(Debug, Clone)]
enum Action {
    SetDpi(u16),
    NoCommand,
}

impl Default for Action {
    fn default() -> Action {
        Action::NoCommand
    }
}

#[derive(Debug, Clone)]
struct Config {
    device_path: PathBuf,
}

impl Config {
    pub fn set_device_mode(&self, mode: DeviceMode) -> io::Result<()> {
        let device_mode_path = self.device_path.join("device_mode");
        let mut file = try!(fs::OpenOptions::new().write(true).open(device_mode_path));
        let bytes: [u8; 2] = mode.into();
        file.write_all(&bytes)
    }

    pub fn set_dpi(&self, dpi: u16) -> io::Result<()> {
        try!(self.set_device_mode(DeviceMode::Managed));
        let dpi_path = self.device_path.join("dpi");
        let mut file = try!(fs::OpenOptions::new().write(true).open(dpi_path));
        file.write_u16::<byteorder::BigEndian>(dpi)
    }
}

fn main() {
    let r = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!(", "))
        .about(crate_description!())
        .arg(Arg::with_name("device")
             .short("d")
             .long("device")
             .value_name("DEVICE")
             .help("Device path")
             .long_help("Path to device directory in the /sys filesystem")
             .takes_value(true)
             .required(true))
        .subcommand(SubCommand::with_name("set-dpi")
                    .about("Sets the dpi of the given device")
                    .arg(Arg::with_name("dpi").required(true)))
        .get_matches_safe()
        .and_then(|matches| {
            let config = Config {
                device_path: matches.value_of("device").map(Path::new).unwrap().to_owned(),
            };
            let action = if let Some(matches) = matches.subcommand_matches("set-dpi") {
                matches.value_of("dpi")
                    .and_then(|dpi| dpi.parse::<u16>().ok())
                    .map(|dpi| Action::SetDpi(dpi))
                    .map(|action| Ok(action))
                    .unwrap_or_else(|| Err(Error::with_description("Target dpi must be an integer", ErrorKind::InvalidValue)))
            } else {
                Ok(Action::NoCommand)
            };
            action.map(|action| (config, action))
        });
    match r {
        Err(e) => e.exit(),
        Ok((config, action)) => {
            match action {
                Action::NoCommand => {},
                Action::SetDpi(dpi) => config.set_dpi(dpi).unwrap(),
            }
        },
    }
}
