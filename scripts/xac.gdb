# SPDX-FileCopyrightText: 2014 Mark Plotnick
# SPDX-License-Identifier: CC-BY-SA-3.0
#
# From:
# https://stackoverflow.com/questions/25786982/how-can-gdb-show-both-hex-and-ascii-when-examing-memory

define xac
    dont-repeat
    set $addr = (char *)($arg0)
    set $endaddr = $addr + $arg1
    while $addr < $endaddr
        printf "%p: ", $addr
        set $lineendaddr = $addr + 8
        if $lineendaddr > $endaddr
            set $lineendaddr = $endaddr
        end
        set $a = $addr
        while $a < $lineendaddr
            printf "0x%02x ", *(unsigned char *)$a
            set $a++
        end
        printf "'"
        set $a = $addr
        while $a < $lineendaddr
            printf "%c", *(char *)$a
            set $a++
        end
        printf "'\n"
        set $addr = $addr + 8
    end
end

document xac
usage: xac <address> <count>
end
