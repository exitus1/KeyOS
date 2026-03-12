#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub(crate) enum Register {
    Status = 0x02,
    RegId = 0xA3,
    PowerMode = 0xA5,

    Touch1XH = 0x03,

    MaxXH = 0x98,
    MaxXL = 0x99,
    MaxYH = 0x9A,
    MaxYL = 0x9B,
}

impl From<Register> for u8 {
    fn from(value: Register) -> Self { value as u8 }
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum PowerMode {
    // High frequency detection
    Active = 0x00,
    // Low frequency detection, most algos are off
    Monitor = 0x01,
    // No detection, most electronics are off
    Standby = 0x02,
    // Shut down everything, including MCU and I2C
    Hibernate = 0x03,
}

#[derive(Debug)]
pub struct Status {
    pub num_frames: u8,
    pub num_touch_points: u8,
}

impl From<u8> for Status {
    fn from(value: u8) -> Self { Self { num_frames: value >> 4, num_touch_points: value & 0b1111 } }
}

#[derive(Debug, Default, Copy, Clone)]
#[repr(u8)]
pub enum TouchKind {
    Press = 0,
    Release,
    Drag,

    #[doc(hidden)]
    #[default]
    Reserved,
}

impl From<u8> for TouchKind {
    fn from(value: u8) -> Self {
        match value {
            0b00 => TouchKind::Press,
            0b01 => TouchKind::Release,
            0b10 => TouchKind::Drag,
            _ => TouchKind::Reserved,
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Touch {
    pub kind: TouchKind,
    pub id: u8,
    pub x: u16,
    pub y: u16,
}

impl Touch {
    pub(crate) fn from_parts(xh: u8, xl: u8, yh: u8, yl: u8) -> Self {
        let x = u16::from_be_bytes([xh & 0b1111, xl]);
        let kind = TouchKind::from((xh >> 6) & 0b11);
        let y = u16::from_be_bytes([yh & 0b1111, yl]);
        let id = (yh >> 4) & 0b1111;
        Self { kind, id, x, y }
    }

    pub(crate) fn from_data(data: &[u8]) -> Self { Self::from_parts(data[0], data[1], data[2], data[3]) }

    pub fn is_reserved(&self) -> bool { matches!(self.kind, TouchKind::Reserved) }
}

// Total number of different registers per touch point to skip through
pub const NUM_REGS_PER_TOUCH: u8 = 6;

#[derive(Debug)]
pub struct Dimensions {
    pub x: u16,
    pub y: u16,
}

impl Dimensions {
    pub(crate) fn from_data(xh: u8, xl: u8, yh: u8, yl: u8) -> Self {
        Dimensions { x: u16::from_le_bytes([xh, xl]), y: u16::from_le_bytes([yh, yl]) }
    }

    pub(crate) fn to_data(&self) -> (u8, u8, u8, u8) {
        let [xh, xl] = self.x.to_le_bytes();
        let [yh, yl] = self.y.to_le_bytes();
        (xh, xl, yh, yl)
    }
}
