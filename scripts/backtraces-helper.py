# SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
# SPDX-License-Identifier: GPL-3.0-or-later

import time

STOP_POINTS = ("::main", "::swi_handler")

class CleanBadUnwinds:
    def __init__(self):
        self.name = "fixbt"
        self.enabled = True
        self.priority = 100

    def filter(self, frames):
        for frame in frames:
            yield frame
            function = frame.function()
            if any(function.endswith(stop) for stop in STOP_POINTS):
                return

gdb.frame_filters["fixbt"] = CleanBadUnwinds()

gdb.set_parameter("pagination", "off")
gdb.execute("add-symbol-file target/armv7a-unknown-xous-elf/release/keyos-kernel")
gdb.execute(f"add-symbol-file target/armv7a-unknown-xous-elf/release/{process}")
gdb.execute("target remote :3334")

ss = gdb.lookup_global_symbol ("keyos_kernel::services::SYSTEM_SERVICES")

while True:
    gdb.execute("monitor halt")
    cp15_raw_output = gdb.execute("monitor cp15 13 0 0 1", to_string=True)
    current_pid = int(cp15_raw_output.split("0x")[1].split(")")[0], 16) & 0xff
    try:
        current_process = ss.value()['processes'][current_pid - 1]["Some"]["__0"]["name"]["Some"]["__0"].string("utf-8")
        print(f"Stopped in {current_process}")
        if current_process == process:
            gdb.execute("maint flush register-cache")
            gdb.execute("bt 100")
            print(" --- ")
    except Exception as e:
        print(e)
        pass
    gdb.execute("monitor go")
    time.sleep(0.05)
