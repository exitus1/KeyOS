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
def Hex2Int(HexValue):
    """
        Convert an hexadecimal string to an integer value
    """
    if len(HexValue) != 0:
        return int(HexValue, 16)
    else: return 0

def Int2Hex(Value, NbChar = 2):
    """
        Convert an integer value to an hexadecimal string
    """
    Res = '%X'%Value
    Res = Res.zfill(NbChar)
    return Res


def Hex2Chr(inHex):
    """
        Convert an hexadecimal string to a string of characters
    """
    Res = ""
    i = 0
    while i < len(inHex)-1:
        Res+= chr(Hex2Int(inHex[i:i+2]))
        i +=2
    return Res

def Chr2Hex(ChrVar):
    """
        Convert a string of characters to an hexadecimal string
    """
    Res = ""
    i = 0
    for i in range(len(ChrVar)):
        Res += Int2Hex(ord(ChrVar[i]))
    return Res

def ReadBinaryFile(Path):
    """
        Read a binary file
    """
    F_In = open(Path, "U")
    Res = F_In.read()
    F_In.close()
    return Res


def main():
    pass

if __name__ == '__main__':
    main()
