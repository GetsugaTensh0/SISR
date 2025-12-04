use std::collections::HashMap;

use super::device::SDLDevice;

#[derive(Debug, Clone)]
pub enum SdlValue {
    String(String),
    OptString(Option<String>),
    U16(u16),
    OptU16(Option<u16>),
    U32(u32),
    Bool(bool),
}

impl std::fmt::Display for SdlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SdlValue::String(s) => write!(f, "{}", s),
            SdlValue::OptString(Some(s)) => write!(f, "{}", s),
            SdlValue::OptString(None) => write!(f, "N/A"),
            SdlValue::U16(v) => write!(f, "{}", v),
            SdlValue::OptU16(Some(v)) => write!(f, "{}", v),
            SdlValue::OptU16(None) => write!(f, "N/A"),
            SdlValue::U32(v) => write!(f, "{}", v),
            SdlValue::Bool(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SdlDeviceInfo {
    pub is_gamepad: bool,
    pub properties: HashMap<String, SdlValue>,
}

impl From<&SDLDevice> for SdlDeviceInfo {
    fn from(device: &SDLDevice) -> Self {
        let mut properties = HashMap::new();

        match device {
            SDLDevice::Joystick(js) => {
                properties.insert("name".into(), SdlValue::String(js.name()));
                properties.insert("id".into(), SdlValue::U32(js.id()));
                properties.insert("guid".into(), SdlValue::String(js.guid().string()));
                properties.insert("connected".into(), SdlValue::Bool(js.connected()));
                properties.insert("num_axes".into(), SdlValue::U32(js.num_axes()));
                properties.insert("num_buttons".into(), SdlValue::U32(js.num_buttons()));
                properties.insert("num_hats".into(), SdlValue::U32(js.num_hats()));

                SdlDeviceInfo {
                    is_gamepad: false,
                    properties,
                }
            }
            SDLDevice::Gamepad(gp) => {
                properties.insert("name".into(), SdlValue::OptString(gp.name()));
                properties.insert("id".into(), SdlValue::U32(gp.id().unwrap_or(0)));
                properties.insert("path".into(), SdlValue::OptString(gp.path()));
                properties.insert("type".into(), SdlValue::String(gp.r#type().string()));
                properties.insert(
                    "real_type".into(),
                    SdlValue::String(gp.real_type().string()),
                );
                properties.insert("connected".into(), SdlValue::Bool(gp.connected()));
                properties.insert("vendor_id".into(), SdlValue::OptU16(gp.vendor_id()));
                properties.insert("product_id".into(), SdlValue::OptU16(gp.product_id()));
                properties.insert(
                    "product_version".into(),
                    SdlValue::OptU16(gp.product_version()),
                );
                properties.insert(
                    "firmware_version".into(),
                    SdlValue::OptU16(gp.firmware_version()),
                );
                properties.insert(
                    "serial_number".into(),
                    SdlValue::OptString(gp.serial_number()),
                );
                properties.insert("player_index".into(), SdlValue::OptU16(gp.player_index()));

                SdlDeviceInfo {
                    is_gamepad: true,
                    properties,
                }
            }
        }
    }
}
