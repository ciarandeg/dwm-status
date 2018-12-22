use super::FEATURE_NAME;
use error::*;
use io;

#[derive(Debug)]
pub(super) struct BacklightDevice {
    max: u32,
    path: String,
}

impl BacklightDevice {
    pub(super) fn new(device: &str) -> Result<Self> {
        let mut device = Self {
            max: 0,
            path: format!("/sys/class/backlight/{}", device),
        };

        device.max = device.get_brightness("max")?;

        Ok(device)
    }

    pub(super) fn brightness_file(&self) -> String {
        self.build_path("actual")
    }

    pub(super) fn value(&self) -> Result<u32> {
        let current = self.get_brightness("actual")?;
        let value = current * 100 / self.max;

        Ok(value)
    }

    fn build_path(&self, name: &str) -> String {
        format!("{}/{}_brightness", self.path, name)
    }

    fn get_brightness(&self, name: &str) -> Result<u32> {
        let brightness = io::read_int_from_file(&self.build_path(name))
            .wrap_error(FEATURE_NAME, format!("error reading {} brightness", name))?;

        Ok(brightness)
    }
}
