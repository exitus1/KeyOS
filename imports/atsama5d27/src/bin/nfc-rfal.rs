// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: MIT OR Apache-2.0

#![no_std]
#![no_main]

use {
    atsama5d27::{
        aic::{Aic, InterruptEntry, SourceKind},
        pio::{Direction, Func, Pio, PioC},
        pit::Pit,
        pmc::{PeripheralId, Pmc},
        sfr::Sfr,
        spi::{BitsPerTransfer, ChipSelect, Spi, SpiMode},
        twi::Twi,
        uart::{Uart, Uart1},
    },
    core::{
        arch::global_asm,
        fmt::{Display, Formatter, Write},
        panic::PanicInfo,
        ptr::addr_of_mut,
        sync::atomic::{compiler_fence, AtomicUsize, Ordering::SeqCst},
    },
    drv2605::{Drv2605, Effect},
    is31fl32xx::{Is31fl32xx, OscillatorClock, PwmResolution, SoftwareShutdownMode, IS31FL3205},
};

static TICK_COUNT: AtomicUsize = AtomicUsize::new(0);

global_asm!(include_str!("../start.S"));

type UartType = Uart<Uart1>;

// MCK: 164MHz
// Clock frequency is divided by 2 because of the default `h32mxdiv` PMC setting
const MASTER_CLOCK_SPEED: u32 = 164000000 / 2;

#[no_mangle]
fn _entry() -> ! {
    extern "C" {
        // These symbols come from `link.ld`
        static mut _sbss: u32;
        static mut _ebss: u32;
    }

    // Initialize RAM
    unsafe {
        r0::zero_bss(addr_of_mut!(_sbss), addr_of_mut!(_ebss));
    }

    let mut sfr = Sfr::new();
    sfr.set_l2_cache_sram_enabled(true);

    let mut pmc = Pmc::new();
    pmc.enable_peripheral_clock(PeripheralId::Pit);
    pmc.enable_peripheral_clock(PeripheralId::Aic);
    pmc.enable_peripheral_clock(PeripheralId::Pioa);
    pmc.enable_peripheral_clock(PeripheralId::Piob);
    pmc.enable_peripheral_clock(PeripheralId::Pioc);
    pmc.enable_peripheral_clock(PeripheralId::Piod);
    pmc.enable_peripheral_clock(PeripheralId::Spi0);
    pmc.enable_peripheral_clock(PeripheralId::Twi0);

    let mut aic = Aic::new();
    aic.init();
    aic.set_spurious_handler_fn_ptr(aic_spurious_handler as unsafe extern "C" fn() as usize);

    let pit_irq_ptr = pit_irq_handler as unsafe extern "C" fn() as usize;
    aic.set_interrupt_handler(InterruptEntry {
        peripheral_id: PeripheralId::Pit,
        vector_fn_ptr: pit_irq_ptr,
        kind: SourceKind::LevelSensitive,
        priority: 0,
    });

    // Enable interrupts
    unsafe {
        core::arch::asm!("cpsie if");
    }

    let mut uart = UartType::new();
    uart.set_rx_interrupt(true);
    uart.set_rx(true);
    let mut console = uart;
    writeln!(console, "Starting").ok();

    let mut pit = Pit::new();
    // Every 1 ms
    pit.set_interval(MASTER_CLOCK_SPEED / 1000 / 16);
    pit.reset();
    pit.set_interrupt(true);
    pit.set_enabled(true);

    // SPI pins
    let sck = Pio::pa14();
    sck.set_func(Func::A); // SPI0_SPCK
    let mosi = Pio::pa15();
    mosi.set_func(Func::A); // SPI0_MOSI
    let miso = Pio::pa16();
    miso.set_func(Func::A); // SPI0_MISO
    let cs0 = Pio::pa17();
    cs0.set_func(Func::A); // SPI0_NPCS0

    // NFC pins
    fn get_nfc_irq_in() -> Pio<PioC, 31> {
        Pio::pc31()
    }
    let mut nfc_irq_in = get_nfc_irq_in();
    nfc_irq_in.set_func(Func::Gpio);
    nfc_irq_in.set_direction(Direction::Output);
    nfc_irq_in.set(true);

    fn get_nfc_irq_out() -> Pio<PioC, 29> {
        Pio::pc29()
    }
    let nfc_irq_out = get_nfc_irq_out();
    nfc_irq_out.set_func(Func::Gpio);
    nfc_irq_out.set_direction(Direction::Input);

    // There's a minimal clock speed!
    // No lower than 1.79 MHz (2.0 MHz is the max per datasheet, although it seems to work
    // with 4 MHz too). Speed lower than 1.79 MHz results in tv(SO) timing (80ns) mismatch
    // that leads to the loss of MSB on MOSI line
    const NFC_SPI_CLOCK_SPEED_HZ: u32 = 2_000_000;

    // SPI
    const SPI_CS: ChipSelect = ChipSelect::Cs0;
    let mut spi = Spi::spi0();
    spi.init();
    spi.init_cs(SPI_CS, BitsPerTransfer::Bits8, SpiMode::Mode0, true);
    spi.set_bitrate(MASTER_CLOCK_SPEED, SPI_CS, NFC_SPI_CLOCK_SPEED_HZ);
    spi.master_enable(true);
    spi.set_enabled(true);

    writeln!(console, "Initial status: {:?}", spi.status()).ok();

    let mut rfal = match rfal::Rfal::new(rfal::Platform {
        spi_poll_send: || {
            // Simple implementation - just return true for now
            true
        },
        spi_reset: || {
            // Reset by toggling enable
            let mut spi = Spi::spi0();
            spi.set_enabled(false);
            for _ in 0..1000 {
                armv7::asm::nop();
            }
            spi.set_enabled(true);
        },
        spi_send_cmd: |cmd, data, sod| {
            // Simple implementation using write_8 for each byte
            let mut spi = Spi::spi0();
            let _ = spi.write_8(0x02); // Send command
            let _ = spi.write_8(cmd);
            let _ = spi.write_8(data.len() as u8);
            for &byte in data {
                let _ = spi.write_8(byte);
            }
        },
        spi_read: |code, data| {
            // Simple implementation - return dummy data
            *code = 0x00;
            if data.len() > 0 {
                data[0] = 0x00;
            }
            1
        },
        spi_read_echo: || {
            // Simple implementation - just return true
            true
        },
        spi_flush: || {
            // Simple implementation - do nothing for now
        },
        handle_error: |e, line| {
            let e = e.to_str().expect("to str");
            writeln!(UartType::new(), "[-] RFAL ERROR @ {}:{}", e, line).ok();
        },
        log: |msg, val| {
            let msg = msg.to_str().expect("to str");
            writeln!(UartType::new(), "[*] RFAL {}{}", msg, val).ok();
        },
        irq_in_pulse_low: || {
            get_nfc_irq_in().set(true);
            delay_ms(1); // wait t0 > 100us
            get_nfc_irq_in().set(false);
            delay_ms(1); // wait t1 > 10us
            get_nfc_irq_in().set(true);
            delay_ms(11); // wait t3 > 10ms
        },
        wait_irq_out_falling_edge: |timeout| {
            let start = TICK_COUNT.load(SeqCst);
            let timeout = timeout as usize;
            loop {
                let is_irq_out_low = !get_nfc_irq_out().get();
                if is_irq_out_low || (TICK_COUNT.load(SeqCst) - start) >= timeout {
                    return is_irq_out_low;
                }
                delay_ms(1);
            }
        },
        get_ticks_ms: || TICK_COUNT.load(SeqCst) as u32,
        delay_ms,
    }) {
        Ok(rfal) => rfal,
        Err(e) => {
            panic!("RFAL init failed: {:?}", e)
        }
    };
    writeln!(console, "[+] Initialized RFAL").ok();

    let state = rfal.nfc.state();
    writeln!(console, "[*] State: {:?}", state).ok();

    // Do one clock cycle of SCL to reset all the possibly stuck slaves
    let mut scl = Pio::pc28();
    scl.set_func(Func::Gpio);
    scl.set_direction(Direction::Output);
    for _ in 0..1 {
        scl.set(false);
        for _ in 0..1000 {
            armv7::asm::nop();
        }
        scl.set(true);
    }

    let scl = Pio::pc28();
    scl.set_func(Func::E); // TWI
    let sda = Pio::pc27();
    sda.set_func(Func::E); // TWI
    let twi0 = Twi::twi0();

    writeln!(console, "TWI0: initializing master").ok();
    twi0.init_master(MASTER_CLOCK_SPEED as usize, 100_000);
    writeln!(console, "TWI0 status: {:?}", twi0.status()).ok();

    let mut hfb_en = Pio::pd21();
    hfb_en.set_func(Func::Gpio);
    hfb_en.set_direction(Direction::Output);
    hfb_en.set(true);

    let mut hfb = Drv2605::new(unsafe { twi0.clone() });
    hfb.init_open_loop_erm().expect("init vibration");
    hfb.set_single_effect(Effect::ShortDoubleClickMediumOne100)
        .expect("set effect");
    hfb.set_go(true).expect("set go");

    hfb.set_single_effect(Effect::SharpClick60)
        .expect("set effect");
    hfb.set_go(true).expect("vibration click");

    let mut led_charge_pump_en = Pio::pd23();

    led_charge_pump_en.set_func(Func::Gpio);
    led_charge_pump_en.set_direction(Direction::Output);
    led_charge_pump_en.set(true); // Enable RGB LED 5v charge pump

    let led_shutdown = Pio::pd26();
    led_shutdown.set_func(Func::Gpio);
    led_shutdown.set_direction(Direction::Output);

    // RGB LED driver
    let mut leds =
        Is31fl32xx::<IS31FL3205, _, _>::init_with_i2c(0x34, led_shutdown, unsafe { twi0.clone() });

    let mut pit = Pit::new();
    pit.set_clock_speed(MASTER_CLOCK_SPEED);
    leds.enable_device(
        &mut pit,
        OscillatorClock::SixteenMHz,
        PwmResolution::Eightbit,
        SoftwareShutdownMode::Normal,
    )
    .expect("leds enable");
    // Set 50% LED scaling on all channels
    leds.set_all_led_scaling(0xff).expect("set led scaling");
    // Set max global current
    leds.set_global_current(0xff / 8)
        .expect("set max global current");

    // Set default 0 brightness (turn off all LEDs)
    for ch in 0..12 {
        leds.set(ch, 0).expect("set led dark");
    }

    leds.set(0, 100).expect("set led0");

    let mut disc = rfal::Discover::default();
    // disc.params.compMode = rfal::rfalComplianceMode::RFAL_COMPLIANCE_MODE_ISO;
    disc.params.compMode = rfal::rfalComplianceMode::RFAL_COMPLIANCE_MODE_NFC;
    disc.params.devLimit = 1;
    disc.params.nfcfBR = rfal::rfalBitRate::RFAL_BR_212;
    disc.params.ap2pBR = rfal::rfalBitRate::RFAL_BR_424;
    disc.params.maxBR = rfal::rfalBitRate::RFAL_BR_KEEP;

    /* P2P communication data */
    let nfcid3 = [0x01, 0xFE, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A];
    disc.params.nfcid3.copy_from_slice(&nfcid3);
    let gb = [
        0x46, 0x66, 0x6d, 0x01, 0x01, 0x11, 0x02, 0x02, 0x07, 0x80, 0x03, 0x02, 0x00, 0x03, 0x04,
        0x01, 0x32, 0x07, 0x01, 0x03,
    ];
    disc.params.GB[..gb.len()].copy_from_slice(&gb);
    disc.params.GBLen = gb.len() as u8;
    // disc.params.p2pNfcaPrio = false; // already default

    disc.params.notifyCb = Some(disc_callback);
    disc.params.totalDuration = 1000;
    disc.params.wakeupEnabled = false; // can be toggled by USER BUTTON
    disc.params.wakeupConfigDefault = true;
    // disc.params.wakeupPollBefore = false; // already default
    // disc.params.wakeupNPolls = 1; // already default

    disc.params.techs2Find |= rfal::RFAL_NFC_POLL_TECH_A as u16;
    // disc.params.techs2Bail = rfal::RFAL_NFC_TECH_NONE as u16; // already default

    #[cfg(feature = "nfc-ce")]
    {
        /* Set configuration for NFC-A CE */
        disc.params
            .lmConfigPA
            .SENS_RES
            .copy_from_slice(&[0x44, 0x00]); /* SENS_RES / ATQA */
        disc.params.lmConfigPA.nfcid[..7]
            .copy_from_slice(&[0x02, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66]); /* NFCID / UID (7 bytes) */
        disc.params.lmConfigPA.nfcidLen = rfal::rfalLmNfcidLen::RFAL_LM_NFCID_LEN_07; /* Set NFCID length to 7 bytes */
        disc.params.lmConfigPA.SEL_RES = 0x20; /* SEL_RES / SAK */
        disc.params.techs2Find |= rfal::RFAL_NFC_LISTEN_TECH_A as u16;
    }

    // disc.params.isoDepFS = rfalIsoDepFSxI::RFAL_ISODEP_FSXI_256; // already default
    // disc.params.nfcDepLR = RFAL_NFCDEP_LR_254 as u8; // already default

    disc.start().expect("[!] Couldn't start discovery");

    writeln!(console, "[>] Started discovery").ok();

    loop {
        rfal.nfc.worker();

        let state = rfal.nfc.state();
        if matches!(state, rfal::rfalNfcState::RFAL_NFC_STATE_ACTIVATED) {
            leds.set(4, 100).expect("set led4");
            hfb.set_go(true).expect("vibration click");

            let nfc_dev = match rfal.nfc.active_device() {
                Ok(nfc_dev) => nfc_dev,
                Err(e) => {
                    writeln!(console, "[*] rfalNfcGetActiveDevice error: {:?}", e).ok();
                    continue;
                }
            };

            writeln!(console, "[+] Activated by {:?}", nfc_dev.dev_type()).ok();
            match nfc_dev.dev_type() {
                rfal::rfalNfcDevType::RFAL_NFC_LISTEN_TYPE_NFCA => {
                    leds.set(6, 100).expect("set led6");

                    let nfc_type = nfc_dev.nfca().type_;
                    writeln!(console, "[+] with type {:?}", nfc_type).ok();
                    if let Some(nfc_id) = nfc_dev.id() {
                        let uid = Uid::from_slice(nfc_id);
                        writeln!(console, "[+] UID: {}", uid).ok();
                    }
                    #[cfg(feature = "nfc-ndef")]
                    {
                        if let Err(e) = rfal.ndef.poller.initialize(&nfc_dev) {
                            writeln!(
                                console,
                                "[*] ndefPollerContextInitialization error: {:?}",
                                e
                            )
                            .ok();
                            if let Err(e) = rfal.nfc.deactivate_and_discovery() {
                                writeln!(console, "[*] rfalNfcDeactivate error: {:?}", e).ok();
                            }
                            continue;
                        };
                        match rfal.ndef.poller.ndef_detect() {
                            Err(e) => {
                                writeln!(
                                    console,
                                    "[+] NDEF NOT DETECTED (ndefPollerNdefDetect error: {:?})",
                                    e
                                )
                                .ok();
                            }
                            Ok(ndef_info) => {
                                // Read
                                writeln!(
                                    console,
                                    "[>] ndef_ctx.type: {:?}",
                                    rfal.ndef.poller.ndef_ctx_type()
                                )
                                .ok();
                                writeln!(console, "[+] NDEF detected.").ok();
                                writeln!(console, "[>] ndef_info.state: {:?}", ndef_info.state)
                                    .ok();
                                #[cfg(feature = "nfc-ndef-read")]
                                if ndef_info.state == rfal::ndefState::NDEF_STATE_INITIALIZED {
                                    writeln!(console, "[+] Nothing to read.").ok();
                                } else {
                                    match rfal.ndef.poller.read_raw_message() {
                                        Err(e) => {
                                            writeln!(
                                                console,
                                                "[*] NDEF message cannot be read \
                                                 (ndefPollerReadRawMessage error: {:?})",
                                                e
                                            )
                                            .ok();
                                            if let Err(e) = rfal.nfc.deactivate_and_discovery() {
                                                writeln!(
                                                    console,
                                                    "[*] rfalNfcDeactivate error: {:?}",
                                                    e
                                                )
                                                .ok();
                                            }
                                            continue;
                                        }
                                        Ok(raw_msg) => {
                                            writeln!(
                                                console,
                                                "[>] Read Raw Message: {:x?}.",
                                                raw_msg
                                            )
                                            .ok();
                                            if let Ok(msg) = ndef::Message::try_from(raw_msg) {
                                                writeln!(
                                                    console,
                                                    "[+] Read Message :
                                                    {:?}.",
                                                    msg
                                                )
                                                .ok();
                                            }
                                        }
                                    }
                                }
                                #[cfg(feature = "nfc-ndef-format")]
                                if rfal.ndef.poller.ndef_ctx_type()
                                    == Some(rfal::ndefDeviceType::NDEF_DEV_T2T)
                                {
                                    writeln!(console, "[+] Tag type 2 formatting.").ok();
                                    let ndef_cc = rfal::ndefCapabilityContainer {
                                        t2t: rfal::ndefCapabilityContainerT2T {
                                            magicNumber: 0xE1,
                                            majorVersion: 1,
                                            minorVersion: 0,
                                            size: 109, // 872 bytes
                                            readAccess: 0,
                                            writeAccess: 0,
                                        },
                                    };
                                    if let Err(e) = rfal.ndef.poller.tag_format(ndef_cc, 0) {
                                        writeln!(
                                            console,
                                            "[*] Tag cannot be formatted (ndefPollerTagFormat \
                                             error: {:?})",
                                            e
                                        )
                                        .ok();
                                        if let Err(e) = rfal.nfc.deactivate(
                                            rfal::rfalNfcDeactivateType::RFAL_NFC_DEACTIVATE_DISCOVERY,
                                        ) {
                                            writeln!(console, "[*] rfalNfcDeactivate error: {:?}", e).ok();
                                        }
                                        continue;
                                    }
                                }
                                #[cfg(feature = "nfc-ndef-write")]
                                {
                                    writeln!(console, "[+] Write 1 Text record to the Tag.").ok();
                                    let mut msg = ndef::Message::default();
                                    let mut rec1 = ndef::Record::new(
                                        None,
                                        ndef::Payload::RTD(ndef::RecordType::Text {
                                            enc: "en",
                                            txt: "NDEF Text in Prime !",
                                        }),
                                    );
                                    msg.append_record(&mut rec1).unwrap();
                                    if let Ok(raw_msg) = msg.to_vec() {
                                        writeln!(
                                            console,
                                            "[>] Write Raw Message: {:x?}.",
                                            raw_msg.as_slice()
                                        )
                                        .ok();
                                        if let Err(e) =
                                            rfal.ndef.poller.write_raw_message(raw_msg.as_slice())
                                        {
                                            writeln!(
                                                console,
                                                "[*] ndefPollerWriteRawMessage error: {:?}",
                                                e
                                            )
                                            .ok();
                                        } else {
                                            writeln!(
                                                console,
                                                "[+] Wrote 1 Text record to the Tag success."
                                            )
                                            .ok();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                rfal::rfalNfcDevType::RFAL_NFC_POLL_TYPE_NFCA => {
                    leds.set(9, 100).expect("set led9");
                    #[cfg(feature = "nfc-ce")]
                    {
                        let mut rx_data = [0u8; 100];
                        let mut rcv_len = 0;
                        let mut tx_buf = [0u8; 100];
                        let mut tx_len: u16;
                        let mut res;
                        #[derive(PartialEq)]
                        enum EmuTagState {
                            Idle,
                            AppSelected,
                            CcSelected,
                            FidSelected,
                        }
                        let mut emu_tag_state = EmuTagState::Idle;
                        #[derive(PartialEq)]
                        enum EmuFile {
                            Cc,
                            Ndef,
                        }
                        let mut file_selected = None;
                        /*
                         * CCLEN : Indicates the size of this CC File <BR>
                         * T4T_VNo : Indicates the Mapping Version <BR>
                         * MLe high : Max R-APDU size <BR>
                         * MLc high : Max C-APDU size <BR>
                         * NDEF FCI T: Indicates the NDEF-File_Ctrl_TLV <BR>
                         * NDEF FCI L: The length of the V-field <BR>
                         * NDEF FCI V1: NDEF File Identifier <BR>
                         * NDEF FCI V2: NDEF File size <BR>
                         * NDEF FCI V3: NDEF Read AC <BR>
                         * NDEF FCI V4: NDEF Write AC <BR>
                         */
                        let emu_cc_file = [
                            0x00, 0x0F, /* CCLEN */
                            0x20, /* T4T_VNo */
                            0x00, 0x7F, /* MLe */
                            0x00, 0x7F, /* MLc */
                            0x04, /* T */
                            0x06, /* L */
                            0xE1, 0x04, /* V1 */
                            0xFF, 0xFE, /* V2 */
                            0x00, /* V3 */
                            0x00, /* V4 */
                        ]
                        .as_slice();
                        /*
                         * NDEF length <BR>
                         * NDEF Header: MB,ME,SR,Well known Type <BR>
                         * NDEF type length <BR>
                         * NDEF payload length <BR>
                         * NDEF Type : URI <BR>
                         * NDEF URI abreviation field : http://www. <BR>
                         * NDEF URI string : st.com/st25r <BR>
                         * NDEF URI string : st.com/st25-demo <BR>
                         */
                        let emu_ndef_uri = [
                            0x00, 0x15, /* NDEF length */
                            0xD1, /* NDEF Header */
                            0x01, /* NDEF type length */
                            0x11, /* NDEF payload length */
                            0x55, /* NDEF Type */
                            0x01, /* NDEF URI abreviation field */
                            0x73, 0x74, 0x2E, 0x63, 0x6F, /* NDEF URI string */
                            0x6D, 0x2F, 0x73, 0x74, 0x32, 0x35, 0x2D, 0x64, 0x65, 0x6D, 0x6F,
                        ]
                        .as_slice();
                        loop {
                            rfal.nfc.worker();
                            let state = rfal.nfc.state();
                            writeln!(console, "[>] rfalNfcGetState: {:?}", state).ok();
                            if matches!(state, rfal::rfalNfcState::RFAL_NFC_STATE_ACTIVATED) {
                                res = rfal.nfc.data_exchange.start(
                                    None,
                                    &mut rx_data,
                                    &mut rcv_len,
                                    0,
                                );
                                writeln!(console, "[>] rfalNfcDataExchangeStart error: {:?}", res)
                                    .ok();
                                if res.is_err() {
                                    writeln!(
                                        console,
                                        "[*] rfalNfcDataExchangeStart error: {:?}",
                                        res
                                    )
                                    .ok();
                                } else {
                                    /* ERR_NONE */
                                    loop {
                                        rfal.nfc.worker();
                                        res = rfal.nfc.data_exchange.get_status();
                                        writeln!(
                                            console,
                                            "[>] rfalNfcDataExchangeGetStatus error: {:?}",
                                            res
                                        )
                                        .ok();
                                        if res != Err(rfal::Error::Busy) {
                                            break;
                                        }
                                    }
                                }
                                if res.is_err() && res != Err(rfal::Error::SleepReq) {
                                    break;
                                }
                            } else if matches!(
                                state,
                                rfal::rfalNfcState::RFAL_NFC_STATE_DATAEXCHANGE
                                    | rfal::rfalNfcState::RFAL_NFC_STATE_DATAEXCHANGE_DONE
                            ) {
                                writeln!(
                                    console,
                                    "[>] rx_data: {:?}",
                                    &rx_data[0..rcv_len as usize]
                                )
                                .ok();
                                tx_len = if rcv_len < 4 {
                                    0
                                } else if rx_data[0] == 0x00 {
                                    /* T4T_CLA_00 */
                                    match rx_data[1] {
                                        0xA4 => {
                                            /* T4T_INS_SELECT */
                                            /*
                                             * Cmd: CLA(1) | INS(1) | P1(1) | P2(1) | Lc(1) |
                                             * Data(n) |
                                             * [Le(1)]
                                             * Rsp: [FCI(n)] | SW12
                                             *
                                             * Select App by Name NDEF:       00 A4 04 00 07 D2
                                             * 76 00 00 85
                                             * 01 01 00
                                             * Select App by Name NDEF 4 ST:  00 A4 04 00 07 A0
                                             * 00 00 00 03
                                             * 00 00 00
                                             * Select CC FID:                 00 A4 00 0C 02 xx
                                             * xx
                                             * Select NDEF FID:               00 A4 00 0C 02 xx
                                             * xx
                                             */
                                            //demoCeT4TSelect(rxData, txBuf)
                                            writeln!(console, "[>] T4T_INS_SELECT").ok();
                                            let aid = [0xD2, 0x76, 0x00, 0x00, 0x85, 0x01, 0x01]
                                                .as_slice();
                                            let fid_cc = [0xE1, 0x03].as_slice();
                                            let fid_ndef = [0xE1, 0x04].as_slice();
                                            let select_file_id =
                                                [0xA4, 0x00, 0x0C, 0x02, 0x00, 0x01].as_slice();
                                            if emu_tag_state != EmuTagState::Idle
                                                && rx_data
                                                    .windows(fid_cc.len())
                                                    .any(|w| w == fid_cc)
                                            {
                                                /* Select CC */
                                                writeln!(console, "[>] Select CC").ok();
                                                emu_tag_state = EmuTagState::CcSelected;
                                                file_selected = Some(EmuFile::Cc);
                                                tx_buf[0] = 0x90;
                                                tx_buf[1] = 0x00;
                                            } else if emu_tag_state != EmuTagState::Idle
                                                && (rx_data
                                                    .windows(fid_ndef.len())
                                                    .any(|w| w == fid_ndef)
                                                    || rx_data
                                                        .windows(select_file_id.len())
                                                        .any(|w| w == select_file_id))
                                            {
                                                /* Select NDEF */
                                                writeln!(console, "[>] Select NDEF").ok();
                                                emu_tag_state = EmuTagState::FidSelected;
                                                file_selected = Some(EmuFile::Ndef);
                                                tx_buf[0] = 0x90;
                                                tx_buf[1] = 0x00;
                                            } else if rx_data.windows(aid.len()).any(|w| w == aid) {
                                                /* Select Appli */
                                                writeln!(console, "[>] Select Appli").ok();
                                                emu_tag_state = EmuTagState::AppSelected;
                                                tx_buf[0] = 0x90;
                                                tx_buf[1] = 0x00;
                                            } else {
                                                writeln!(console, "[>] Select OTHER").ok();
                                                emu_tag_state = EmuTagState::Idle;
                                                tx_buf[0] = 0x6A;
                                                tx_buf[1] = 0x82;
                                            }
                                            2
                                        }
                                        0xB0 => {
                                            /* T4T_INS_READ */
                                            /*
                                             * Cmd: CLA(1) | INS(1) | P1(1).. offset inside file
                                             * high |
                                             * P2(1).. offset inside file high | Le(1).. nBytes
                                             * to read
                                             * Rsp: BytesRead | SW12
                                             */
                                            writeln!(console, "[>] T4T_INS_READ").ok();
                                            let offset = ((rx_data[2] as usize) << 8)
                                                | (rx_data[3] as usize);
                                            let mut to_read = rx_data[4] as usize;
                                            writeln!(
                                                console,
                                                "[>] offset: {}, to_read: {}",
                                                offset, to_read
                                            )
                                            .ok();
                                            if file_selected.is_none() {
                                                tx_buf[0] = 0x6A;
                                                tx_buf[1] = 0x82;
                                                2
                                            } else {
                                                if offset + to_read > 0xFFFE {
                                                    /* Max NDEF size emulated */
                                                    to_read = 0xFFFE - offset;
                                                }
                                                match file_selected.as_ref().unwrap() {
                                                    EmuFile::Cc => tx_buf[..to_read]
                                                        .copy_from_slice(
                                                            &emu_cc_file[offset..offset + to_read],
                                                        ),
                                                    EmuFile::Ndef => tx_buf[..to_read]
                                                        .copy_from_slice(
                                                            &emu_ndef_uri[offset..offset + to_read],
                                                        ),
                                                };
                                                tx_buf[to_read] = 0x90;
                                                tx_buf[to_read + 1] = 0x00;
                                                (to_read + 2) as u16
                                            }
                                        }
                                        0xD6 => {
                                            /* T4T_INS_UPDATE */
                                            writeln!(console, "[>] T4T_INS_UPDATE").ok();
                                            let offset = ((rx_data[2] as usize) << 8)
                                                | (rx_data[3] as usize);
                                            let length = rx_data[4] as usize;
                                            writeln!(
                                                console,
                                                "[>] offset: {}, length: {}",
                                                offset, length
                                            )
                                            .ok();
                                            if file_selected != Some(EmuFile::Ndef) {
                                                tx_buf[0] = 0x6A;
                                                tx_buf[1] = 0x82;
                                            } else if offset + length > 0xFFFE {
                                                /* Max NDEF size emulated */
                                                tx_buf[0] = 0x62;
                                                tx_buf[1] = 0x82;
                                            } else {
                                                tx_buf[0] = 0x90;
                                                tx_buf[1] = 0x00;
                                            }
                                            2
                                        }
                                        _ => {
                                            writeln!(console, "[>] OTHER: 0x{:02X}", rx_data[0])
                                                .ok();
                                            tx_buf[0] = 0x68;
                                            tx_buf[1] = 0x00;
                                            2
                                        }
                                    }
                                } else {
                                    tx_buf[0] = 0x68;
                                    tx_buf[1] = 0x00;
                                    2
                                };
                                writeln!(console, "[>] reply : {:x?}", &tx_buf[..tx_len as usize])
                                    .ok();
                                res = rfal.nfc.data_exchange.start(
                                    Some(&mut tx_buf[..tx_len as usize]),
                                    &mut rx_data,
                                    &mut rcv_len,
                                    rfal::RFAL_FWT_NONE,
                                );
                                writeln!(console, "[>] rfalNfcDataExchangeStart error: {:?}", res)
                                    .ok();
                                if res.is_err() {
                                    writeln!(
                                        console,
                                        "[*] rfalNfcDataExchangeStart error: {:?}",
                                        res
                                    )
                                    .ok();
                                } else {
                                    loop {
                                        rfal.nfc.worker();
                                        res = rfal.nfc.data_exchange.get_status();
                                        writeln!(
                                            console,
                                            "[>] rfalNfcDataExchangeGetStatus error: {:?}",
                                            res
                                        )
                                        .ok();
                                        if res != Err(rfal::Error::Busy) {
                                            break;
                                        }
                                    }
                                }
                                if res.is_err() && res != Err(rfal::Error::SleepReq) {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
                _ => {}
            }
            // Finished
            if let Err(e) = rfal.nfc.deactivate_and_discovery() {
                writeln!(console, "[*] rfalNfcDeactivate error: {:?}", e).ok();
            }
            leds.set(4, 0).expect("set led4");
            leds.set(6, 0).expect("set led6");
            leds.set(9, 0).expect("set led9");
        }
    }
}

unsafe extern "C" fn disc_callback(state: rfal::rfalNfcState) {
    if let rfal::rfalNfcState::RFAL_NFC_STATE_WAKEUP_MODE = state {
        writeln!(UartType::new(), "[+] Wake Up mode started").ok();
    } else if let rfal::rfalNfcState::RFAL_NFC_STATE_POLL_TECHDETECT = state {
        writeln!(
            UartType::new(),
            "[+] Wake Up mode terminated. Polling for devices"
        )
        .ok();
    } else if let rfal::rfalNfcState::RFAL_NFC_STATE_POLL_SELECT = state {
        let nfc = rfal::Nfc::default();
        match nfc.get_devices_found() {
            Ok(devs) => {
                writeln!(
                    UartType::new(),
                    "[+] Multiple Tags detected: {}",
                    devs.len()
                )
                .ok();
            }
            Err(e) => {
                writeln!(UartType::new(), "[*] rfalNfcGetDevicesFound error: {:?}", e).ok();
            }
        }
        if let Err(e) = nfc.select(0) {
            writeln!(UartType::new(), "[*] rfalNfcSelect error: {:?}", e).ok();
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Uid {
    Single([u8; 4]),
    Double([u8; 7]),
    Triple([u8; 10]),
}

impl Uid {
    /// If `uid0` is `0x08`, then `uid1` to `uid3` is a random number which is dynamically
    /// generated.
    pub fn is_random(&self) -> bool {
        if let Uid::Single([a, ..]) = self {
            *a == 0x08
        } else {
            false
        }
    }

    pub fn from_slice(bytes: &[u8]) -> Self {
        match bytes.len() {
            4 => Uid::Single(bytes[..4].try_into().unwrap()),
            7 => Uid::Double(bytes[..7].try_into().unwrap()),
            10 => Uid::Triple(bytes[..10].try_into().unwrap()),
            _ => panic!("invalid UID length"),
        }
    }
}

impl Display for Uid {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Uid::Single(u) => write!(f, "{:02x}:{:02x}:{:02x}:{:02x}", u[0], u[1], u[2], u[3]),
            Uid::Double(u) => write!(
                f,
                "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                u[0], u[1], u[2], u[3], u[4], u[5], u[6]
            ),
            Uid::Triple(u) => write!(
                f,
                "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                u[0], u[1], u[2], u[3], u[4], u[5], u[6], u[7], u[8], u[9]
            ),
        }
    }
}

#[no_mangle]
unsafe extern "C" fn pio_irq_handler() {
    // let mut uart = UartType::new();
    let nfc_irq_pin = Pio::pc29();
    if nfc_irq_pin.get_interrupt_status() {
        // writeln!(uart, "NFC IRQ").ok();
    }
}

#[no_mangle]
unsafe extern "C" fn aic_spurious_handler() {
    core::arch::asm!("bkpt");
}

#[no_mangle]
unsafe extern "C" fn pit_irq_handler() {
    let mut pit = Pit::new();
    // Every 1 ms
    pit.reset();
    pit.set_enabled(true);
    TICK_COUNT.fetch_add(1, SeqCst);
}

// FIXME: this doesn't seem to work well with RTT
#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // TODO: disable interrupts

    #[cfg(feature = "rtt")]
    {
        if let Some(mut channel) = unsafe { UpChannel::conjure(0) } {
            channel.set_mode(ChannelMode::BlockIfFull);

            writeln!(channel, "{}", _info).ok();
        }
    }

    let mut console = Uart::<Uart1>::new();

    loop {
        compiler_fence(SeqCst);
        writeln!(console, "{}", _info).ok();
        // Jump to reset handler
        unsafe {
            let pc = 0x20000000;
            core::arch::asm!("mov pc, {}", in(reg) pc);
            unreachable!()
        }
    }
}

fn delay_ms(delay: u32) {
    let goal = TICK_COUNT.load(SeqCst) + delay as usize;
    while TICK_COUNT.load(SeqCst) < goal {
        armv7::asm::wfi();
    }
}
