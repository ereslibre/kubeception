use dbus;
use dbus::{BusType, Connection, Message};

use std;

pub struct Systemd {}

pub enum SystemdError {
    UnknownError,
}

impl From<dbus::Error> for SystemdError {
    fn from(_error: dbus::Error) -> SystemdError {
        SystemdError::UnknownError
    }
}

impl From<std::string::String> for SystemdError {
    fn from(_error: std::string::String) -> SystemdError {
        SystemdError::UnknownError
    }
}

impl Systemd {
    pub fn start(service: &str) -> Result<(), SystemdError> {
        let connection = Connection::get_private(BusType::System)?;
        let message = Message::new_method_call(
            "org.freedesktop.systemd1",
            "/org/freedesktop/systemd1",
            "org.freedesktop.systemd1.Manager",
            "StartUnit",
        )?
            .append2(service, "replace");
        connection.send_with_reply_and_block(message, 2000)?;
        Ok(())
    }

    pub fn enable(service: &str) -> Result<(), SystemdError> {
        let connection = Connection::get_private(BusType::System)?;
        let message = Message::new_method_call(
            "org.freedesktop.systemd1",
            "/org/freedesktop/systemd1",
            "org.freedesktop.systemd1.Manager",
            "EnableUnitFiles",
        )?
            .append3(vec![service], false, true);
        connection.send_with_reply_and_block(message, 2000)?;
        Ok(())
    }

    pub fn restart(service: &str) -> Result<(), SystemdError> {
        let connection = Connection::get_private(BusType::System)?;
        let message = Message::new_method_call(
            "org.freedesktop.systemd1",
            "/org/freedesktop/systemd1",
            "org.freedesktop.systemd1.Manager",
            "RestartUnit",
        )?
            .append2(service, "replace");
        connection.send_with_reply_and_block(message, 2000)?;
        Ok(())
    }
}
