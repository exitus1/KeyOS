// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::BTreeMap;

use atsama5d27::{
    pmc::PeripheralId,
    udphs::{
        DmaControl, EndpointConfiguration, EndpointControl, EndpointDirection, EndpointStatus, UsbDevice,
    },
};
use gpio::{messages::IrqMessage, GpioPin, PinSettings};
use server::{
    send_archive, ArchiveHandler, BlockingScalarAsyncHandler, BlockingScalarHandler, BlockingScalarRequest,
    DeferredLendMut, DeferredLendMutHandler, ScalarEventHandler, ScalarHandler,
};
use usb::{
    device::{messages::*, SetupPacket, BLD_DEV_VERSION, MAJ_DEV_VERSION, MIN_DEV_VERSION},
    UsbError,
};
use utralib::{HW_UDPHS_BASE, HW_UDPHS_RAM_MEM, HW_UDPHS_RAM_MEM_LEN};
use xous::arch::irq::IrqNumber;

use super::messages::*;

gpio::use_api!();
power_manager::use_api!();

#[derive(server::Server)]
#[name = "os/usbdev"]
pub struct UsbDeviceServer {
    power_manager: PowerManagerApi,
    hw: UsbDevice,
    pending_address: Option<u8>,
    otg_device_connected: bool,
    vbus_has_power: bool,
    is_configured: bool,
    should_be_enabled: bool,
    enabled: bool,
    interfaces: Vec<RegisteredInterface>,
    capabilities: Vec<RegisteredCapability>,
    setup_responders: Vec<xous::CID>,
    config_descriptor: Vec<u8>,
    bos_descriptor: Vec<u8>,
    remaining_setup_tx_data: Vec<u8>,
    end_setup_tx_with_short_packet: bool,
    endpoints: BTreeMap<u8, RuntimeEndpointData>,
    connection_waiters: Vec<BlockingScalarRequest<WaitForConnection>>,
    custom_vid: Option<u16>,
    custom_pid: Option<u16>,
}

#[derive(Debug, Default, Clone, server::Permissions)]
#[server_name = "os/usbdev"]
#[all_permissions]
struct InternalPermissions;

struct RegisteredInterface {
    descriptors: Vec<u8>,
}

struct RegisteredCapability {
    descriptors: Vec<u8>,
}

struct InterruptContext {
    conn: server::CheckedConn<InternalPermissions>,
    hw: UsbDevice,
}

struct RuntimeEndpointData {
    properties: EndpointProperties,
    ongoing_read: Option<DeferredLendMut<ReadEndpoint>>,
    ongoing_write: Option<DeferredLendMut<WriteEndpoint>>,
}

const EPT0_MAX_PACKET_SIZE: usize = 0x40;

const MANUFACTURER: &str = "Foundation Devices, Inc.";
const PRODUCT: &str = "Passport Prime";

#[rustfmt::skip]
const DEVICE_DESCRIPTOR: [u8; 0x12] = [
    0x12, // bLength
    0x01, // bDescriptorType: Device
    0x10, 0x02, // bcdUSB: 2.1 for BOS support
    0xef, 0x02, 0x01, // bDeviceClass, bDeviceSubClass and bDeviceProtocol
                      // These values must be used if we have IADs
                      // see https://learn.microsoft.com/en-us/windows-hardware/drivers/usbcon/usb-interface-association-descriptor
    EPT0_MAX_PACKET_SIZE as u8, // bMaxPacketSize0
    0x07, 0x13, // idVendor (Transcend)
    0x65, 0x01, // idProduct (Mass Storage Device)
    (MIN_DEV_VERSION << 4) | BLD_DEV_VERSION, MAJ_DEV_VERSION, // bcdDevice
    0x01, // iManufacturer (string index)
    0x02, // iProduct (string index)
    0x03, // iSerial (string index)
    0x01, // bNumConfigurations
];

const GET_STATUS: u8 = 0;
const SET_ADDRESS: u8 = 5;
const GET_DESCRIPTOR: u8 = 6;
const SET_CONFIGURATION: u8 = 9;

impl server::Server for UsbDeviceServer {
    fn on_start(&mut self, context: &mut server::ServerContext<Self>) {
        log::debug!("Claiming UDPHS IRQ");
        let int_ctx = Box::into_raw(Box::new(InterruptContext {
            conn: server::CheckedConn::default(),
            hw: self.hw.clone(),
        }));
        xous::claim_interrupt(IrqNumber::Udphs, udphs_irq_handler, int_ctx as *mut usize)
            .expect("Could not claim UHPHS interrupt");

        let gpio_api = GpioApi::default();
        gpio_api
            .claim_pin(GpioPin::UsbOtgId, PinSettings::InterruptBoth, false)
            .expect("Could not claim pin");

        log::debug!("Enabling OTG_ID IRQ");
        gpio_api.enable_irq(GpioPin::UsbOtgId, context).expect("Could not subscribe to gpio interrupt");

        gpio_api
            .claim_pin(GpioPin::UsbVbusIrq, PinSettings::InterruptBoth, false)
            .expect("Could not claim pin");

        log::debug!("Enabling VBUS IRQ");
        gpio_api.enable_irq(GpioPin::UsbVbusIrq, context).expect("Could not subscribe to gpio interrupt");

        self.vbus_has_power = gpio_api.get_pin(GpioPin::UsbVbusIrq).expect("Could not get VBUS pin status");
        self.handle_otg_pin_state(gpio_api.get_pin(GpioPin::UsbOtgId).expect("Could not get OTG pin status"));
        self.update_hw_enabled_state();
    }
}

impl UsbDeviceServer {
    pub fn new() -> Self {
        let udphs_banks = xous::map_memory(
            xous::MemoryAddress::new(HW_UDPHS_RAM_MEM),
            None,
            HW_UDPHS_RAM_MEM_LEN,
            xous::MemoryFlags::W | xous::MemoryFlags::DEV | xous::MemoryFlags::NO_CACHE,
        )
        .expect("Could not map UDPHS RAM");

        let udphs_csr = xous::map_memory(
            xous::MemoryAddress::new(HW_UDPHS_BASE),
            None,
            0x1000,
            xous::MemoryFlags::W | xous::MemoryFlags::DEV | xous::MemoryFlags::NO_CACHE,
        )
        .expect("Could not map UDPHS registers");

        let power_manager = PowerManagerApi::default();
        power_manager.enable_peripheral(PeripheralId::Udphs).unwrap();
        let mut hw = UsbDevice::new(udphs_csr.as_mut_ptr(), udphs_banks.as_mut_ptr());
        // Disable for now, it will be reenabled (and reset) when we read the OTG gpio line later
        hw.set_enabled(false);
        power_manager.disable_peripheral(PeripheralId::Udphs).unwrap();

        let capabilities = vec![RegisteredCapability {
            descriptors: vec![
                0x07, // bLength
                0x10, // bDescriptorType
                0x02, // bDevCapabilityType: USB 2.0 EXTENSION
                0, 0, 0, 0, // bmAttributes
            ],
        }];

        Self {
            power_manager,
            hw,
            pending_address: None,
            otg_device_connected: false,
            vbus_has_power: false,
            connection_waiters: Default::default(),
            interfaces: Default::default(),
            capabilities,
            setup_responders: Default::default(),
            config_descriptor: Default::default(),
            endpoints: Default::default(),
            bos_descriptor: Default::default(),
            is_configured: false,
            should_be_enabled: false,
            enabled: false,
            remaining_setup_tx_data: Default::default(),
            end_setup_tx_with_short_packet: false,
            custom_vid: None,
            custom_pid: None,
        }
    }

    fn update_hw_enabled_state(&mut self) {
        if self.should_be_enabled
            && self.vbus_has_power
            && !self.otg_device_connected
            && !self.config_descriptor.is_empty()
        {
            if !self.enabled {
                self.power_manager.enable_peripheral(PeripheralId::Udphs).unwrap();
                self.hw.set_enabled(true);
                self.enabled = true;
            }
        } else if self.enabled {
            self.hw.set_enabled(false);
            self.send_disconnected();
            self.power_manager.disable_peripheral(PeripheralId::Udphs).unwrap();
            self.enabled = false;
            self.is_configured = false;
        }
    }

    fn handle_otg_pin_state(&mut self, pin_state: bool) {
        if pin_state {
            log::debug!("OTG slave device disconnected, disabling power");
            self.power_manager.set_usb_boost(false).ok();
            self.otg_device_connected = false;
        } else {
            log::debug!("OTG slave device connected, enabling power to it");
            self.power_manager.set_usb_boost(true).ok();
            self.otg_device_connected = true;
        }
    }

    fn configure(&mut self) {
        for (ept_num, ept_data) in &mut self.endpoints {
            log::debug!("Setting up EP{ept_num} as {:?}", ept_data.properties);
            self.hw.reset_endpoint(*ept_num as usize);
            let ep = self.hw.endpoint(*ept_num as usize);

            let mut config = EndpointConfiguration(0);
            config.set_ept_size(ept_data.properties.max_packet_len.ilog2().saturating_sub(3));
            config.set_ept_type(ept_data.properties.ep_type);
            config.set_ept_dir(ept_data.properties.ep_direction);
            // See SAMA5D2 Datasheet Table 41-4: EPT_1 and EPT2 can have 3 banks.
            config.set_bank_number(if *ept_num == 1 || *ept_num == 2 { 3 } else { 2 });
            ep.cfg.set(config);
            assert!(ep.cfg.get().mapped());

            let mut control = EndpointControl(0);
            control.set_enable(true);
            control.set_auto_valid(true);
            ep.ctl_enable.set(control);
            self.hw.enable_dma_interrupt(*ept_num as usize);
        }
        self.is_configured = true;

        log::info!("Usb device configured");
        // Drop all waiters, which will return the blocking scalars to the callers.
        self.connection_waiters.truncate(0);
    }

    fn start_dma(&mut self, endpoint_number: u8, buf: *const u8, length: u16) {
        let mut control = DmaControl(0);
        control.set_enable(true);
        control.set_end_of_transfer_enable(true);
        control.set_end_of_buffer_enable(true);
        control.set_end_of_transfer_interrupt(true);
        control.set_end_of_buffer_interrupt(true);
        control.set_burst_lock(true);
        control.set_length(length);
        let dma = self.hw.dma(endpoint_number as usize);
        dma.address.set(xous::virt_to_phys(buf as usize).unwrap() as u32);
        dma.control.set(control);
    }

    fn to_string_descriptor(s: &str) -> Vec<u8> {
        let mut payload: Vec<u8> = s.encode_utf16().flat_map(|c| c.to_le_bytes()).collect();
        // Add a header of [Descriptor length, Descriptor type (3)]
        payload.insert(0, 0x03);
        payload.insert(0, payload.len() as u8 + 1);
        payload
    }

    fn send_disconnected(&mut self) {
        for ep in self.endpoints.values_mut() {
            if let Some(mut read) = ep.ongoing_read.take() {
                read.set_response(Err(UsbError::HostDisconnected));
            }
            if let Some(mut write) = ep.ongoing_write.take() {
                write.set_response(Err(UsbError::HostDisconnected));
            }
        }
    }

    fn recalculate_config_descriptor(&mut self) {
        self.config_descriptor = vec![
            // Configuration Descriptor
            0x09, // bLength
            0x02, // bDescriptorType: Configuration
            0x00, // wTotalLength (u16, fixed up later)
            0x00,
            self.interfaces.len() as u8, // bNumInterfaces
            0x01,                        // bConfigurationValue (used to call SetConfiguration)
            2,                           // iConfiguration: index to iProduct
            0xc0,                        // bmAttributes (self-powered, no remote wakeup)
            0x10,                        /* MaxPower: 32mA. This does not have to be accurate,
                                          * but if it's too large,
                                          * the device will be rejected with "insufficient available bus
                                          * power" */
        ];
        for interface in &self.interfaces {
            self.config_descriptor.extend_from_slice(&interface.descriptors);
        }
        self.config_descriptor[2] = self.config_descriptor.len() as u8;
        self.config_descriptor[3] = (self.config_descriptor.len() >> 8) as u8;
    }

    // Returns the registered endpoint numbers.
    fn register_interface(&mut self, msg: RegisterInterface) -> Vec<u8> {
        let mut descriptors = Vec::new();

        if msg.associated_interface_count > 0 {
            descriptors.extend_from_slice(&[
                0x08,                           // bLength
                0x0b,                           // bDescriptorType: Interface Association
                self.interfaces.len() as u8,    // bFirstInterface: First iface
                msg.associated_interface_count, // Two ifaces
                msg.if_class,                   // bInterfaceClass
                msg.if_subclass,                // bInterfaceSubClass
                msg.if_protocol,                // bInterfaceProtocol
                2,                              // iInterface: index to iProduct
            ]);
        }

        descriptors.extend_from_slice(&[
            // Interface Descriptor
            0x09,                        // bLength
            0x04,                        // bDescriptorType: Interface
            self.interfaces.len() as u8, // bInterfaceNumber
            0x00,                        // bAlternateSetting
            msg.endpoints.len() as u8,   // bNumEndpoints
            msg.if_class,                // bInterfaceClass
            msg.if_subclass,             // bInterfaceSubClass
            msg.if_protocol,             // bInterfaceProtocol
            2,                           // iInterface: index to iProduct
        ]);
        descriptors.extend_from_slice(&msg.interface_functional_descriptors);
        let mut result = Vec::new();
        for properties in msg.endpoints {
            let ep_number = (self.endpoints.len() + 1) as u8;
            descriptors.extend_from_slice(&[
                // Endpoint Descriptor
                0x07, // bLength
                0x05, // bDescriptorType: Endpoint
                ep_number + if properties.ep_direction == EndpointDirection::In { 0x80 } else { 0 }, // bEndpointAddress
                properties.ep_type as u8, // bmAttributes
                properties.max_packet_len as u8, // wMaxPacketSize
                (properties.max_packet_len >> 8) as u8,
                properties.interval, // bInterval
            ]);
            self.endpoints.insert(
                ep_number,
                RuntimeEndpointData { properties, ongoing_read: None, ongoing_write: None },
            );
            result.push(ep_number);
        }
        self.interfaces.push(RegisteredInterface { descriptors });
        result
    }

    fn recalculate_bos_descriptor(&mut self) {
        self.bos_descriptor = vec![
            // Binary Object Store Descriptor
            0x05, // bLength
            0x0f, // bDescriptorType: Binary Object Store
            0x00, // wTotalLength (u16, fixed up later)
            0x00,
            self.capabilities.len() as u8, // bNumDeviceCaps
        ];
        for capability in &self.capabilities {
            self.bos_descriptor.extend_from_slice(&capability.descriptors);
        }
        self.bos_descriptor[2] = self.bos_descriptor.len() as u8;
        self.bos_descriptor[3] = (self.bos_descriptor.len() >> 8) as u8;
    }

    fn register_capability(&mut self, msg: RegisterCapability) {
        let mut descriptors = vec![
            // Platform Device Capability
            20 + msg.capability_functional_descriptors.len() as u8, // bLength
            msg.cap_type,                                           // bDescriptorType
            msg.cap_subtype,                                        // bDevCapabilityType
            0,                                                      // bReserved
        ];
        descriptors.extend_from_slice(&msg.cap_uuid);
        descriptors.extend_from_slice(&msg.capability_functional_descriptors);
        self.capabilities.push(RegisteredCapability { descriptors });
    }

    fn send_remaining_setup_tx(&mut self) {
        let mut bytes = core::mem::take(&mut self.remaining_setup_tx_data);
        if bytes.len() >= EPT0_MAX_PACKET_SIZE {
            self.remaining_setup_tx_data = bytes.split_off(EPT0_MAX_PACKET_SIZE);
        } else {
            self.end_setup_tx_with_short_packet = false;
        }
        log::trace!("Sending setup response {bytes:02x?}");
        self.hw.write_endpoint_memory(0, 0, &bytes);
        let mut status = EndpointStatus(0x0);
        status.set_tx_packet_ready(true);
        self.hw.endpoint(0).status_set.set(status);
    }
}

impl ArchiveHandler<RegisterInterface> for UsbDeviceServer {
    fn handle(
        &mut self,
        msg: RegisterInterface,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<Vec<u8>, UsbError> {
        log::info!("Registering interface class {} with {} endpoints", msg.if_class, msg.endpoints.len());
        let result = self.register_interface(msg);
        self.recalculate_config_descriptor();
        self.recalculate_bos_descriptor();
        self.update_hw_enabled_state();

        Ok(result)
    }
}

impl ArchiveHandler<RegisterCapability> for UsbDeviceServer {
    fn handle(
        &mut self,
        msg: RegisterCapability,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<(), UsbError> {
        log::info!("Registering capability type {}:{}", msg.cap_type, msg.cap_subtype);
        self.register_capability(msg);
        self.recalculate_bos_descriptor();

        Ok(())
    }
}

impl BlockingScalarHandler<RegisterSetupResponder> for UsbDeviceServer {
    fn handle(
        &mut self,
        msg: RegisterSetupResponder,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> Result<(), UsbError> {
        self.setup_responders.push(msg.0);
        Ok(())
    }
}

impl ScalarHandler<SetEndpointStalled> for UsbDeviceServer {
    fn handle(
        &mut self,
        msg: SetEndpointStalled,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) {
        if !self.is_configured {
            log::warn!("SetEndpointStalled called when device was not configured");
            return;
        }
        log::debug!("Setting stall on endpoint {msg:?}");
        let mut status = EndpointStatus(0x0);
        status.set_force_stall(true);
        if msg.stalled {
            self.hw.endpoint(msg.endpoint as usize).status_set.set(status);
        } else {
            self.hw.endpoint(msg.endpoint as usize).status_clr.set(status);
        }
    }
}

impl BlockingScalarAsyncHandler<WaitForConnection> for UsbDeviceServer {
    fn handle(
        &mut self,
        msg: BlockingScalarRequest<WaitForConnection>,
        _context: &mut server::ServerContext<Self>,
    ) {
        if !self.is_configured {
            self.connection_waiters.push(msg);
        }
    }

    fn default_response() {}
}

impl DeferredLendMutHandler<ReadEndpoint> for UsbDeviceServer {
    fn handle(&mut self, mut msg: DeferredLendMut<ReadEndpoint>, _context: &mut server::ServerContext<Self>) {
        if !self.is_configured {
            msg.set_response(Err(UsbError::HostDisconnected));
            return;
        }
        let endpoint_number = msg.body().endpoint;
        let Some(endpoint) = self.endpoints.get_mut(&endpoint_number) else {
            msg.set_response(Err(UsbError::NotFound));
            return;
        };
        if endpoint.ongoing_read.is_some() {
            msg.set_response(Err(UsbError::Busy));
            return;
        }
        if endpoint.properties.ep_direction != EndpointDirection::Out {
            msg.set_response(Err(UsbError::WrongDirection));
            return;
        }
        log::trace!("Reading {} bytes on EP{}", msg.body().length, msg.body().endpoint);
        xous::flush_cache(msg.body().buf, xous::CacheOperation::Invalidate).ok();
        self.start_dma(endpoint_number, msg.body().buf.as_ptr(), msg.body().length);
        self.endpoints.get_mut(&endpoint_number).unwrap().ongoing_read = Some(msg);
    }

    fn default_response() -> <ReadEndpoint as server::LendMut>::Response { Err(UsbError::HostDisconnected) }
}

impl DeferredLendMutHandler<WriteEndpoint> for UsbDeviceServer {
    fn handle(
        &mut self,
        mut msg: DeferredLendMut<WriteEndpoint>,
        _context: &mut server::ServerContext<Self>,
    ) {
        if !self.is_configured {
            msg.set_response(Err(UsbError::HostDisconnected));
            return;
        }
        let endpoint_number = msg.body().endpoint;
        let Some(endpoint) = self.endpoints.get_mut(&endpoint_number) else {
            msg.set_response(Err(UsbError::NotFound));
            return;
        };
        if endpoint.ongoing_write.is_some() {
            msg.set_response(Err(UsbError::Busy));
            return;
        }
        if endpoint.properties.ep_direction != EndpointDirection::In {
            msg.set_response(Err(UsbError::WrongDirection));
            return;
        }
        log::trace!("Writing {} bytes on EP{}", msg.body().length, msg.body().endpoint);
        xous::flush_cache(msg.body().buf, xous::CacheOperation::Clean).ok();
        self.start_dma(endpoint_number, msg.body().buf.as_ptr(), msg.body().length);
        self.endpoints.get_mut(&endpoint_number).unwrap().ongoing_write = Some(msg);
    }

    fn default_response() -> <WriteEndpoint as server::LendMut>::Response { Err(UsbError::HostDisconnected) }
}

impl BlockingScalarHandler<NumInterfaces> for UsbDeviceServer {
    fn handle(
        &mut self,
        _msg: NumInterfaces,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> usize {
        self.interfaces.len()
    }
}

impl BlockingScalarHandler<ResetController> for UsbDeviceServer {
    fn handle(
        &mut self,
        _msg: ResetController,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> <ResetController as server::BlockingScalar>::Response {
        if self.enabled {
            self.hw.set_enabled(false);
            self.send_disconnected();
            self.hw.set_enabled(true);
        }
        Ok(())
    }
}

impl ScalarHandler<SetVidPid> for UsbDeviceServer {
    fn handle(&mut self, msg: SetVidPid, _sender: xous::PID, _context: &mut server::ServerContext<Self>) {
        self.custom_vid = msg.vid;
        self.custom_pid = msg.pid;
    }
}

impl ScalarHandler<EndOfReset> for UsbDeviceServer {
    fn handle(&mut self, _msg: EndOfReset, sender: xous::PID, _context: &mut server::ServerContext<Self>) {
        if sender != xous::current_pid().unwrap() {
            return;
        }

        self.send_disconnected();
        self.is_configured = false;

        log::info!("Got End of Reset");
        let ep0 = self.hw.endpoint(0);

        let mut config = EndpointConfiguration(0);
        config.set_ept_size(3); // Size: 8<<3 == 0x40
        config.set_bank_number(1);
        ep0.cfg.set(config);
        if !ep0.cfg.get().mapped() {
            // This should only happen if the host disconnects between the UDPHS EOR signal
            // and us reaching this point.
            log::warn!("Could not map EP0");
            return;
        }

        let mut control = EndpointControl(0);
        control.set_enable(true);
        control.set_received_setup_interupt(true);
        control.set_received_out_interrupt(true);
        control.set_transmission_complete_interrupt(true);
        ep0.ctl_enable.set(control);
        self.hw.enable_endpoint_interrupt(0);
    }
}

impl ScalarHandler<SetupPacket> for UsbDeviceServer {
    fn handle(&mut self, msg: SetupPacket, sender: xous::PID, _context: &mut server::ServerContext<Self>) {
        if sender != xous::current_pid().unwrap() {
            return;
        }
        log::trace!("Setup received: {msg:02x?}");
        let response = match (msg.request_type, msg.request) {
            (0x80, GET_STATUS) => Some(vec![1, 0]), // Self Powered
            (0x80, GET_DESCRIPTOR) => {
                match msg.value {
                    0x100 => {
                        // Type: device(1), index: 0
                        let mut response_bytes = Vec::new();
                        response_bytes.extend_from_slice(&DEVICE_DESCRIPTOR);
                        if let Some(custom_vid) = self.custom_vid {
                            response_bytes[9] = (custom_vid >> 8) as u8;
                            response_bytes[8] = (custom_vid & 0xff) as u8;
                        }
                        if let Some(custom_pid) = self.custom_pid {
                            response_bytes[11] = (custom_pid >> 8) as u8;
                            response_bytes[10] = (custom_pid & 0xff) as u8;
                        }
                        response_bytes.extend_from_slice(&self.config_descriptor);
                        Some(response_bytes)
                    }
                    0x200 => {
                        // Type: configuration(2), index: 0
                        Some(self.config_descriptor.clone())
                    }
                    0x300 => {
                        // Type: string(3), index: 0 (languages)
                        Some(Vec::from([0x04, 0x03, 0x09, 0x04]))
                    }
                    0x301 => {
                        // Type: string(3), index: 1 (manufacturer, see DEVICE_DESCRIPTOR)
                        Some(Self::to_string_descriptor(MANUFACTURER))
                    }
                    0x302 => {
                        // Type: string(3), index: 2 (product, see DEVICE_DESCRIPTOR)
                        Some(Self::to_string_descriptor(PRODUCT))
                    }
                    0x303 => {
                        // Type: string(3), index: 3 (serial, see DEVICE_DESCRIPTOR)
                        Some(Self::to_string_descriptor(
                            &crate::DEVICE_NAME.lock().unwrap_or_else(|e| e.into_inner()).clone(),
                        ))
                    }
                    0xF00 => {
                        // Type: bos(15), index: 0
                        Some(self.bos_descriptor.clone())
                    }
                    _ => {
                        log::warn!("Unknown descriptor request: {msg:02x?}");
                        None
                    }
                }
            }
            (0x00, SET_ADDRESS) => {
                log::debug!("Set address (pending): {}", msg.value);
                // Only set the address once the STATUS phase (IN, i.e. transmission) is over
                self.pending_address = Some(msg.value as u8);
                Some(Vec::new())
            }
            (0x00, SET_CONFIGURATION) => {
                log::debug!("Set configuration: {}", msg.value);
                if !self.is_configured {
                    self.configure();
                }
                Some(Vec::new())
            }
            _ => self
                .setup_responders
                .iter()
                .find_map(|setup_responder| send_archive(*setup_responder, SetupPacketCallback(msg.clone()))),
        };
        match response {
            Some(mut bytes) => {
                bytes.truncate(msg.length as usize);
                self.end_setup_tx_with_short_packet = bytes.len() < msg.length as usize;
                self.remaining_setup_tx_data = bytes;
                self.send_remaining_setup_tx();
            }
            None => {
                log::trace!("Stalling control endpoint");
                let mut status = EndpointStatus(0x0);
                status.set_force_stall(true);
                self.hw.endpoint(0).status_set.set(status);
            }
        }
    }
}

impl ScalarHandler<Ep0RxComplete> for UsbDeviceServer {
    fn handle(&mut self, _msg: Ep0RxComplete, sender: xous::PID, _context: &mut server::ServerContext<Self>) {
        if sender != xous::current_pid().unwrap() {
            return;
        }
        log::trace!("Rx complete");
    }
}

impl ScalarHandler<Ep0TxComplete> for UsbDeviceServer {
    fn handle(&mut self, _msg: Ep0TxComplete, sender: xous::PID, _context: &mut server::ServerContext<Self>) {
        if sender != xous::current_pid().unwrap() {
            return;
        }
        log::trace!("Tx complete");
        if let Some(addr) = self.pending_address.take() {
            log::debug!("Set address: {}", addr);
            self.hw.set_address(addr);
        }
        if !self.remaining_setup_tx_data.is_empty() || self.end_setup_tx_with_short_packet {
            self.send_remaining_setup_tx();
        }
    }
}

impl ScalarHandler<DmaInterrupt> for UsbDeviceServer {
    fn handle(&mut self, msg: DmaInterrupt, sender: xous::PID, _context: &mut server::ServerContext<Self>) {
        if sender != xous::current_pid().unwrap() {
            return;
        }
        log::trace!("Dma interrupt: {msg:?}");
        let Some(ep) = self.endpoints.get_mut(&msg.endpoint) else {
            return;
        };
        if let Some(mut read) = ep.ongoing_read.take() {
            read.set_response(Ok((read.body().length - msg.status.length()) as usize))
        }
        if let Some(mut write) = ep.ongoing_write.take() {
            write.set_response(Ok((write.body().length - msg.status.length()) as usize))
        }
    }
}

impl ScalarEventHandler<IrqMessage> for UsbDeviceServer {
    fn handle(&mut self, msg: IrqMessage, _sender: xous::PID, _context: &mut server::ServerContext<Self>) {
        log::trace!("GPIO IRQ: {msg:?}");
        match msg.pin {
            GpioPin::UsbOtgId => self.handle_otg_pin_state(msg.is_high),
            GpioPin::UsbVbusIrq => self.vbus_has_power = msg.is_high,
            _ => log::warn!("Unexpected GPIO IRQ: {msg:?}"),
        }
        self.update_hw_enabled_state();
    }
}

impl ScalarHandler<SetDeviceEmulationEnabled> for UsbDeviceServer {
    fn handle(
        &mut self,
        msg: SetDeviceEmulationEnabled,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) {
        self.should_be_enabled = msg.0;
        self.update_hw_enabled_state();
    }
}

impl BlockingScalarHandler<IsDeviceEmulationEnabled> for UsbDeviceServer {
    fn handle(
        &mut self,
        _msg: IsDeviceEmulationEnabled,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> bool {
        self.enabled
    }
}

impl BlockingScalarHandler<IsDeviceEmulationConnected> for UsbDeviceServer {
    fn handle(
        &mut self,
        _msg: IsDeviceEmulationConnected,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> bool {
        self.is_configured
    }
}

impl BlockingScalarHandler<IsCableConnected> for UsbDeviceServer {
    fn handle(
        &mut self,
        _msg: IsCableConnected,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> bool {
        self.vbus_has_power
    }
}

impl BlockingScalarHandler<IsDeviceMode> for UsbDeviceServer {
    fn handle(
        &mut self,
        _msg: IsDeviceMode,
        _sender: xous::PID,
        _context: &mut server::ServerContext<Self>,
    ) -> bool {
        !self.otg_device_connected
    }
}

fn udphs_irq_handler(_irq_no: usize, arg: *mut usize) {
    let context = unsafe { &mut *(arg as *mut InterruptContext) };
    let interrupts = context.hw.interrupt_status();
    if interrupts.end_of_reset() {
        context.conn.send_scalar_nowait(EndOfReset).ok();
    }
    if interrupts.endpoint(0) != 0 {
        let status = context.hw.endpoint(0).status.get();
        let mut clear = EndpointStatus(0x0);
        if status.received_setup() {
            clear.set_received_setup(true);
            if status.byte_count() == 8 {
                let mut setup_data = [0; 8];
                context.hw.read_endpoint_memory(0, 0, &mut setup_data);
                context.conn.send_scalar_nowait(SetupPacket::from_bytes(&setup_data)).ok();
            }
        }
        if status.received_out() {
            clear.set_received_out(true);
            // TODO: we throw away the received bytes. If we ever need setup packets with incoming data,
            //       this is the place to implement that.
            context.conn.send_scalar_nowait(Ep0RxComplete).ok();
        }
        if status.transmission_complete() {
            clear.set_transmmission_complete(true);
            context.conn.send_scalar_nowait(Ep0TxComplete).ok();
        }
        context.hw.endpoint(0).status_clr.set(clear);
    }
    for dma_endpoint in 1..8 {
        if interrupts.dma(dma_endpoint) != 0 {
            // Reading the status clears the interrupt
            let status = context.hw.dma(dma_endpoint).status.get();
            context.conn.send_scalar_nowait(DmaInterrupt { endpoint: dma_endpoint as u8, status }).ok();
        }
    }
    context.hw.clear_interrupt(interrupts);
}
