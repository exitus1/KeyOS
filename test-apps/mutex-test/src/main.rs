// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{Arc, Mutex};

use xous::current_tid;

pub fn main() {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info); // Switch to Debug if test fails to get more info

    log::info!("Starting");

    let val = Arc::new(Mutex::new(0));

    let val_clone = val.clone();
    let val_clone = &val_clone as *const _ as usize;
    let t1 = xous::create_thread_1(
        move |a| {
            let val = unsafe { &*(a as *const Arc<Mutex<u32>>) };
            let tid = current_tid().expect("tid");

            loop {
                let mut val = val.lock().expect("lock");
                *val += 1;
                log::info!("TID {}: {}", tid, val);
            }
        },
        val_clone,
    )
    .expect("t1 create");

    let val_clone = val.clone();
    let val_clone = &val_clone as *const _ as usize;
    let t2 = xous::create_thread_1(
        move |a| {
            let val = unsafe { &*(a as *const Arc<Mutex<u32>>) };
            let tid = current_tid().expect("tid");

            loop {
                let mut val = val.lock().expect("lock");
                *val += 1;
                log::info!("TID {}: {}", tid, val);
            }
        },
        val_clone,
    )
    .expect("t2 create");

    xous::wait_thread(t1).expect("t1 join");
    xous::wait_thread(t2).expect("t2 join");

    log::info!("Success");
}
