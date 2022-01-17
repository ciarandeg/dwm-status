use std::collections::HashMap;
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
    city_cache: HashMap<String, String>
}

impl Updater {
    pub(super) fn new(data: Data, config: UpdateConfig) -> Self {
        Self { data, config, city_cache: HashMap::new() }
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
        let city = {
            let ip_if_any = ipv4.as_ref().or(ipv6.as_ref());
            let city = match ip_if_any {
                Some(ip) => self.get_if_enabled(self.config.show_city, | | {
                    match self.city_cache.get(ip) {
                        Some(city) => Some(city.clone()),
                        None => city()
                    }
                } ),
                None => None
            };
            if let Some(ip) = ip_if_any {
                if let Some(city) = &city {
                    if !self.city_cache.contains_key(ip) {
                        info!("Network: Cached city info for {}", ip);
                        self.city_cache.insert(ip.clone(), city.clone());
                    }
                }
            }
            city
        };

        self.data.update(ipv4, ipv6, essid, city);

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

fn city() -> Option<String> {
    use std::process::Command; // using stdlib because wrapper requires raw strings

    info!("Network: Grabbing geoip info from external API");

    let ip_address = match ip_address(&IpAddress::V4) {
        Some(string) => string,
        None => return None
    };

    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("curl http://ip-api.com/line/{}?fields=city", ip_address))
        .output()
        .expect("failed to execute geoip lookup");

    let geoip = String::from_utf8(output.stdout).expect("UTF8 parsing failed");
    geoip.split_whitespace().next().map(|raw| { raw.to_string() })
}

fn ip_address(address_type: &IpAddress) -> Option<String> {
    let command = process::Command::new(
        "dig",
        &[
            // decrease time and tries because commands are executed synchronously
            // TODO: make asychronous
            "+time=3",  // default: 5 seconds
            "+tries=1", // default: 3
            "TXT",
            match address_type {
                IpAddress::V4 => "-4",
                IpAddress::V6 => "-6",
            },
            "@ns1.google.com",
            "o-o.myaddr.l.google.com",
            "+short",
        ],
    );

    let output = command.output().wrap_error(
        FEATURE_NAME,
        format!("ip address {} could not be fetched", address_type),
    );

    // Google's myaddr service wraps ip in double quotes
    let parsed = output.map(|result| {
        let tokens = result.split('"').collect::<Vec<&str>>();
        tokens[1].to_string()
    });
    normalize_output(parsed)
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
