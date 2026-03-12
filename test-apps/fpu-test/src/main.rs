// SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

// Won't compile without this symbol defined
#[no_mangle]
static __aeabi_unwind_cpp_pr1: usize = 0;

static mut T1_RES: f32 = 0.0;
const T1_EXPECTED_RESULT: f32 = 2.7048109;
static mut T2_RES: f32 = 0.0;
const T2_EXPECTED_RESULT: f32 = 0.36971152;

pub fn main() {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Info); // Switch to Debug if test fails to get more info

    log::info!("Starting");

    let thread1 = xous::create_thread_0(thread1).expect("create thread");
    let thread2 = xous::create_thread_0(thread2).expect("create thread");
    xous::wait_thread(thread1).expect("join thread");
    xous::wait_thread(thread2).expect("join thread");

    unsafe {
        assert_eq!(T1_RES, T1_EXPECTED_RESULT, "Unexpected result from Thread 1");
        assert_eq!(T2_RES, T2_EXPECTED_RESULT, "Unexpected result from Thread 2");
    }

    log::info!("Success");
}

fn thread1() {
    let tid = xous::current_tid().expect("get tid");

    let res = 1.0f32;
    let val = 1.01f32;
    for i in 0..100 {
        // Perform
        //    res = res * val;
        // But with a context switch in between.
        unsafe {
            core::arch::asm!(
                "vldr.f32 s0, [{res_in}]",
                "vldr.f32 s2, [{val_in}]",
                res_in = in(reg) &res,
                val_in = in(reg) &val,
            );
            xous::yield_slice();
            core::arch::asm!(
                "vmul.f32 s0, s0, s2",
                "vstr.f32 s0, [{res_out}]",
                res_out = in(reg) &res,
            );

            log::debug!("Thread #{}, step #{}: {}", tid, i, res);
            T1_RES = res;
        }
    }
}

fn thread2() {
    let tid = xous::current_tid().expect("get tid");

    let res = 1.0f32;
    let val = 1.01f32;
    for i in 0..100 {
        // Perform
        //    res = res / val;
        // But with a context switch in between.
        unsafe {
            core::arch::asm!(
                "vldr.f32 s0, [{res_in}]",
                "vldr.f32 s2, [{val_in}]",
                res_in = in(reg) &res,
                val_in = in(reg) &val,
            );
            xous::yield_slice();
            core::arch::asm!(
                "vdiv.f32 s0, s0, s2",
                "vstr.f32 s0, [{res_out}]",
                res_out = in(reg) &res,
            );

            log::debug!("Thread #{}, step #{}: {}", tid, i, res);
            T2_RES = res;
        }
    }
}
