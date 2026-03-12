// SPDX-FileCopyrightText: 2026 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

const LOG_CONFIG: log_file::Config = log_file::Config {
    location: log_file::Location::User,
    directory: ".log",
    file_prefix: "log",
    description: "log file",
    retry_on_error: false,
};

fn main() -> ! { log_file::run(LOG_CONFIG) }
