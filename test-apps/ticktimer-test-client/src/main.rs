#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use {
    log::info,
    std::{thread::sleep, time::Duration},
};

fn main() -> ! {
    log_server::init_wait(env!("CARGO_CRATE_NAME")).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    const DELAY_MS: u64 = 5000;

    for i in 0.. {
        info!("Loop #{}, waiting {} ms", i, DELAY_MS);
        sleep(Duration::from_millis(DELAY_MS));
    }

    panic!("Finished an infinite loop");
}
