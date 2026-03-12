#-------------------------------------------------------------------------------
#	Copyright (c) 2020, Microchip Technology.
#
#	This program is free software; you can redistribute it and/or modify it
#	under the terms and conditions of the GNU General Public License,
#	version 2, as published by the Free Software Foundation.
#
#	This program is distributed in the hope it will be useful, but WITHOUT
#	ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
#	FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for
#	more details.
#-------------------------------------------------------------------------------
import Crypto.Cipher.AES
import binascii
import hashlib
import logging
from conversions import *

class AES_PRF():
    # all parameters ares en hexadecimal format
    def __init__(self):
        self.__K__ = "" # The LSB_128bits(ROM Key)
        self.__K1__ = "" # to generate using CMAC
        self.__K2__ = "" # to generate using CMAC
        self.__R128__ = "00"*(120//8) + "87"  # R_128== 0^120||10000111 (bits representation)

    def Set__K__(self, __K__):
        self.__K__ = __K__

    def Get__K__(self):
        return self.__K__

    def Get__K1__(self):
        return self.__K1__

    def Get__K2__(self):
        return self.__K2__

    def Gen__K1__and__K2__(self):
        """
            Generate K1 and K2
        """
        Temp = Crypto.Cipher.AES.new(binascii.unhexlify(self.__K__), Crypto.Cipher.AES.MODE_ECB )
        L = binascii.hexlify(Temp.encrypt(binascii.unhexlify("00"*16))).decode("latin-1")   # compute L = CIPH_k(0^b) '(encrypt b bits of 0)'
        Temp2 = Hex2Int(L)
        Temp2 = Int2Hex(Temp2.__lshift__(1), NbChar=32)
        if (len(Temp2)>0x20):
            Temp2 = Temp2[1:]
        if (Hex2Int(L[0:2]) & 0x80) == 0:
            self.__K1__ = Temp2
        else:
            self.__K1__ = Int2Hex(Hex2Int(Temp2) ^ Hex2Int(self.__R128__), NbChar=32)
        Temp2 = Int2Hex(Hex2Int(self.__K1__).__lshift__(1))
        if (len(Temp2)>0x20):
            Temp2 = Temp2[1:]
        if (Hex2Int(self.__K1__[0:2]) & 0x80) == 0:
            self.__K2__ = Temp2
        else:
            self.__K2__ = Int2Hex(Hex2Int(Temp2) ^ Hex2Int(self.__R128__), NbChar=32)

    def CMAC_Generation(self, HexMessage):
        """ *******************************
                +-----+      +-----+
                | M1  |      | M2* |
                +-----+      +-----+
                   |            |
                   |     +---> (+)<--|K1|
                   |     |      |
                +-----+  |   +-----+
                |AES_K|  |   |AES_K|
                +-----+  |   +-----+
                   |     |      |
                   +-----+      |
                             +-----+
                             |  T  |
                             +-----+
            ********************************
            Compute the encryption of of HexMessage
            len(HexMessage) == 32 bytes
            M1 = the 1st 16 bytes
            M2 = the last 16 bytes
        """
        self.Gen__K1__and__K2__()        # Generate __K1__
        Nb_Blocs = int(((len(HexMessage)-1)/32) + 1)
        C = "0"*32
        Temp = Crypto.Cipher.AES.new(binascii.unhexlify(self.__K__), Crypto.Cipher.AES.MODE_ECB)
        for i in range(Nb_Blocs-1):
            aaa = Hex2Int(HexMessage[i*32 : i*32 + 32])^Hex2Int(C)
            Mi = Int2Hex(aaa, NbChar=32)
            C = binascii.hexlify(Temp.encrypt(binascii.unhexlify(Mi))).decode("latin-1")

        Mn = HexMessage[(Nb_Blocs-1)*32:]
        if len(Mn)<32:
            __Ki__ = self.__K2__
            Mn += Int2Hex(1 << (((32 - len(Mn))*4) - 1))
        else:
            __Ki__ = self.__K1__
        Mn = Int2Hex(Hex2Int(__Ki__)^Hex2Int(Mn)^Hex2Int(C), NbChar=32) # Compute M2 Xor __K1__ Xor C(i-1)
        T = binascii.hexlify(Temp.encrypt(binascii.unhexlify(Mn))).decode("latin-1")
        return T

class Key_Derivation():
    """
        Key Derivation based on AES CMAC
        @params:
            __KeyI__ : The AES CMAC Key on 128, 192 or 256 bits in hexa
            __L__ : size in bit of the output on 4 bytes in hexa
                default = "00000280"=640bits = 5*128bits
            __Context__ : Context 4 bytes in hexa
            __Label__ : Label 7 bytes in hexa
            __Nb_Loops__ : Number of Loops in integer
                default = 5
            __IV__ : Initial value on 16 bytes in hexa
        @output:
            Number __Nb_Loops__ blocks of 128bits in hexa
            Default usage with __Nb_Loops__ = 5 :
                KEY : 256 bits
                IV_KEY: 128 bits
                CMAC_KEY: 256 bits
    """
    def __init__ (self, __KeyI__ = "00"*16,
                        __L__ = "00000280",
                        __Context__= "00000000",
                        __Label__= "00000000000000",
                        __IV__="00"*16,
                        __Nb_Loops__ = 5,
                        __padding__ = "00"):
        self.__KeyI__ = __KeyI__
        self.__L__ = __L__
        self.__Context__ = __Context__
        self.__Label__ = __Label__
        self.__IV__=__IV__
        self.__padding__ = __padding__
        self.__Ki__ = ""
        self.DataIn = ""
        self.DataOut = ""
        self.__Nb_Loops__ = __Nb_Loops__

    def Set__L__(self, __L__):
        self.__L__ = __L__

    def Set__Context__(self, __Context__):
        self.__Context__ = __Context__

    def Set__Label__(self, __Label__):
        self.__Label__ = __Label__

    def Set__KeyI__(self, __KeyI__):
        self.__KeyI__ = __KeyI__

    def Set__Ki__(self, __Ki__):
        self.__Ki__ = __Ki__

    def Set__Nb_Loops__(self, __Nb_Loops__):
        self.__Nb_Loops__ = __Nb_Loops__

    def Set_DataIn(self):
        self.DataIn = self.__Ki__ + self.__Label__ + self.__padding__ + self.__Context__ + self.__L__

    def Compute_Derivations(self):
        self.DataOut = ""
        AES_PRF_Obj = AES_PRF()
        AES_PRF_Obj.Set__K__(self.__KeyI__)  # AES Key
        self.Set__Ki__(self.__IV__)
        for i in range(self.__Nb_Loops__):
            self.Set_DataIn()

            self.Set__Ki__(AES_PRF_Obj.CMAC_Generation(self.DataIn))
            self.DataOut += self.__Ki__
        return self.DataOut

def GenerateKeysFromSeed256Bits(Seed256Bits, Label=Chr2Hex("TEMP")):
    """
        Generate KEY, IV_KEY and CMAC_KEY from seed and label
        @params:
            Seed256Bits : 256bits in hexa
            __Label__ : 4 bytes in hexa (optional)
        @output:
            KEY : 256 bits
            IV_KEY: 128 bits
            CMAC_KEY: 256 bits
    """
    TpmLabel = ""
    if (len(Label) < 7):
        TpmLabel = Label
        for i in range(7 - len(Label)):
            TpmLabel = TpmLabel + "00"
    else:
        TpmLabel = Label[0:7]
    Tmp_Deriv_Key = Key_Derivation(__KeyI__=Seed256Bits[32:64], __Label__=TpmLabel, __IV__=Seed256Bits[0:32])
    KEYS = Tmp_Deriv_Key.Compute_Derivations()
    return KEYS[0:64], KEYS[64:96], KEYS[96:]

#############
#   Tests   #
#############
def main():
    KEY = "a1b8a95b30794147cd6e61ba712f174ee45ffe63b199149c032d983bc2064ac1"
    IV = "dc8530058c9503162de934710037d56c"
    cust_id = "C26242CF0C4AF0A067"

    KEY_CUST = ""
    TmpDivCUSTKey = Key_Derivation( __KeyI__ = KEY,
                                    __L__ = "00000280",
                                    __Context__= "FEFEFEFE",
                                    __Label__= Chr2Hex("DIV_ROM\0"),
                                    __IV__= IV,
                                    __Nb_Loops__ = 5)
    # CUST_Key derivation
    KEYS = TmpDivCUSTKey.Compute_Derivations()
    print(KEYS[:64])
    print(KEYS[64:96])
    print(KEYS[96:160])

if __name__ == '__main__':
    main()