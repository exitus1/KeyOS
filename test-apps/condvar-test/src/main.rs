// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ops::Deref;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use xous::current_tid;

pub fn main() {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    log::info!("Starting");

    let mutex_condvar = Arc::new((Mutex::new(false), Condvar::new()));

    let mutex_condvar_clone = mutex_condvar.clone();
    let mutex_condvar_clone = &mutex_condvar_clone as *const _ as usize;
    xous::create_thread_1(
        move |a| {
            thread::sleep(Duration::from_secs(5));

            let tid = current_tid().expect("tid");
            let val = unsafe { &*(a as *const Arc<(Mutex<bool>, Condvar)>) };
            let (lock, cvar) = val.deref();
            let mut started = lock.lock().unwrap();

            log::info!("TID {}: starting", tid);
            *started = true;

            log::info!("TID {}: notifying", tid);
            cvar.notify_one();
        },
        mutex_condvar_clone,
    )
    .expect("t1 create");

    let (lock, cvar) = &*mutex_condvar;
    let mut started = lock.lock().unwrap();
    while !*started {
        log::info!("Waiting");
        started = cvar.wait(started).unwrap();
    }

    log::info!("Success");
}
