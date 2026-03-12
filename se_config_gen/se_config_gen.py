#!/usr/bin/env python3
#
# SPDX-FileCopyrightText: © 2020 Foundation Devices, Inc. <hello@foundationdevices.com>
# SPDX-License-Identifier: GPL-3.0-or-later
#
# SPDX-FileCopyrightText: 2018 Coinkite, Inc. <coldcardwallet.com>
# SPDX-License-Identifier: GPL-3.0-only
#
# (c) Copyright 2018 by Coinkite Inc. This file is part of Coldcard <coldcardwallet.com>
# and is covered by GPLv3 license found in COPYING.
#
# Determine bits needed to configure ATECC608A for Passport.
#
# Some secrets are configured at factory initialization time, and are then used by
# the secure code in the main firmware.
#

import sys
from secel_config import *
from textwrap import TextWrapper
from contextlib import contextmanager
from binascii import unhexlify as a2b_hex

# Specific slots (aka key numbers) are reserved for specific purposes.


class Slot:
    # reserve 0: it's weird
    IoProtectionSecret = 1
    PinStretch = 2
    PinAttempt = 3
    PinHash = 4
    MatchCount = 5
    LastGood = 7
    AesEntropy = 8
    FirmwareTimestamp = 9
    Seed = 10
    SecurityCheckPrivateKey = 11
    KeycardAuthenticity = 12
    SeedFingerprint = 14
    FidoPrivateKey = 15


class SEConfig:
    def __init__(self):
        # typical data from a specific virgin chip; serial number and hardware rev will vary!
        self.data = bytearray(a2b_hex('01233b7e00005000e9f5342beec05400c0005500832087208f20c48f8f8f8f8f9f8faf8f0000000000000000000000000000af8fffffffff00000000ffffffff00000000ffffffffffffffffffffffffffffffff00005555ffff0000000000003300330033001c001c001c001c001c003c003c003c003c003c003c003c001c00'))  # nopep8
        assert len(self.data) == 4 * 32 == 128
        self.d_slot = [None] * 16

    def set_slot(self, n, slot_conf, key_conf):
        assert 0 <= n <= 15, n
        assert isinstance(slot_conf, SlotConfig)
        assert 'KeyConfig' in str(type(key_conf))

        self.data[20 + (n * 2): 22 + (n * 2)] = slot_conf.pack()
        self.data[96 + (n * 2): 98 + (n * 2)] = key_conf.pack()

    def set_combo(self, n, combo):
        self.set_slot(n, combo.sc, combo.kc)

    def get_combo(self, n):
        rv = ComboConfig()
        blk = self.data
        rv.kc = KeyConfig.unpack(blk[96 + (2 * n):2 + 96 + (2 * n)])
        rv.sc = SlotConfig.unpack(blk[20 + (2 * n):2 + 20 + (2 * n)])
        return rv

    def set_otp_mode(self, read_only):
        # set OTPmode for consumption or read only
        # default is consumption.
        self.data[18] = 0xAA if read_only else 0x55

    def dump(self):
        secel_dump(self.data)

    def set_gpio_config(self, kn):
        # GPIO is active-high output, controlled by indicated key number
        assert 0 <= kn <= 15
        assert self.data[14] & 1 == 0, "can only work on chip w/ SWI not I2C"
        self.data[16] = 0x1 | (kn << 4)     # "Auth0" mode in table 7-1

    def disable_KdfIvLoc(self):
        # prevent use of weird AES KDF init vector junk
        self.data[72] = 0xf0

    def checks(self):
        # reserved areas / known values
        c = self.data
        assert c[17] == 0               # reserved
        if self.partno == 5:
            assert c[18] in (0xaa, 0x55)    # OTPmode
        assert c[86] in (0x00, 0x55)    # LockValue
        if self.partno == 5:
            assert set(c[90:96]) == set([0])  # RFU, X509Format
        if self.partno == 6:
            assert set(c[92:96]) == set([0])  # RFU, X509Format


class SEConfig608(SEConfig):
    def __init__(self):
        # typical data from a specific virgin chip; serial number and hardware rev will vary!
        self.data = bytearray(a2b_hex('01236c4100006002bbe66928ee015400c0000000832087208f20c48f8f8f8f8f9f8faf8f0000000000000000000000000000af8fffffffff00000000ffffffff000000000000000000000000000000000000000000005555ffff0000000000003300330033001c001c001c001c001c003c003c003c003c003c003c003c001c00'))  # nopep8
        assert len(self.data) == 4 * 32 == 128
        self.d_slot = [None] * 16
        self.partno = 6

    def counter_match(self, kn):
        assert 0 <= kn <= 15
        self.data[18] = (kn << 4) | 0x1

    @contextmanager
    def chip_options(self):
        co = ChipOptions.unpack(self.data[90:92])
        yield co
        self.data[90:92] = co.pack()


def rust_dump_hex(buf):
    # format for CPP macro
    txt = ' '.join('0x%02x,' % i for i in buf)
    tw = TextWrapper(width=106)
    return '\n'.join('    %s' % i for i in tw.wrap(txt))


def main():
    doit(SEConfig608())


def doit(se):
    # default all slots to storage
    cfg = [ComboConfig() for i in range(16)]
    for j in range(16):
        cfg[j].for_storage()

    # Seed is freely writable, but can only be read with the PIN
    cfg[Slot.Seed].secret_storage(Slot.PinHash).deterministic().require_auth(Slot.IoProtectionSecret)
    cfg[Slot.Seed].sc.WriteConfig = 0x0
    cfg[Slot.Seed].sc.WriteKey = 0x0

    # PIN is not readable, writable with seed
    cfg[Slot.PinHash].hash_key(write_kn=Slot.Seed).require_auth(Slot.IoProtectionSecret).lockable(False)

    # unique keys per-device
    # - pairing key for linking SE and main micro together
    # - critical!
    cfg[Slot.IoProtectionSecret].hash_key()

    # chip-enforced pin attempts: link keynum and enable "match count" feature
    cfg[Slot.MatchCount].writeable_storage(Slot.PinHash).require_auth(Slot.IoProtectionSecret)
    se.counter_match(Slot.MatchCount)

    # new slots related to pin attempt- and rate-limiting
    # - both hold random, unknown contents, can't be changed
    # - use of the first one will cost a counter incr
    # - actual PIN to be used is rv=HMAC(pin_stretch, rv) many times
    cfg[Slot.PinAttempt].hash_key().require_auth(Slot.IoProtectionSecret).deterministic().limited_use()

    # to rate-limit PIN attempts (also used for prefix words) we require
    # many HMAC cycles using this random+unknown value.
    cfg[Slot.PinStretch].hash_key().require_auth(Slot.IoProtectionSecret).deterministic()

    # Seed fingerprint - updatable with PIN, readable with just the pairing secret
    cfg[Slot.SeedFingerprint].writeable_storage(Slot.Seed).require_auth(Slot.IoProtectionSecret)

    cfg[Slot.SecurityCheckPrivateKey].ec_key().require_auth(Slot.IoProtectionSecret).lockable(True)
    cfg[Slot.FidoPrivateKey].ec_key(priv_write=True).require_auth(Slot.IoProtectionSecret).lockable(True)

    cfg[Slot.AesEntropy].secret_storage(Slot.PinHash).require_auth(Slot.IoProtectionSecret)
    cfg[Slot.AesEntropy].sc.WriteKey = Slot.Seed

    # written and locked during provisioning
    cfg[Slot.KeycardAuthenticity].hash_key().require_auth(Slot.IoProtectionSecret).deterministic().lockable(True)

    # turn off selftest feature (performance problem), and enforce encryption
    # (io protection) for verify, etc.
    with se.chip_options() as opt:
        opt.POSTEnable = 0
        opt.IOProtKeyEnable = 1
        opt.ECDHProt = 0x1      # allow encrypted output
        opt.KDFProt = 0x1       # allow encrypted output
        opt.IOProtKey = Slot.IoProtectionSecret

    # don't want
    se.disable_KdfIvLoc()

    # used to hold counter0 value when we last successfully got that PIN
    cfg[Slot.LastGood].writeable_storage(Slot.PinHash).require_auth(Slot.IoProtectionSecret)

    cfg[Slot.FirmwareTimestamp].for_storage(lockable=False).require_auth(Slot.IoProtectionSecret)

    assert len(cfg) == 16
    for idx, x in enumerate(cfg):
        se.set_combo(idx, cfg[idx])

    se.checks()

    # se.dump()

    print('// SP DX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>'.replace("SP DX", "SPDX"))
    print('// SP DX-License-Identifier: GPL-3.0-or-later\n'.replace("SP DX", "SPDX"))

    print('//! Autogenerated; see se_config_gen.\n')

    # generate slot numbers
    print('/// Slot numbers for the SE.')
    print('#[derive(Debug, Clone, Copy, PartialEq, Eq)]')
    print('#[repr(u16)]')
    print('pub enum Slot {')
    print('    None = 0,')
    names = [nm for nm in dir(Slot) if nm[0] != '_']
    for v, nm in sorted((getattr(Slot, nm), nm) for nm in names):
        print('    %s = %d,' % (nm, v))
    print('}\n')

    # generate the slot `size` function
    print('impl Slot {')
    print('    /// Return the size of the slot in bytes.')
    print('    ///')
    print('    /// # Panics')
    print('    ///')
    print('    /// Panics if the slot number is invalid (not in range 1..=15).')
    print('    pub const fn size(self) -> usize {')
    print('        match self as u8 {')
    print('            1..=7 => 36,')
    print('            8 => 416,')
    print('            9..=15 => 72,')
    print('            _ => panic!("Invalid slot number"),')
    print('        }')
    print('    }')
    print('}\n')

    # generate a single header file we will need
    print('/// Bytes [16..84) of chip config area.')
    print('pub const SE_CONFIG_1: [u8; 68] = [')
    print(rust_dump_hex(se.data[16:84]))
    print('];\n')

    print('/// Bytes [90..128) of chip config area.')
    print('pub const SE_CONFIG_2: [u8; 38] = [')
    print(rust_dump_hex(se.data[90:]))
    print('];\n')

    print('/*')
    se.dump()
    print('*/')


if __name__ == '__main__':
    main()
