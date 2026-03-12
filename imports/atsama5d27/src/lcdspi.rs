//! 9-bit LCD-specific bit-banged SPI

use crate::{
    pit::Pit,
    spi::{BitsPerTransfer, ChipSelect, Spi, SpiMode},
};

const SPI_FREQ_HZ: u32 = 10_000_000;

/// LCD configuration implementation. Uses SPI for communication and PIT for generating
/// delays.
pub struct LcdSpi {
    spi: Spi,
    cs: ChipSelect,
    pit: Pit,
    curr_clock_freq: u32,
}

impl LcdSpi {
    /// Create instance
    #[inline]
    pub fn new(mut spi: Spi, cs: ChipSelect, curr_clock_freq: u32, pit: Pit) -> Self {
        spi.init();
        spi.init_cs(cs, BitsPerTransfer::Bits9, SpiMode::Mode0, true);
        spi.set_bitrate(curr_clock_freq, cs, SPI_FREQ_HZ);
        spi.master_enable(true);
        spi.set_enabled(true);

        LcdSpi {
            spi,
            cs,
            curr_clock_freq,
            pit,
        }
    }

    #[inline]
    pub fn send_command(&mut self, cmd: u8) {
        self.send_bits(true, cmd);
    }

    #[inline]
    pub fn send_data(&mut self, dat: u8) {
        self.send_bits(false, dat);
    }

    fn send_bits(&mut self, is_cmd: bool, bits: u8) {
        let data = ((!is_cmd as u16) << 8) | bits as u16;
        self.spi.with_cs(self.cs, |spi| {
            spi.write_16(data).expect("send 9-bit word");
            let _ = spi.read_16().expect("dummy read");
        });
    }

    fn send_sequence(&mut self, stream: &[(u8, &[u8])]) {
        for (cmd, data) in stream {
            self.send_command(*cmd);
            for bits in *data {
                self.send_data(*bits);
            }
        }
    }

    #[inline]
    pub fn run_init_sequence(&mut self) {
        self.send_sequence(&[
            (0xFF, &[0x77, 0x01, 0x00, 0x00, 0x13]),
            (0xEF, &[0x08]),
            (0xFF, &[0x77, 0x01, 0x00, 0x00, 0x10]),
            (0xC0, &[0x63, 0x00]),
            (
                0xC1, // Porch control
                &[
                    0x10, // VBP
                    0x02, // VFP
                ],
            ),
            (0xC2, &[0x01, 0x02]),
            (0xCC, &[0x10]),
            (
                0xB0,
                &[
                    0xC0, 0x0C, 0x92, 0x0C, 0x10, 0x05, 0x02, 0x0D, 0x07, 0x21, 0x04, 0x53, 0x11,
                    0x6A, 0x32, 0x1F,
                ],
            ),
            (
                0xB1,
                &[
                    0xC0, 0x87, 0xCF, 0x0C, 0x10, 0x06, 0x00, 0x03, 0x08, 0x1D, 0x06, 0x54, 0x12,
                    0xE6, 0xEC, 0x0F,
                ],
            ),
            (0xFF, &[0x77, 0x01, 0x00, 0x00, 0x11]),
            (0xB0, &[0x5D]),
            (0xB1, &[0x52]),
            (0xB2, &[0x82]),
            (0xB3, &[0x80]),
            (0xB5, &[0x42]),
            (0xB7, &[0x85]),
            (0xB8, &[0x20]),
            (0xC0, &[0x09]),
            (0xC1, &[0x78]),
            (0xC2, &[0x78]),
            (0xD0, &[0x88]),
            (0xEE, &[0x42]),
        ]);

        self.pit.busy_wait_ms(self.curr_clock_freq, 100);

        self.send_sequence(&[
            (0xE0, &[0x00, 0x00, 0x02]),
            (
                0xE1,
                &[
                    0x04, 0xA0, 0x06, 0xA0, 0x05, 0xA0, 0x07, 0xA0, 0x00, 0x44, 0x44,
                ],
            ),
            (
                0xE2,
                &[
                    0x00, 0x00, 0x33, 0x33, 0x01, 0xA0, 0x00, 0x00, 0x01, 0xA0, 0x00, 0x00,
                ],
            ),
            (0xE3, &[0x00, 0x00, 0x33, 0x33]),
            (0xE4, &[0x44, 0x44]),
            (
                0xE5,
                &[
                    0x0C, 0x30, 0xA0, 0xA0, 0x0E, 0x32, 0xA0, 0xA0, 0x08, 0x2C, 0xA0, 0xA0, 0x0A,
                    0x2E, 0xA0, 0xA0,
                ],
            ),
            (0xE6, &[0x00, 0x00, 0x33, 0x33]),
            (0xE7, &[0x44, 0x44]),
            (
                0xE8,
                &[
                    0x0D, 0x31, 0xA0, 0xA0, 0x0F, 0x33, 0xA0, 0xA0, 0x09, 0x2D, 0xA0, 0xA0, 0x0B,
                    0x2F, 0xA0, 0xA0,
                ],
            ),
            (0xEB, &[0x00, 0x01, 0xE4, 0xE4, 0x44, 0x88, 0x00]),
            (
                0xED,
                &[
                    0xFF, 0xF5, 0x47, 0x6F, 0x0B, 0xA1, 0xA2, 0xBF, 0xFB, 0x2A, 0x1A, 0xB0, 0xF6,
                    0x74, 0x5F, 0xFF,
                ],
            ),
            (0xEF, &[0x08, 0x08, 0x08, 0x40, 0x3F, 0x64]),
            (0xFF, &[0x77, 0x01, 0x00, 0x00, 0x13]),
            (0xE8, &[0x00, 0x0E]),
            (0xFF, &[0x77, 0x01, 0x00, 0x00, 0x00]),
            (0x11, &[]),
        ]);
        self.pit.busy_wait_ms(self.curr_clock_freq, 200);

        self.send_sequence(&[
            (0xFF, &[0x77, 0x01, 0x00, 0x00, 0x13]),
            (0xE8, &[0x00, 0x0C]),
        ]);
        self.pit.busy_wait_ms(self.curr_clock_freq, 10);

        self.send_sequence(&[
            (0xE8, &[0x00, 0x00]),
            (0xFF, &[0x77, 0x01, 0x00, 0x00, 0x00]),
            (
                0x3A, // COLMOD
                &[
                    0x70, // 24 bit
                ],
            ),
            (
                0x36, // MADCTL: Display data access control
                &[
                    0x08, // BGR color order
                ],
            ),
            (
                0x29, // DISPON
                &[],
            ),
        ]);
        self.pit.busy_wait_ms(self.curr_clock_freq, 50);
    }
}
