# SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
# SPDX-License-Identifier: GPL-3.0-or-later

#
# MMU-related cp15 commands
#

define get_pid
  mon cp15 13 0 0 1
end

define get_ttbr0
  mon cp15 2 0 0 0
end

define get_ttbr1
  mon cp15 2 0 0 1
end

define mmu_disable
  mon cp15 1 0 0 0 = 0x10C51878
end

define mmu_enable
  mon cp15 1 0 0 0 = 0x10C51879
end

define tlb_flush
  mon cp15 8 3 0 0 = 1
  mon cp15 8 7 0 0 = 1
end

define read_dfar_ifar
  mon cp15 6 0 0 0
  mon cp15 6 0 0 2
end

define clear_dfar_ifar
  mon cp15 6 0 0 0 = 0
  mon cp15 6 0 0 2 = 0
end

define read_dfsr_ifsr
  mon cp15 5 0 0 0
  mon cp15 5 0 0 1
end

define clear_dfsr_ifsr
  mon cp15 5 0 0 0 = 0
  mon cp15 5 0 0 1 = 0
end

define peek
  mon long $arg0
end

# --
# Note: these address translation functions sometimes return inaccurate results
# even when the translation is defined
# --

define v2p_priv
  mon cp15 7 8 0 0 = $arg0
  mon cp15 7 4 0 0
end
define v2p_user
  mon cp15 7 8 0 2 = $arg0
  mon cp15 7 4 0 0
end

# --
