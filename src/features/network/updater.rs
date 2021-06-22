use std::fmt;

use log::info;

use crate::error::Result;
use crate::error::WrapErrorExt;
use crate::feature;
use crate::wrapper::process;

use super::Data;
use super::UpdateConfig;
use super::FEATURE_NAME;

enum IpAddress {
    V4,
    V6,
}

impl fmt::Display for IpAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "IPv{}",
            match self {
                IpAddress::V4 => 4,
                IpAddress::V6 => 6,
            }
        )
    }
}

pub(super) struct Updater {
    data: Data,
    config: UpdateConfig,
}

impl Updater {
    pub(super) const fn new(data: Data, config: UpdateConfig) -> Self {
        Self { data, config }
    }

    fn get_if_enabled<F: Fn() -> Option<String>>(
        &self,
        enabled: bool,
        builder: F,
    ) -> Option<String> {
        if enabled { builder() } else { None }
    }
}

impl feature::Updatable for Updater {
    fn renderable(&self) -> &dyn feature::Renderable {
        &self.data
    }

    fn update(&mut self) -> Result<()> {
        let ipv4 = self.get_if_enabled(self.config.show_ipv4, || ip_address(&IpAddress::V4));
        let ipv6 = self.get_if_enabled(self.config.show_ipv6, || ip_address(&IpAddress::V6));
        let essid = self.get_if_enabled(self.config.show_essid, essid);

        self.data.update(ipv4, ipv6, essid);

        Ok(())
    }
}

fn essid() -> Option<String> {
    let command = process::Command::new("iwgetid", &["-r"]);
    let output = command
        .output()
        .wrap_error(FEATURE_NAME, "essid {} could not be fetched");

    normalize_output(output)
}

fn ip_address(address_type: &IpAddress) -> Option<String> {
    let ip =
        curl("https://ipinfo.io/ip")
        .output()
        .wrap_error(
            FEATURE_NAME,
            format!("ip address {} could not be fetched", address_type),
        );
    let city =
        curl("https://ipinfo.io/city")
        .output()
        .wrap_error(
            FEATURE_NAME,
            format!("geoip {} could not be fetched", address_type),
        );

    return merge_opts(normalize_output(ip), normalize_output(city));
}

fn curl(url: &str) -> process::Command {
    process::Command::new("curl", &[ "--insecure", url ])
}

fn merge_opts(first: Option<String>, second: Option<String>) -> Option<String> {
    match first {
        None => None,
        Some(term1) => match second {
            None => Some(term1),
            Some(term2) => Some(format!("{} {}", term1, term2))
        }
    }
}

fn normalize_output(output: Result<String>) -> Option<String> {
    match output {
        Ok(string) => {
            if string.is_empty() {
                None
            } else {
                Some(string)
            }
        },
        Err(error) => {
            info!("{}", error);
            None
        },
    }
}
