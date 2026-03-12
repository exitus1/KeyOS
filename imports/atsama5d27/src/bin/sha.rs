// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundationdevices.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

#![no_std]
#![no_main]

use {
    atsama5d27::{
        aic::{Aic, InterruptEntry, SourceKind},
        dma::{DmaChannel, Xdmac, XdmacChannel},
        pit::{Pit, PIV_MAX},
        pmc::{PeripheralId, Pmc},
        sha::{Algo, HashTypeHelper, Sha, Sha224, Sha256, Sha384, Sha512},
        uart::{Uart, Uart1},
    },
    core::{
        arch::global_asm,
        fmt::Write,
        panic::PanicInfo,
        ptr::addr_of_mut,
        sync::atomic::{compiler_fence, Ordering::SeqCst},
    },
};

global_asm!(include_str!("../start.S"));

type UartType = Uart<Uart1>;
const UART_PERIPH_ID: PeripheralId = PeripheralId::Uart1;

// MCK: 164MHz
// Clock frequency is divided by 2 because of the default `h32mxdiv` PMC setting
const MASTER_CLOCK_SPEED: u32 = 164000000 / 2;

// Needs to be mut to be put into .bss and not .rodata
static mut BIG_ZERO: [u8; 64000000] = [0; 64000000];
const BIG_ZERO_HASH: &str = "dbcb3a959f7dba70347a2e6f528f421c67701b8ed5dbed575ff22f6eb4fb94b7";

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

    atsama5d27::l1cache::disable_dcache();

    let mut pmc = Pmc::new();
    pmc.enable_peripheral_clock(PeripheralId::Pit);
    pmc.enable_peripheral_clock(PeripheralId::Aic);
    pmc.enable_peripheral_clock(PeripheralId::Pioa);
    pmc.enable_peripheral_clock(PeripheralId::Piob);
    pmc.enable_peripheral_clock(PeripheralId::Pioc);
    pmc.enable_peripheral_clock(PeripheralId::Piod);
    pmc.enable_peripheral_clock(PeripheralId::Sha);
    pmc.enable_peripheral_clock(PeripheralId::Xdmac0);

    let mut aic = Aic::new();
    aic.init();
    aic.set_spurious_handler_fn_ptr(aic_spurious_handler as unsafe extern "C" fn() as usize);

    let uart_irq_ptr = uart_irq_handler as unsafe extern "C" fn() as usize;
    aic.set_interrupt_handler(InterruptEntry {
        peripheral_id: UART_PERIPH_ID,
        vector_fn_ptr: uart_irq_ptr,
        kind: SourceKind::LevelSensitive,
        priority: 0,
    });

    // Enable interrupts
    unsafe {
        core::arch::asm!("cpsie if");
    }

    // Timer for delays
    let mut pit = Pit::new();
    pit.set_interval(PIV_MAX);
    pit.set_enabled(true);
    pit.set_clock_speed(MASTER_CLOCK_SPEED);
    pit.busy_wait_ms(MASTER_CLOCK_SPEED, 500);

    let mut uart = UartType::new();
    uart.set_rx_interrupt(true);
    uart.set_rx(true);

    sha_small_tests();
    hmac_small_tests();

    loop {
        armv7::asm::wfi();
    }
}

fn sha_small_tests() {
    #[rustfmt::skip]
    const SHA_TESTS: [( &str, &str, &str, &str, &str); 6] = [
        // Empty string
        ("",
            "d14a028c2a3a2bc9476102bb288234c415a2b01f828ea62ac5b3e42f",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da274edebfe76f65fbd51ad2f14898b95b",
            "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"),
        // abc
        ("616263",
            "23097d223405d8228642a477bda255b32aadbce4bda0b3f7e36c9da7",
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            "cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7",
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"),
        // The quick brown fox jumps over the lazy dog
        ("54686520717569636b2062726f776e20666f78206a756d7073206f76657220746865206c617a7920646f67",
            "730e109bd7a8a32b1cb9d9a09aa2325d2430587ddbc0c38bad911525",
            "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592",
            "ca737f1014a48f4c0b6dd43cb177b0afd9e5169367544c494011e3317dbf9a509cb1e5dc1e85a941bbee3d7f2afbc9b1",
            "07e547d9586f6a73f73fbac0435ed76951218fb7d0c8d788a309d785436bbb642e93a252a954f23912547d1e8a3b5ed6e1bfd7097821233fa0538f3db854fee6"),
        // The quick brown fox jumps over the lazy dog.
        ("54686520717569636b2062726f776e20666f78206a756d7073206f76657220746865206c617a7920646f672e",
            "619cba8e8e05826e9b8c519c0a5c68f4fb653e8a3d8aa04bb2c8cd4c",
            "ef537f25c895bfa782526529a9b63d97aa631564d5d789c2b765448c8635fb6c",
            "ed892481d8272ca6df370bf706e4d7bc1b5739fa2177aae6c50e946678718fc67a7af2819a021c2fc34e91bdb63409d7",
            "91ea1245f20d46ae9a037a989f54f1f790f0a47607eeb8a14d12890cea77a1bbc6c7ed9cf205e67b7f2b8fd4c7dfd3a7a8617e45f3c463d481c7e586c39ac1ed"),
        ("cafebabedeadbeef",
            "eaa90aeb8d36523c6eca943ef5a271693ce92708005cf90b476f70e3",
            "ffe50723976ec90e97a3bcdc648ee639384dc8a2515a8b50405422eec7ff0e3e",
            "23b1355112046913507ccf61618945d4f2c791706f12c744ea2028e953dfcbbf3960f543363be04c5045c84b6dc20402",
            "2559250c93432443c41dff9141a05eef23433390663a08c1e65eee138717030928d1670405c1fee7f43f936a21abf9f0a10cf1b2e26f234699928e218656da95"),
        ("ff00ff00ff00ff",
            "738b192d76395c4e84a8fd1a91a9b70dc1428a2d782d98d016d5f221",
            "311268d1c31027d1670ae3eba7b2b90e2c8b900d099346ad140e805fa9288cbf",
            "67d038af43a1fbcfc12a91e6c8e6b1092594400191ad0defd21f4fec797bebac1a77ca35b81c272b74971b767b2d1f09",
            "bcc90d3bacb00ad6ef8a4b7c5f48e7b7c496f1e3f62c15d156742fde26b87591d396739a58f63c8e8a64ba22c12387e795f1f8a066f15630b5ea254999299571"),
    ];

    let mut temp_buf: [u8; 1024] = [0; 1024];

    for (data, expected_224_hash, expected_256_hash, expected_384_hash, expected_512_hash) in
        SHA_TESTS.iter()
    {
        assert_eq!(data.len() % 2, 0, "uneven data length");
        assert_eq!(expected_224_hash.len(), 56, "hash length is not 224 bit");
        assert_eq!(expected_256_hash.len(), 64, "hash length is not 256 bit");
        assert_eq!(expected_384_hash.len(), 96, "hash length is not 384 bit");
        assert_eq!(expected_512_hash.len(), 128, "hash length is not 512 bit");

        temp_buf.fill(0);
        let len = data.len() / 2;
        if len != 0 {
            hex::decode_to_slice(data, &mut temp_buf[..len]).unwrap();
        }
        sha_single_test::<Sha224>(None, "224", "Small", &temp_buf[..len], expected_224_hash);
        sha_single_test::<Sha256>(None, "256", "Small", &temp_buf[..len], expected_256_hash);
        sha_single_test::<Sha384>(None, "384", "Small", &temp_buf[..len], expected_384_hash);
        sha_single_test::<Sha512>(None, "512", "Small", &temp_buf[..len], expected_512_hash);
    }
    sha_single_test::<Sha256>(
        None,
        "256",
        "File without DMA",
        include_bytes!("../../misc/mit.txt"),
        "21785883212d06d2c262b7e9f73f92ae7a03a7f2fe86809d7c7ad9bd6961d265",
    );
    sha_single_test::<Sha256>(
        Some(Xdmac::xdmac0().channel(DmaChannel::Channel0)),
        "256",
        "File with DMA",
        include_bytes!("../../misc/mit.txt"),
        "21785883212d06d2c262b7e9f73f92ae7a03a7f2fe86809d7c7ad9bd6961d265",
    );
    sha_single_test::<Sha256>(
        Some(Xdmac::xdmac0().channel(DmaChannel::Channel0)),
        "256",
        "Zeros with DMA",
        unsafe { &BIG_ZERO },
        BIG_ZERO_HASH,
    );
    sha_single_test::<Sha224>(
        Some(Xdmac::xdmac0().channel(DmaChannel::Channel0)),
        "224",
        "Smol with DMA",
        b"abcd",
        "a76654d8e3550e9a2d67a0eeb6c67b220e5885eddd3fde135806e601",
    );
    sha_single_test::<Sha256>(
        Some(Xdmac::xdmac0().channel(DmaChannel::Channel0)),
        "256",
        "Smol with DMA",
        b"abcd",
        "88d4266fd4e6338d13b845fcf289579d209c897823b9217da3e161936f031589",
    );
    sha_single_test::<Sha384>(
        Some(Xdmac::xdmac0().channel(DmaChannel::Channel0)),
        "384",
        "Smol with DMA",
        b"abcd",
        "1165b3406ff0b52a3d24721f785462ca2276c9f454a116c2b2ba20171a7905ea5a026682eb659c4d5f115c363aa3c79b",
    );
    sha_single_test::<Sha512>(
        Some(Xdmac::xdmac0().channel(DmaChannel::Channel0)),
        "512",
        "Smol with DMA",
        b"abcd",
        "d8022f2060ad6efd297ab73dcc5355c9b214054b0d1776a136a669d26a7d3b14f73aa0d0ebff19ee333368f0164b6419a96da49e3e481753e7e96b716bdccb6f",
    );
}

fn sha_single_test<A: Algo>(
    dma_channel: Option<XdmacChannel>,
    kind: &str,
    name: &str,
    data: &[u8],
    expected_hash: &str,
) {
    let mut uart = UartType::new();

    let sha = Sha::new();

    writeln!(uart, "Hashing {} bytes", data.len()).ok();
    let hash = match dma_channel {
        Some(dma_channel) => {
            let (prefix, aligned_data, postfix) = unsafe { data.align_to::<u32>() };
            if prefix.is_empty() && postfix.is_empty() {
                let sha1_phys_addr = sha.dma_in_address();
                dma_channel.configure_peripheral_transfer(Sha::DMA_CONFIG);
                sha.hash_dma::<A, _>(aligned_data.len() * 4, || -> Result<(), ()> {
                    dma_channel.execute_transfer(
                        aligned_data.as_ptr() as u32,
                        sha1_phys_addr as u32,
                        aligned_data.len(),
                    );
                    Ok(())
                })
                .unwrap()
            } else {
                sha.hash::<A>(data)
            }
        }
        None => sha.hash::<A>(data),
    };

    let mut temp_buf = [0u8; 128];
    temp_buf.fill(0);
    let str_len = A::EMPTY_HASH.as_slice().len() * 2;
    temp_buf.fill(0);
    hex::encode_to_slice(hash.as_slice(), &mut temp_buf[..str_len]).unwrap();
    let hash_str = core::str::from_utf8(&temp_buf[..str_len]).unwrap();

    if hash_str != expected_hash {
        writeln!(uart, "SHA-{kind} {name} test failed").ok();
        writeln!(uart, "Expected: {expected_hash}").ok();
        writeln!(uart, "Result:   {hash_str}\n").ok();
    } else {
        writeln!(uart, "SHA-{kind} {name} test passed\n").ok();
    }
}

fn hmac_small_tests() {
    #[rustfmt::skip]
    const HMAC_TESTS: [(&str,  &str, &str, &str, &str, &str); 6] = [
        // Empty string
        ("",
            "smol_key",
            "d607498d3b60523d5f5be78f8ddc93984d38868bb1a92364aa12b8ca",
            "a07f76d4c7699709244ea35874916b592885ab348c35d8caa9a5082c7f9f69fa",
            "4a83e11c5d83da3c3b7574bf5daa01fbfd138d16470304610257510678658fcdc8291cf014d9fe7bc7bf89d3db0943f8",
            "531487fe1ea4312aa54fb26e515aefc7091d417b67c76b353e07eef85e4f019e53ad611db4ce11f4c2ec73646202f5c1d3412205eff606ffeca56b2695f98232"),
        // abc
        ("616263",
            "longer_key_of_64_bytes_which_correspond_to_sha256_block_size_lol",
            "ef114bf737aa000ed639ce4149e23e15973748d105f0b1e23ec5be0e",
            "ce60c979dd559f3baeaeab010b81d065034712baa685c16dbd7f493c015a5214",
            "6380d4c66b16418363ed67c1df605019815dd249c9df2024130b2e40e25a35b588fe49f583c4f8a3ddf0fa39d613d323",
            "ca905188ce0d194c6503cdf00bd9e415afc1386b2f7a5a5f7f6d3daa27cd3f5608e7adefcd3a5db8af68fa91e1ce8079468225a007f9c4646520e86aa742ccf8"),
        // The quick brown fox jumps over the lazy dog
        ("54686520717569636b2062726f776e20666f78206a756d7073206f76657220746865206c617a7920646f67",
            "smol_key",
            "201cb421f6de0357fac8631984d04e5e8665f0c1868e168da301cc40",
            "e9023ffb47ce3109edd5a37cf2cf9c8540dcf205830d80df3e19ce70f5b99660",
            "531aa960976c401fabf3354af3958b15d545cbdaa4091dd0b49b24557e76a67fcc9d9d3b809d67ce3019aeee0e8bb0d7",
            "c5c018310349a629ff28842881e13dc9b76462cdd97a5308099153c949b677be855aa9f8eb4d8efd5a9dc1302c6be12ac442759122744de17103437657a9acad"),
        // The quick brown fox jumps over the lazy dog.
        ("54686520717569636b2062726f776e20666f78206a756d7073206f76657220746865206c617a7920646f672e",
            "smol_key",
            "eee26d021e116a138f08d92f2ca13f9b59d8c5628f177959a2159eaf",
            "5f956e67fb5c66c7b5f3705b6cfc36e5c870a5c2cdab25f339cf53e9b148d04b",
            "7496712523172a4d9f0f89669f23fb9f5a0f9e42ba6050f5b2223b108c63c00df3dd3a74fc8e5db2d2553180b31bad2d",
            "d0ce47d4ecdbae7ac80b6a05af3f7a35ca8375b5dcc7d44fec509499e9d9222ad74fcd5a98572f995c4529e1f0bfb63a9de93010936da984fb44ea680b44df29"),
        ("cafebabedeadbeef",
            "smol_key",
            "c438cd2c76b3f5b848da1f59230992f80156223116a8739217c1746c",
            "c26fd76ee7077d294352f82933eb2ecd4d0a1955b7511ea33d953eb63bb1b241",
            "d8acab8dc3dd18640ddffb062c5af425e362a37776428991d12ad4ee9d33ec0c4d0f0cc56441666fb57b85b6993154f3",
            "2f99576c560989530f6dcc36bd78d161882a76589385ff13bdf762e7a5b3c4da712854975a46e7543caf61ba551b6fb36f6f6e8e29d2b8d0cb56375b0e841ad9"),
        ("ff00ff00ff00ff",
            "very_long_key_of_65_bytes_in_order_to_trigger_the_key_hashing_lol",
            "53c7c236717e5dc6e3fe92c3e425f1ce2c85fd7476bf3160e877826a",
            "3aa0b4c47a605d89d690b8d1c68a8933aee9887985aea92641f0ef003f642743",
            "bc6c09062f501f469434823f273a2c45d5f5e40add27d7ed53adfca128074d8798c6c5f1f72cb74ef21d3fd470e1b70e",
            "449cae78980b2428f387171521bca8ee9e92f1a897d29a3b4ab41594d90213c0babf5aee65d4fb3a3becd9853371071cea20b3c2623a87f545765edadaf0783a"),
    ];

    let mut temp_buf: [u8; 1024] = [0; 1024];

    for (data, key, expected_224_hash, expected_256_hash, expected_384_hash, expected_512_hash) in
        HMAC_TESTS.iter()
    {
        assert_eq!(data.len() % 2, 0, "uneven data length");
        assert_eq!(expected_224_hash.len(), 56, "hash length is not 224 bit");
        assert_eq!(expected_256_hash.len(), 64, "hash length is not 256 bit");
        assert_eq!(expected_384_hash.len(), 96, "hash length is not 384 bit");
        assert_eq!(expected_512_hash.len(), 128, "hash length is not 512 bit");

        temp_buf.fill(0);
        let len = data.len() / 2;
        if len != 0 {
            hex::decode_to_slice(data, &mut temp_buf[..len]).unwrap();
        }
        hmac224_single_test(
            "Small",
            &key.as_bytes(),
            &temp_buf[..len],
            expected_224_hash,
        );
        hmac256_single_test(
            "Small",
            &key.as_bytes(),
            &temp_buf[..len],
            expected_256_hash,
        );
        hmac384_single_test(
            "Small",
            &key.as_bytes(),
            &temp_buf[..len],
            expected_384_hash,
        );
        hmac512_single_test(
            "Small",
            &key.as_bytes(),
            &temp_buf[..len],
            expected_512_hash,
        );
    }
}

fn hmac224_single_test(name: &str, key: &[u8], data: &[u8], expected_hash: &str) {
    let mut uart = UartType::new();

    let sha = Sha::new();

    writeln!(uart, "HMAC-224 {} bytes", data.len()).ok();
    let hash = sha.hmac::<Sha224>(key, data);

    let mut temp_buf: [u8; 56] = [0; 56];
    temp_buf.fill(0);
    hex::encode_to_slice(hash.as_slice(), &mut temp_buf[..56]).unwrap();
    let hash_str = core::str::from_utf8(&temp_buf[..56]).unwrap();

    if hash_str != expected_hash {
        writeln!(uart, "HMAC-224 {name} test failed").ok();
        writeln!(uart, "Expected: {expected_hash}").ok();
        writeln!(uart, "Result:   {hash_str}\n").ok();
    } else {
        writeln!(uart, "HMAC-224 {name} test passed\n").ok();
    }
}

fn hmac256_single_test(name: &str, key: &[u8], data: &[u8], expected_hash: &str) {
    let mut uart = UartType::new();

    let sha = Sha::new();

    writeln!(uart, "HMAC-256 {} bytes", data.len()).ok();
    let hash = sha.hmac::<Sha256>(key, data);

    let mut temp_buf: [u8; 64] = [0; 64];
    temp_buf.fill(0);
    hex::encode_to_slice(hash.as_slice(), &mut temp_buf[..64]).unwrap();
    let hash_str = core::str::from_utf8(&temp_buf[..64]).unwrap();

    if hash_str != expected_hash {
        writeln!(uart, "HMAC-256 {name} test failed").ok();
        writeln!(uart, "Expected: {expected_hash}").ok();
        writeln!(uart, "Result:   {hash_str}\n").ok();
    } else {
        writeln!(uart, "HMAC-256 {name} test passed\n").ok();
    }
}

fn hmac384_single_test(name: &str, key: &[u8], data: &[u8], expected_hash: &str) {
    let mut uart = UartType::new();

    let sha = Sha::new();

    writeln!(uart, "HMAC-384 {} bytes", data.len()).ok();
    let hash = sha.hmac::<Sha384>(key, data);

    let mut temp_buf: [u8; 96] = [0; 96];
    temp_buf.fill(0);
    hex::encode_to_slice(hash.as_slice(), &mut temp_buf[..96]).unwrap();
    let hash_str = core::str::from_utf8(&temp_buf[..96]).unwrap();

    if hash_str != expected_hash {
        writeln!(uart, "HMAC-384 {name} test failed").ok();
        writeln!(uart, "Expected: {expected_hash}").ok();
        writeln!(uart, "Result:   {hash_str}\n").ok();
    } else {
        writeln!(uart, "HMAC-384 {name} test passed\n").ok();
    }
}

fn hmac512_single_test(name: &str, key: &[u8], data: &[u8], expected_hash: &str) {
    let mut uart = UartType::new();

    let sha = Sha::new();

    writeln!(uart, "HMAC-512 {} bytes", data.len()).ok();
    let hash = sha.hmac::<Sha512>(key, data);

    let mut temp_buf: [u8; 128] = [0; 128];
    temp_buf.fill(0);
    hex::encode_to_slice(hash.as_slice(), &mut temp_buf[..128]).unwrap();
    let hash_str = core::str::from_utf8(&temp_buf[..128]).unwrap();

    if hash_str != expected_hash {
        writeln!(uart, "HMAC-512 {name} test failed").ok();
        writeln!(uart, "Expected: {expected_hash}").ok();
        writeln!(uart, "Result:   {hash_str}\n").ok();
    } else {
        writeln!(uart, "HMAC-512 {name} test passed\n").ok();
    }
}

#[no_mangle]
unsafe extern "C" fn aic_spurious_handler() {
    core::arch::asm!("bkpt");
}

#[no_mangle]
unsafe extern "C" fn uart_irq_handler() {
    let mut uart = UartType::new();
    let char = uart.getc() as char;
    writeln!(uart, "Received character: {}", char).ok();
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut console = Uart::<Uart1>::new();

    compiler_fence(SeqCst);
    writeln!(console, "{}", _info).ok();

    loop {
        unsafe {
            core::arch::asm!("bkpt");
        }
    }
}
