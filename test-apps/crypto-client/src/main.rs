// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: MIT OR Apache-2.0

use crypto::{Direction, AES_BLOCK_SIZE};
use hmac::{Hmac, Mac};
use server::xous::{map_memory, MemoryFlags};
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};

crypto::use_api!();
fs::use_api!();

/// The cosign2 header extracted from a signed 128MB file of zeroes
pub const COSIGN2_HEADER_128MB: &[u8; 2048] = include_bytes!("../cosign2_header.bin");

fn main() {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    log::info!("AES client pid: {}", server::xous::current_pid().unwrap());

    // Allocate 4 megabytes
    let mut data = map_memory(None, None, 0x400000, MemoryFlags::W | MemoryFlags::POPULATE).unwrap();

    let modes = &[
        ("CBC", crypto::AesMode::Cbc { key: &[b'x'; 32], iv: &[b'z'; 16] }),
        ("ECB", crypto::AesMode::Ecb { key: &[b'x'; 32] }),
    ];

    let crypto = CryptoApi::default();

    for (mode_str, mode) in modes {
        data.as_slice_mut().fill(b'a');
        data.as_slice_mut()[0] = b'b';
        data.as_slice_mut()[1] = b'c';
        data.as_slice_mut()[2] = b'd';

        log::info!("Testing {mode_str} AES mode...");

        let aes_ctx = crypto.setup_aes(mode.clone()).unwrap();
        log::info!("Testing with small data");
        log::info!("original data: {:02x?}", &data.as_slice::<u8>()[..256]);
        aes_ctx.execute(data, 0, 15, Direction::Encrypt).unwrap();
        log::info!("encrypted: {:02x?}", &data.as_slice::<u8>()[..256]);
        aes_ctx.execute(data, 0, 15, Direction::Decrypt).unwrap();
        log::info!("decrypted: {:02x?}", &data.as_slice::<u8>()[..256]);

        log::info!("Testing with big data");
        log::info!("original data: {:02x?}", &data.as_slice::<u8>()[data.len() - 32..]);
        aes_ctx.execute(data, 0, data.len() / AES_BLOCK_SIZE, Direction::Encrypt).unwrap();
        log::info!("encrypted: {:02x?}", &data.as_slice::<u8>()[data.len() - 32..]);
        aes_ctx.execute(data, 0, data.len() / AES_BLOCK_SIZE, Direction::Decrypt).unwrap();
        log::info!("decrypted: {:02x?}", &data.as_slice::<u8>()[data.len() - 32..]);

        log::info!("Proper load testing (should take around 5 seconds)");
        let start = std::time::Instant::now();
        for _ in 0..32 {
            aes_ctx.execute(data, 0, data.len() / AES_BLOCK_SIZE, Direction::Encrypt).unwrap();
        }
        let elapsed = start.elapsed().as_millis();
        let mbps = 32.0 * data.len() as f64 / (1024.0 * 1024.0) / (elapsed as f64 / 1000.0);
        log::info!("AES-{mode_str} encrypt speed: {mbps:.2} MB/s");

        const CHUNK_SIZE: usize = 32 * 1024;
        log::info!("Load testing with {CHUNK_SIZE} byte chunks");
        let start = std::time::Instant::now();
        for _ in 0..16 {
            for offset in (0..data.len()).step_by(CHUNK_SIZE) {
                aes_ctx
                    .execute(
                        data.subrange(offset, CHUNK_SIZE).unwrap(),
                        0,
                        CHUNK_SIZE / AES_BLOCK_SIZE,
                        Direction::Encrypt,
                    )
                    .unwrap();
            }
        }
        let elapsed = start.elapsed().as_millis();
        let mbps = 16.0 * data.len() as f64 / (1024.0 * 1024.0) / (elapsed as f64 / 1000.0);
        log::info!("AES-{mode_str} encrypt speed (chunked): {mbps:.2} MB/s");
    }

    log::info!("Testing XTS mode j correctness");
    data.as_slice_mut().fill(b'a');
    let tweak = [b'z'; 16];

    unsafe {
        crypto
            .disk_encrypt_unsafe(
                tweak,
                200,
                data.subrange(0, 16 * 16).unwrap(),
                data.subrange(0, 16 * 16).unwrap(),
                Direction::Encrypt,
            )
            .unwrap();
        log::info!("encrypted: {:02x?}", &data.as_slice::<u8>()[..256]);
        crypto
            .disk_encrypt_unsafe(
                tweak,
                200,
                data.subrange(0, 2 * 16).unwrap(),
                data.subrange(0, 2 * 16).unwrap(),
                Direction::Decrypt,
            )
            .unwrap();
        crypto
            .disk_encrypt_unsafe(
                tweak,
                202,
                data.subrange(2 * 16, 14 * 16).unwrap(),
                data.subrange(2 * 16, 14 * 16).unwrap(),
                Direction::Decrypt,
            )
            .unwrap();
    }
    log::info!("decrypted: {:02x?}", &data.as_slice::<u8>()[..256]);

    log::info!("All AES tests done");
    log::info!("Hashing correctness testing (SHA224, SHA256, SHA384, SHA512)");

    for (i, d) in data.as_slice_mut()[..0x4000].iter_mut().enumerate() {
        *d = ((i % 123) * (i % 57)) as u8;
    }

    for offset in [0, 0x40, 0x1000 - 0x40, 0x1000, 0x1040] {
        for len in [4, 5, 16, 17, 0x785, 0x1000 - 0x40, 0x1000, 0x1001, 0x14f6] {
            let known_good: [u8; 28] = Sha224::digest(&data.as_slice()[offset..offset + len]).into();
            let service_result = crypto.sha224(data.subrange(0, 0x4000).unwrap(), offset, len).unwrap();
            assert_eq!(known_good, service_result, "Wrong result for SHA224 @ offset={offset} len={len}");
            let known_good: [u8; 32] = Sha256::digest(&data.as_slice()[offset..offset + len]).into();
            let service_result = crypto.sha256(data.subrange(0, 0x4000).unwrap(), offset, len).unwrap();
            assert_eq!(known_good, service_result, "Wrong result for SHA256 @ offset={offset} len={len}");
            let known_good: [u8; 48] = Sha384::digest(&data.as_slice()[offset..offset + len]).into();
            let service_result = crypto.sha384(data.subrange(0, 0x4000).unwrap(), offset, len).unwrap();
            assert_eq!(known_good, service_result, "Wrong result for SHA384 @ offset={offset} len={len}");
            let known_good: [u8; 64] = Sha512::digest(&data.as_slice()[offset..offset + len]).into();
            let service_result = crypto.sha512(data.subrange(0, 0x4000).unwrap(), offset, len).unwrap();
            assert_eq!(known_good, service_result, "Wrong result for SHA512 @ offset={offset} len={len}");
        }
    }

    log::info!("Hashing load testing (SHA256)");
    let start = std::time::Instant::now();
    data.as_slice_mut().fill(0);
    for _ in 0..32 {
        crypto.sha256(data, 0, data.len()).unwrap();
    }
    let elapsed = start.elapsed().as_millis();
    log::info!("Hash: {:02x?}", crypto.sha256(data, 0, data.len()).unwrap());
    let mbps = 32.0 * data.len() as f64 / (1024.0 * 1024.0) / (elapsed as f64 / 1000.0);
    log::info!("SHA256 speed: {mbps:.2} MB/s");

    log::info!("Hashing non-contiguous buffer");
    let mut non_contiguous_buffer = map_memory(None, None, 0x4000, MemoryFlags::W).unwrap();

    // We on-demand map these out of order to explicitly break contiguity
    non_contiguous_buffer.as_slice_mut::<u8>()[0x2000] = 1;
    non_contiguous_buffer.as_slice_mut::<u8>()[0x1000] = 2;
    non_contiguous_buffer.as_slice_mut::<u8>()[0x3000] = 3;
    non_contiguous_buffer.as_slice_mut::<u8>()[0x0000] = 0;

    for (i, c) in non_contiguous_buffer.as_slice_mut::<usize>().iter_mut().enumerate() {
        *c = i;
    }
    let known_good: [u8; 32] = Sha256::digest(&non_contiguous_buffer.as_slice()).into();
    let service_result = crypto.sha256(non_contiguous_buffer, 0, non_contiguous_buffer.len()).unwrap();
    assert_eq!(known_good, service_result, "Wrong result for non-contiguous SHA256");

    log::info!("All SHA tests done");

    log::info!("Hashing correctness testing (HMAC-224, HMAC-256, HMAC-384, HMAC-512)");
    for key in [[b'k'; 0].to_vec(), [b'k'; 32].to_vec(), [b'k'; 48].to_vec(), [b'k'; 64].to_vec()] {
        for msg in [[b'm'; 0].to_vec(), [b'm'; 32].to_vec(), [b'm'; 48].to_vec(), [b'm'; 64].to_vec()] {
            type HmacSha224 = Hmac<Sha224>;
            let mut mac = HmacSha224::new_from_slice(&key).unwrap();
            mac.update(&msg);
            let known_good = mac.finalize().into_bytes().to_vec();
            let key_len = key.len();
            let msg_len = msg.len();
            let service_result = crypto.hmac224(key.clone(), msg.clone()).unwrap();
            assert_eq!(
                known_good, service_result,
                "Wrong result for HMAC-224 key_len={key_len} msg_len={msg_len}"
            );
            type HmacSha256 = Hmac<Sha256>;
            let mut mac = HmacSha256::new_from_slice(&key).unwrap();
            mac.update(&msg);
            let known_good = mac.finalize().into_bytes().to_vec();
            let key_len = key.len();
            let msg_len = msg.len();
            let service_result = crypto.hmac256(key.clone(), msg.clone()).unwrap();
            assert_eq!(
                known_good, service_result,
                "Wrong result for HMAC-256 key_len={key_len} msg_len={msg_len}"
            );
            type HmacSha384 = Hmac<Sha384>;
            let mut mac = HmacSha384::new_from_slice(&key).unwrap();
            mac.update(&msg);
            let known_good = mac.finalize().into_bytes().to_vec();
            let key_len = key.len();
            let msg_len = msg.len();
            let service_result = crypto.hmac384(key.clone(), msg.clone()).unwrap();
            assert_eq!(
                known_good, service_result,
                "Wrong result for HMAC-384 key_len={key_len} msg_len={msg_len}"
            );
            type HmacSha512 = Hmac<Sha512>;
            let mut mac = HmacSha512::new_from_slice(&key).unwrap();
            mac.update(&msg);
            let known_good = mac.finalize().into_bytes().to_vec();
            let key_len = key.len();
            let msg_len = msg.len();
            let service_result = crypto.hmac512(key.clone(), msg).unwrap();
            assert_eq!(
                known_good, service_result,
                "Wrong result for HMAC-512 key_len={key_len} msg_len={msg_len}"
            );
        }
    }
    log::info!("All HMAC tests done");

    // Multi-context streaming SHA tests
    test_streaming_sha_multi_context(&crypto, &mut data);

    log::info!("cosign2 fw file verification test");
    let fs = FileSystem::default();

    let start = std::time::Instant::now();
    fw_utils::hash::verify_cosign2(&fs, &crypto, "/keyos/app.bin", fs::Location::System, |_| (), false)
        .expect("verify cosign2");
    let elapsed = start.elapsed().as_millis();
    let mbps = fs.metadata("/keyos/app.bin", fs::Location::System).unwrap().size as f64
        / (1024.0 * 1024.0)
        / (elapsed as f64 / 1000.0);
    log::info!("app.bin cosign verification speed: {mbps:.2} MB/s");

    log::info!("Verifying an enormous file (128 MB of zeroes + header)");
    {
        use std::io::Write;

        // The header was signed for 128MB of zeroes as the binary content
        // Total file = header (2048 bytes) + 128MB of zeroes
        const BINARY_SIZE: usize = 128 * 1024 * 1024; // 128 MB of zeroes
        const TOTAL_FILE_SIZE: usize = cosign2::Header::DEFAULT_SIZE + BINARY_SIZE;
        const TEMP_FILE_PATH: &str = "/big_test_file.bin";

        // Create a big temp file using pre-generated cosign2 header for 128MB of zeroes
        let mut temp_file = fs
            .open_file(
                TEMP_FILE_PATH,
                fs::Location::System,
                fs::OpenFlags { read: true, write: true, create: true },
            )
            .expect("create temp file");

        // Write the pre-generated cosign2 header (signed for 128MB of zeroes)
        temp_file.write_all(COSIGN2_HEADER_128MB).expect("write header");

        // Write 128MB of zeros (the binary content that was signed)
        let zeros = vec![0u8; 64 * 1024]; // 64KB chunks
        let mut written = 0;
        while written < BINARY_SIZE {
            let chunk_size = (BINARY_SIZE - written).min(zeros.len());
            temp_file.write_all(&zeros[..chunk_size]).expect("write zeros");
            written += chunk_size;
        }
        temp_file.flush().expect("flush temp file");
        drop(temp_file);

        log::info!(
            "Created test file: {} byte header + {}MB of zeroes = {}MB total",
            cosign2::Header::DEFAULT_SIZE,
            BINARY_SIZE / (1024 * 1024),
            TOTAL_FILE_SIZE / (1024 * 1024)
        );

        // Now try to verify it - this file has a valid signature for 128MB of zeroes
        let start = std::time::Instant::now();
        let result = fw_utils::hash::verify_cosign2(
            &fs,
            &crypto,
            TEMP_FILE_PATH,
            fs::Location::System,
            |progress| {
                if (progress * 100.0) as u32 % 10 == 0 {
                    log::debug!("Verification progress: {:.0}%", progress * 100.0);
                }
            },
            false,
        );
        let elapsed = start.elapsed().as_millis();

        match result {
            Ok(_) => log::info!("Big file verification succeeded"),
            Err(e) => log::info!("Big file verification failed: {:?}", e),
        }

        let mbps = TOTAL_FILE_SIZE as f64 / (1024.0 * 1024.0) / (elapsed as f64 / 1000.0);
        log::info!("Big file verification speed: {mbps:.2} MB/s (elapsed: {}ms)", elapsed);

        // Clean up
        fs.remove(TEMP_FILE_PATH, fs::Location::System).ok();
    }

    log::info!("All tests done");
}

/// Test streaming SHA multi-context handling
/// Verifies:
/// 1. Multiple contexts can be created (up to 4 per process)
/// 2. Creating a 5th context fails with TooManyShaContexts
/// 3. Multiple threads can use their contexts concurrently
/// 4. Results are correct when contexts are used interleaved
fn test_streaming_sha_multi_context(crypto: &CryptoApi, data: &mut server::xous::MemoryRange) {
    use std::sync::{Arc, Barrier};
    use std::thread;

    use crypto::error::CryptoError;

    log::info!("Testing streaming SHA multi-context handling");

    // Prepare test data
    const CHUNK_SIZE: usize = 0x1000; // 4KB chunks, aligned to SHA_DMA_ALIGNMENT (64)
    const TOTAL_SIZE: usize = CHUNK_SIZE * 4; // 16KB total per context

    for (i, d) in data.as_slice_mut()[..TOTAL_SIZE * 4].iter_mut().enumerate() {
        *d = ((i * 17 + 31) % 256) as u8;
    }

    log::info!("test 1: 4 streaming SHA contexts");
    {
        let ctx1 = crypto.sha256_init(TOTAL_SIZE).expect("Failed to create context 1");
        let ctx2 = crypto.sha256_init(TOTAL_SIZE).expect("Failed to create context 2");
        let ctx3 = crypto.sha256_init(TOTAL_SIZE).expect("Failed to create context 3");
        let ctx4 = crypto.sha256_init(TOTAL_SIZE).expect("Failed to create context 4");

        log::info!("test 2: verifying 5th context creation fails");
        let result = crypto.sha256_init(TOTAL_SIZE);
        match result {
            Err(CryptoError::TooManyShaContexts) => {
                log::info!("  5th context correctly rejected with TooManyShaContexts");
            }
            Ok(_) => {
                panic!("5th context should have failed with TooManyShaContexts");
            }
            Err(e) => {
                panic!("5th context failed with unexpected error: {:?}", e);
            }
        }

        log::info!("test 3: interleaved contexts");
        for chunk_idx in 0..4 {
            let offset = chunk_idx * CHUNK_SIZE;
            ctx1.update(data.subrange(0, TOTAL_SIZE).unwrap(), offset, CHUNK_SIZE)
                .expect("ctx1 update failed");
            ctx2.update(data.subrange(TOTAL_SIZE, TOTAL_SIZE).unwrap(), offset, CHUNK_SIZE)
                .expect("ctx2 update failed");
            ctx3.update(data.subrange(TOTAL_SIZE * 2, TOTAL_SIZE).unwrap(), offset, CHUNK_SIZE)
                .expect("ctx3 update failed");
            ctx4.update(data.subrange(TOTAL_SIZE * 3, TOTAL_SIZE).unwrap(), offset, CHUNK_SIZE)
                .expect("ctx4 update failed");
        }

        let hash1 = ctx1.finalize().expect("ctx1 finalize failed");
        let hash2 = ctx2.finalize().expect("ctx2 finalize failed");
        let hash3 = ctx3.finalize().expect("ctx3 finalize failed");
        let hash4 = ctx4.finalize().expect("ctx4 finalize failed");

        let expected1: [u8; 32] = Sha256::digest(&data.as_slice()[0..TOTAL_SIZE]).into();
        let expected2: [u8; 32] = Sha256::digest(&data.as_slice()[TOTAL_SIZE..TOTAL_SIZE * 2]).into();
        let expected3: [u8; 32] = Sha256::digest(&data.as_slice()[TOTAL_SIZE * 2..TOTAL_SIZE * 3]).into();
        let expected4: [u8; 32] = Sha256::digest(&data.as_slice()[TOTAL_SIZE * 3..TOTAL_SIZE * 4]).into();

        assert_eq!(hash1.as_slice(), expected1.as_slice(), "context 1 hash mismatch");
        assert_eq!(hash2.as_slice(), expected2.as_slice(), "context 2 hash mismatch");
        assert_eq!(hash3.as_slice(), expected3.as_slice(), "context 3 hash mismatch");
        assert_eq!(hash4.as_slice(), expected4.as_slice(), "context 4 hash mismatch");
        log::info!("  interleaved context usage: all hashes match");
    }

    log::info!("test 4: verifying contexts can be reused after drop");
    {
        let ctx = crypto.sha256_init(CHUNK_SIZE).expect("Failed to create context after drop");
        ctx.update(data.subrange(0, CHUNK_SIZE).unwrap(), 0, CHUNK_SIZE).expect("update failed");
        let hash = ctx.finalize().expect("finalize failed");
        let expected: [u8; 32] = Sha256::digest(&data.as_slice()[0..CHUNK_SIZE]).into();
        assert_eq!(hash.as_slice(), expected.as_slice(), "reused context hash mismatch");
        log::info!("  context reuse after drop: OK");
    }

    log::info!("test 4b: verifying context drop cleanup prevents TooManyShaContexts leak");
    {
        // This test verifies that dropping contexts without finalizing them
        // properly cleans up server-side state, preventing context ID exhaustion.
        // Without proper Drop cleanup, this would fail with TooManyShaContexts
        // after 4 iterations since the limit is 4 contexts per process.

        for iteration in 0..8 {
            // Create 4 contexts and drop them without finalizing
            {
                let _ctx1 = crypto.sha256_init(CHUNK_SIZE).expect("Failed to create context 1");
                let _ctx2 = crypto.sha256_init(CHUNK_SIZE).expect("Failed to create context 2");
                let _ctx3 = crypto.sha256_init(CHUNK_SIZE).expect("Failed to create context 3");
                let _ctx4 = crypto.sha256_init(CHUNK_SIZE).expect("Failed to create context 4");
                // All 4 contexts are dropped here without being finalized
            }
            log::debug!("  iteration {}: dropped 4 contexts without finalizing", iteration);
        }

        // After 8 iterations of creating and dropping 4 contexts each (32 total),
        // we should still be able to create new contexts if Drop cleanup works
        let ctx = crypto
            .sha256_init(CHUNK_SIZE)
            .expect("Failed to create context after multiple drop cycles - Drop cleanup may not be working");
        ctx.update(data.subrange(0, CHUNK_SIZE).unwrap(), 0, CHUNK_SIZE).expect("update failed");
        let hash = ctx.finalize().expect("finalize failed");
        let expected: [u8; 32] = Sha256::digest(&data.as_slice()[0..CHUNK_SIZE]).into();
        assert_eq!(hash.as_slice(), expected.as_slice(), "hash mismatch after drop cycles");

        log::info!("  context drop cleanup: OK (32 contexts dropped without leak)");
    }

    log::info!("test 4c: verifying partial update followed by drop cleanup");
    {
        // Test that contexts with partial updates are properly cleaned up on drop
        for _ in 0..4 {
            let ctx = crypto.sha256_init(TOTAL_SIZE).expect("Failed to create context");
            // Only do a partial update (not all data)
            ctx.update(data.subrange(0, CHUNK_SIZE).unwrap(), 0, CHUNK_SIZE).expect("update failed");
            // Drop without finalizing - this simulates an error path where
            // we bail out early with ? before calling finalize()
        }

        // Should still be able to create contexts
        let ctx =
            crypto.sha256_init(CHUNK_SIZE).expect("Failed to create context after partial update drops");
        ctx.update(data.subrange(0, CHUNK_SIZE).unwrap(), 0, CHUNK_SIZE).expect("update failed");
        let _ = ctx.finalize().expect("finalize failed");

        log::info!("  partial update drop cleanup: OK");
    }

    log::info!("test 5: multi-threaded concurrent sha contexts");
    {
        const NUM_THREADS: usize = 4;
        let barrier = Arc::new(Barrier::new(NUM_THREADS));
        let mut handles = Vec::new();

        // pre-compute expected hashes for each thread's data region
        let mut expected_hashes: Vec<[u8; 32]> = Vec::new();
        for t in 0..NUM_THREADS {
            let start = t * TOTAL_SIZE;
            let end = start + TOTAL_SIZE;
            let expected: [u8; 32] = Sha256::digest(&data.as_slice()[start..end]).into();
            expected_hashes.push(expected);
        }

        for thread_id in 0..NUM_THREADS {
            let barrier = Arc::clone(&barrier);
            let expected_hash = expected_hashes[thread_id];
            let data_start = thread_id * TOTAL_SIZE;

            let handle = thread::spawn(move || {
                let crypto = CryptoApi::default();
                let mut thread_data =
                    map_memory(None, None, TOTAL_SIZE, MemoryFlags::W | MemoryFlags::POPULATE)
                        .expect("failed to allocate thread buffer");

                // initialize with a thread-specific pattern
                for (i, d) in thread_data.as_slice_mut().iter_mut().enumerate() {
                    *d = (data_start + i).wrapping_mul(17).wrapping_add(31) as u8;
                }

                // wait for all threads to be ready
                barrier.wait();

                let ctx = crypto
                    .sha256_init(TOTAL_SIZE)
                    .expect(&format!("thread {thread_id}: Failed to create context"));

                for chunk_idx in 0..4 {
                    let offset = chunk_idx * CHUNK_SIZE;
                    ctx.update(thread_data.subrange(offset, CHUNK_SIZE).unwrap(), 0, CHUNK_SIZE)
                        .expect(&format!("thread {thread_id}: update failed at chunk {chunk_idx}"));
                }

                let hash = ctx.finalize().expect(&format!("thread {thread_id}: finalize failed"));
                assert_eq!(hash.as_slice(), expected_hash.as_slice(), "thread {thread_id}: hash mismatch");

                log::info!("  thread {thread_id}: hash verified OK");
                thread_id
            });

            handles.push(handle);
        }

        // wait for all threads to complete
        for handle in handles {
            handle.join().expect("thread panicked");
        }
        log::info!("  all {NUM_THREADS} threads completed successfully");
    }

    log::info!("test 6: testing other SHA algorithms with streaming");
    {
        use crypto::messages::ShaAlgo;

        // SHA-224
        let ctx = crypto.sha_init(ShaAlgo::Sha224, CHUNK_SIZE).expect("SHA224 init failed");
        ctx.update(data.subrange(0, CHUNK_SIZE).unwrap(), 0, CHUNK_SIZE).expect("SHA224 update failed");
        let hash = ctx.finalize().expect("SHA224 finalize failed");
        let expected: [u8; 28] = Sha224::digest(&data.as_slice()[0..CHUNK_SIZE]).into();
        assert_eq!(hash.as_slice(), expected.as_slice(), "SHA224 streaming hash mismatch");

        // SHA-384
        let ctx = crypto.sha_init(ShaAlgo::Sha384, CHUNK_SIZE).expect("SHA384 init failed");
        ctx.update(data.subrange(0, CHUNK_SIZE).unwrap(), 0, CHUNK_SIZE).expect("SHA384 update failed");
        let hash = ctx.finalize().expect("SHA384 finalize failed");
        let expected: [u8; 48] = Sha384::digest(&data.as_slice()[0..CHUNK_SIZE]).into();
        assert_eq!(hash.as_slice(), expected.as_slice(), "SHA384 streaming hash mismatch");

        // SHA-512
        let ctx = crypto.sha_init(ShaAlgo::Sha512, CHUNK_SIZE).expect("SHA512 init failed");
        ctx.update(data.subrange(0, CHUNK_SIZE).unwrap(), 0, CHUNK_SIZE).expect("SHA512 update failed");
        let hash = ctx.finalize().expect("SHA512 finalize failed");
        let expected: [u8; 64] = Sha512::digest(&data.as_slice()[0..CHUNK_SIZE]).into();
        assert_eq!(hash.as_slice(), expected.as_slice(), "SHA512 streaming hash mismatch");

        log::info!("  all SHA algorithms: OK");
    }

    log::info!("test 7: testing non-word-aligned final chunk in multi-chunk streaming");
    {
        // Use a page-aligned buffer that's large enough for all test cases
        let buf_size = CHUNK_SIZE * 4; // 16KB buffer, page-aligned

        // Test various non-aligned final chunk sizes (1, 2, 3 bytes past word boundary)
        for final_extra_bytes in 1..=3 {
            let first_chunk_size = CHUNK_SIZE; // Word-aligned first chunk (4096 bytes = 1 page)
            let final_chunk_size = CHUNK_SIZE + final_extra_bytes; // Non-word-aligned final chunk
            let total_size = first_chunk_size + final_chunk_size;

            // Prepare test data in a contiguous region
            for (i, d) in data.as_slice_mut()[..total_size].iter_mut().enumerate() {
                *d = ((i * 13 + 7) % 256) as u8;
            }

            // Compute expected hash using software implementation
            let expected: [u8; 32] = Sha256::digest(&data.as_slice()[..total_size]).into();

            // Test streaming with non-aligned final chunk
            let ctx = crypto.sha256_init(total_size).expect("init failed");

            // First chunk (word-aligned) - use offset 0 within page-aligned buffer
            ctx.update(data.subrange(0, buf_size).unwrap(), 0, first_chunk_size)
                .expect("first update failed");

            // Final chunk (non-word-aligned) - use offset first_chunk_size
            // The buffer is page-aligned, offset is 64-byte aligned (SHA_DMA_ALIGNMENT)
            let final_update = ctx.update(
                data.subrange(0, buf_size).unwrap(),
                first_chunk_size, // offset within buffer
                final_chunk_size,
            );

            match final_update {
                Ok(_) => {
                    let hash = ctx.finalize().expect("finalize failed");
                    assert_eq!(
                        hash.as_slice(),
                        expected.as_slice(),
                        "hash mismatch for non-aligned final chunk with {} extra bytes",
                        final_extra_bytes
                    );
                }
                Err(e) => {
                    panic!(
                        "final update failed with unexpected error for {} extra bytes: {:?}",
                        final_extra_bytes, e
                    );
                }
            }
        }

        log::info!("  non-word-aligned final chunk: OK");
    }

    log::info!("test 8: testing various total message sizes with non-aligned lengths");
    {
        // Test message sizes that are 1, 2, 3 bytes past various boundaries
        // Note: For small sizes, we use single-shot (one update). For larger sizes,
        // we use multi-chunk with a page-aligned first chunk.
        let test_sizes = [
            17,   // Small non-aligned
            63,   // Just under block size
            65,   // Just over block size
            127,  // Just under 2x block size
            129,  // Just over 2x block size
            1023, // Larger non-aligned
            4097, // Just over page size
            8193, // Two pages + 1
        ];

        // Use a page-aligned buffer large enough for all tests
        let buf_size = CHUNK_SIZE * 4; // 16KB

        for &total_size in &test_sizes {
            // Prepare test data
            for (i, d) in data.as_slice_mut()[..total_size].iter_mut().enumerate() {
                *d = ((i * 31 + 17) % 256) as u8;
            }

            // Compute expected hash
            let expected: [u8; 32] = Sha256::digest(&data.as_slice()[..total_size]).into();

            // For sizes >= 2 pages, test multi-chunk with non-aligned final
            if total_size > CHUNK_SIZE {
                // First chunk is one page, final chunk is the rest (non-aligned)
                let first_chunk = CHUNK_SIZE;
                let final_chunk = total_size - first_chunk;

                let ctx = crypto.sha256_init(total_size).expect("init failed");

                // First chunk at offset 0
                ctx.update(data.subrange(0, buf_size).unwrap(), 0, first_chunk).expect("first update failed");

                // Final chunk at offset first_chunk
                let final_update = ctx.update(data.subrange(0, buf_size).unwrap(), first_chunk, final_chunk);

                match final_update {
                    Ok(_) => {
                        let hash = ctx.finalize().expect("finalize failed");
                        assert_eq!(
                            hash.as_slice(),
                            expected.as_slice(),
                            "hash mismatch for multi-chunk total_size={}",
                            total_size
                        );
                    }
                    Err(e) => {
                        panic!("final update failed for total_size={}: {:?}", total_size, e);
                    }
                }
            } else {
                // For small sizes, test single-shot non-aligned
                let ctx = crypto.sha256_init(total_size).expect("init failed");

                ctx.update(data.subrange(0, buf_size).unwrap(), 0, total_size)
                    .expect(&format!("single update failed for total_size={}", total_size));

                let hash = ctx.finalize().expect("finalize failed");
                assert_eq!(
                    hash.as_slice(),
                    expected.as_slice(),
                    "hash mismatch for single-shot total_size={}",
                    total_size
                );
            }
        }

        log::info!("  various non-aligned message sizes: OK");
    }

    log::info!("test 9: invalid offset/length returns InvalidParameter");
    {
        let total_len = CHUNK_SIZE + 1;
        let ctx = crypto.sha256_init(total_len).expect("init failed");
        let buf = data.subrange(0, CHUNK_SIZE).unwrap();
        match ctx.update(buf, 0, total_len) {
            Err(CryptoError::InvalidParameter) => {
                log::info!("  invalid offset/length: OK");
            }
            Err(e) => {
                panic!("unexpected error for invalid offset/length: {:?}", e);
            }
            Ok(_) => {
                panic!("invalid offset/length unexpectedly succeeded");
            }
        }
    }

    log::info!("all streaming SHA tests passed");
}
