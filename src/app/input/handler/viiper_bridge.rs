use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::MutexGuard;

use anyhow::{Result, anyhow};
use tracing::{info, warn};
use viiper_client::{DeviceStream, ViiperClient};

use super::State;
use crate::app::input::device::Device;

pub(super) struct ViiperBridge {
    client: Option<ViiperClient>,
    streams: HashMap<u32, DeviceStream>,
}

impl ViiperBridge {
    pub fn new(viiper_address: Option<SocketAddr>) -> Self {
        Self {
            client: match viiper_address {
                Some(addr) => Some(ViiperClient::new(addr)),
                None => {
                    warn!("No VIIPER address provided; VIIPER integration disabled");
                    None
                }
            },
            streams: HashMap::new(),
        }
    }

    pub fn create_device(
        &self,
        device: &mut Device,
        guard: &mut MutexGuard<'_, State>,
    ) -> Result<()> {
        let bus_id = match guard.viiper_bus {
            Some(id) => id,
            None => self.create_bus(guard)?,
        };

        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("No VIIPER client available"))?;

        let response = client
            .bus_device_add(
                bus_id,
                &viiper_client::types::DeviceCreateRequest {
                    r#type: Some(device.viiper_type.clone()),
                    id_vendor: None,
                    id_product: None,
                },
            )
            .map_err(|e| anyhow!("Failed to create VIIPER device: {}", e))?;

        info!("Created VIIPER device with {:?}", response);
        device.viiper_device = Some(response);
        Ok(())
    }

    pub fn connect_device(&mut self, device: &mut Device) -> Result<()> {
        let viiper_dev = device
            .viiper_device
            .as_ref()
            .ok_or_else(|| anyhow!("Device has no VIIPER device"))?;

        let client = self
            .client
            .as_mut()
            .ok_or_else(|| anyhow!("No VIIPER client available"))?;

        let dev_stream = client
            .connect_device(viiper_dev.bus_id, &viiper_dev.dev_id)
            .map_err(|e| anyhow!("Failed to connect VIIPER device: {}", e))?;

        self.streams.insert(device.id, dev_stream);
        info!("Connected VIIPER device {:?}", device.viiper_device);
        Ok(())
    }

    pub fn disconnect_device(&mut self, device_id: u32) {
        if self.streams.remove(&device_id).is_some() {
            info!("Disconnected VIIPER device for pad {}", device_id);
        }
    }

    pub fn create_bus(&self, guard: &mut MutexGuard<'_, State>) -> Result<u32> {
        if guard.viiper_bus.is_some() {
            warn!("VIIPER bus already created; Recreating");
        }

        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("No VIIPER client available"))?;

        let response = client
            .bus_create(None)
            .map_err(|e| anyhow!("Failed to create VIIPER bus: {}", e))?;

        info!("Created VIIPER bus with ID {}", response.bus_id);
        guard.viiper_bus = Some(response.bus_id);
        Ok(response.bus_id)
    }
}
