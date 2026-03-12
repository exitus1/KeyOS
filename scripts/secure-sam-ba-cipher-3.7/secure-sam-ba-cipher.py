#!/usr/bin/env python3
#-------------------------------------------------------------------------------
#	Copyright (c) 2022, Microchip Technology.
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
import argparse
import os
import re
import sys
try:
    file_name = __file__.replace("/", ";").split(";")[-1:]
    sys.path.insert(1, __file__.replace(file_name[0], "") + "../common")
except:
    pass
import rsa_lib
import crypto_common

__version__="3.7"

HEADER_LICENSE_FILE = "LICENSE___0350__"

sama5d2x_devices = {"sama5d2x":crypto_common.sama5d2x_cipher, \
                    "sama5d2x_nk":crypto_common.sama5d2nk_cipher}
sama5d3x_devices = {"sama5d3x":crypto_common.sama5d3x_cipher, \
                    "sama5d3x_nk":crypto_common.sama5d3nk_cipher}
sama5d4x_devices = {"sama5d4x":crypto_common.sama5d4x_cipher}
sama7g5_devices = {"sama7g5":crypto_common.sama7g5_cipher}
sam9x60_devices = {"sam9x60":crypto_common.sam9x60_cipher}
pic32cxmt_devices = {"pic32cxmt":crypto_common.pic32cxmt_cipher}
sama5d29_devices = {"sama5d29":crypto_common.sama5d29_cipher}
sam9x7x_devices = {"sam9x7x":crypto_common.sam9x7x_cipher}

supported_device_families = {"sama5d2x":sama5d2x_devices, \
                             "sama5d3x":sama5d3x_devices, \
                             "sama5d4x":sama5d4x_devices, \
                             "sama7g5":sama7g5_devices, \
                             "sam9x60":sam9x60_devices, \
                             "pic32cxmt":pic32cxmt_devices, \
                             "sama5d29":sama5d29_devices, \
                             "sam9x7x":sam9x7x_devices}

def _request_license_command(args):
    if (args.device == None):
        sys.stderr.write("Error: device name is missing.\n")
        return False

    try:
        tmp = supported_device_families[args.device]
    except:
        sys.stderr.write("Error: device name not supported.\n")
        return False

    if (args.priv_key_file == None):
        priv_file = "private_key.pem"
    else:
        priv_file = args.priv_key_file

    if (args.rsa_size == None):
        rsa_size = 4096
    else:
        rsa_size = args.rsa_size

    req = crypto_common.generate_license_request(args.device, out_priv_file = priv_file, \
                                                passphrase = args.priv_key_pass, rsa_size = rsa_size)
    if req != None:
        if args.output_file == None:
            sys.stdout.write("%s\n" % req)
        else:
            tmp_file = open(args.output_file, "w")
            tmp_file.write(req)
            tmp_file.close()
            sys.stdout.write("License request successfully written to '%s'.\n" % args.output_file)
        return True
    else:
        sys.stderr.write("Error: Could not generate license request.\n")
        return False
    return True

def _check_common_args(args, check_lic=True):

    if (check_lic):
        if args.license == None:
            sys.stderr.write("Error: No license file specified.\n")
            return False
        if args.priv_key_file == None:
            sys.stderr.write("Error: No device specified.\n")
            return False

    if args.device == None:
        sys.stderr.write("Error: No device specified.\n")
        return False

    return True

def _init_cipher(args, license_line, extract_lic_keys=True):
    """ Read private key file
        decrypt Div ROM Key 2 and update device's parameters
    """
    lic_data = license_line.split(",")
    try:
        device_obj = supported_device_families[args.device][lic_data[0]]()
    except:
        sys.stderr.write("Error: device name not supported.\n")
        return None
    if (extract_lic_keys):
        if not(device_obj.extract_license_args(lic_data, args.priv_key_file, args.priv_key_pass)):
            return None
        return device_obj

def _load_customer_key_file(key_file):
    try:
        key = None
        cbc_key = None
        cbc_iv = None
        cmac_key = None
        master_key = None
        root_ca_hash = None
        auth_type = None
        is_paired = None
        padding = None
        sec_boot_cfg = None
        f = open(key_file, "rt")
        for line in f:
            line = line.strip()
            if len(line) == 0 or line.startswith("#"):
                continue

            match = re.match("KEY_CUST=([0-9A-Fa-f]*)$", line)
            if match != None:
                key = match.group(1)
                continue

            match = re.match("CBC_KEY=([0-9A-Fa-f]{64})$", line)
            if match != None:
                cbc_key = match.group(1)
                continue

            match = re.match("CBC_IV=([0-9A-Fa-f]{32})$", line)
            if match != None:
                cbc_iv = match.group(1)
                continue

            match = re.match("CMAC_KEY=([0-9A-Fa-f]{64})$", line)
            if match != None:
                cmac_key = match.group(1)
                continue

            match = re.match("MASTER_KEY=([0-9A-Fa-f]{64})$", line)
            if match != None:
                master_key = match.group(1)
                continue

            match = re.match("ROOT_CA_HASH=([0-9A-Fa-f]{128})$", line)
            if match != None:
                root_ca_hash = match.group(1)
                continue

            match = re.match("AUTH_TYPE=([0-9A-Fa-f]{8})$", line)
            if match != None:
                auth_type = match.group(1)
                continue

            match = re.match("IS_PAIRED=([0-9A-Fa-f]{8})$", line)
            if match != None:
                is_paired = match.group(1)
                continue

            match = re.match("PADDING=([0-9A-Fa-f]{16})$", line)
            if match != None:
                padding = match.group(1)
                continue

            match = re.match("SEC_BOOT_CFG=([0-9A-Fa-f]{32})$", line)
            if match != None:
                sec_boot_cfg = match.group(1)
                continue

            key = None
            cbc_key = None
            cbc_iv = None
            cmac_key = None
            master_key = None
            root_ca_hash = None
            auth_type = None
            is_paired = None
            padding = None
            sec_boot_cfg = None
            break
        f.close()

        if cbc_key != None and cbc_iv != None and cmac_key != None and root_ca_hash != None and sec_boot_cfg != None:
            return cbc_key + cbc_iv + cmac_key + root_ca_hash + sec_boot_cfg
        if cbc_key != None and cbc_iv != None and cmac_key != None:
            return cbc_key + cbc_iv + cmac_key
        if cbc_key != None and master_key != None and root_ca_hash != None and auth_type != None and is_paired != None and padding != None:
            return cbc_key + master_key + root_ca_hash + auth_type + is_paired + padding
        if key != None:
            return key
    except:
        sys.stderr.write("Error: Could not load customer key from '%s'\n" % key_file)
        return None

    sys.stderr.write("Error: Could not load customer key.\n")
    return None

def _load_application_key_file(key_file):
    try:
        key = None
        iv = None
        cmac_key = None
        f = open(key_file, "rt")
        for line in f:
            line = line.strip()
            if len(line) == 0 or line.startswith("#"):
                continue
            match = re.match("([A-Z_]*)=([0-9A-Fa-f]*)$", line)
            if match != None:
                ident = match.group(1)
                if ident == "KEY":
                    key = match.group(2)
                elif ident == "IV_KEY":
                    iv = match.group(2)
                elif ident == "CMAC_KEY":
                    cmac_key = match.group(2)
                else:
                    key = iv = cmac_key = None
                    break
            else:
                key = iv = cmac_key = None
                break
        f.close()
        if key != None and iv != None and cmac_key != None:
            return key, iv, cmac_key
    except:
        sys.stderr.write("Error: Could not load application keys from '%s'\n" % key_file)
        return None, None, None

    sys.stderr.write("Error: Could not load application keys.\n")
    return None, None, None

def _get_license_lines(license_file, device_family):
    try:
        tmp_file = open(license_file, "r")
        header = tmp_file.readline().replace(":", "").replace("\n", "")
        if(supported_device_families.get(device_family) == None):
            sys.stderr.write("Error: Device family name \"" + device_family + "\" not supported.\n")
            return []
        if (header[:len(header) - len(device_family)] != HEADER_LICENSE_FILE):
            sys.stderr.write("Error: Wrong license header.\n")
            return []
        if (header[-len(device_family):] != device_family):
            sys.stderr.write("Error: Wrong device family name.\n")
            return []
        license_lines = tmp_file.readlines()
        tmp_file.close()
        return license_lines
    except:
        sys.stderr.write("Error: Can't read license file.\n")
        return []

def _customer_key_command(args):
    sys.stdout.write("-- Ciphering Customer Key --\n")
    sys.stdout.write("License: %s\n" % args.license)
    sys.stdout.write("Device: %s\n" % args.device)
    sys.stdout.write("Customer private key file: %s\n" % args.priv_key_file)
    sys.stdout.write("Customer passphrase: %s\n" % args.priv_key_pass)
    sys.stdout.write("Customer Key File: %s\n" % args.key_file)
    sys.stdout.write("Output File: %s\n" % args.output_file)
    sys.stdout.write("Customer Key Padding: %s\n" % args.cust_key_padding)

    if not _check_common_args(args):
        return False

    license_lines = _get_license_lines(args.license, args.device)
    if (len(license_lines) == 0):
        return False

    for lic_line in license_lines:
        ciph = _init_cipher(args, lic_line.replace('\n', ''))

        if ciph == None:
            return False

        out_file_name = args.output_file.replace('.cip', '') + "_" + ciph.device_name + ".cip"
        key = _load_customer_key_file(args.key_file)
        if key == None:
            return False

        if not(ciph.cipher_customer_key(key, out_file_name, args.cust_key_padding)):
            return False
    return True

def _root_cert_hash_command(args):
    sys.stdout.write("-- Ciphering Root Certificate Hash --\n")
    sys.stdout.write("License: %s\n" % args.license)
    sys.stdout.write("Device: %s\n" % args.device)
    sys.stdout.write("Customer private key file: %s\n" % args.priv_key_file)
    sys.stdout.write("Customer passphrase: %s\n" % args.priv_key_pass)
    if args.certs_file != None:
        sys.stdout.write("Certificate Chain File: %s\n" % args.certs_file)
    else:
        sys.stderr.write("Certificate Chain File: (none)\n")

    if not _check_common_args(args):
        return False

    license_lines = _get_license_lines(args.license, args.device)
    if (len(license_lines) == 0):
        return False

    for lic_line in license_lines:
        ciph = _init_cipher(args, lic_line.replace('\n', ''))

        if ciph == None:
            return False

        out_file_name = args.output_file.replace('.cip', '') + "_" + ciph.device_name + ".cip"
        if not ciph.cipher_root_cert_hash(args.certs_file, out_file_name):
            return False
        sys.stdout.write("Root certificate hash successfully written to '%s'.\n" % out_file_name)
    return True

def _bootstrap_command(args):
    sys.stdout.write("-- Ciphering Bootstrap --\n")
    if args.license != None:
        sys.stdout.write("Received License: %s, ignored\n" % args.license)
    sys.stdout.write("Device: %s\n" % args.device)
    sys.stdout.write("Customer Key File: %s\n" % args.key_file)
    if args.priv_key_file != None:
        sys.stdout.write("Private Key File: %s\n" % args.priv_key_file)
    else:
        sys.stdout.write("Private Key File: (none)\n")
    if args.priv_key_pass != None:
        if args.priv_key_pass[:5] == 'file:':
            sys.stdout.write("Private Key Password: from file %s\n" % args.priv_key_pass[5:])
        else:
            sys.stdout.write("Private Key Password: ******** (not displayed)\n")
    else:
        sys.stdout.write("Private Key Password: (none)\n")
    if args.certs_file != None:
        sys.stdout.write("Certificate Chain File: %s\n" % args.certs_file)
    else:
        sys.stderr.write("Certificate Chain File: (none)\n")
    sys.stdout.write("Input File: %s\n" % args.input_file)
    sys.stdout.write("Output File: %s\n" % args.output_file)

    if args.device == None:
        sys.stderr.write("Error: No device specified.\n")
        return False

    bypass_root_cert_verif = False
    try:
        if args.bypass_root_ca_verif.upper() == "TRUE":
            bypass_root_cert_verif = True
    except:
        bypass_root_cert_verif = False

    ciph = supported_device_families[args.device][args.device]()

    key = _load_customer_key_file(args.key_file)
    if key == None:
        return False

    out_file_name = args.output_file.replace('.cip', '') + "_" + ciph.device_name + ".cip"
    if not ciph.cipher_bootstrap(key, args.priv_key_file, args.priv_key_pass,
            args.certs_file, args.input_file, out_file_name, bypass_root_cert_verif):
        return False

    sys.stdout.write("Ciphered bootstrap successfully written to '%s'.\n" % out_file_name)
    return True

def _application_command(args):
    sys.stdout.write("-- Ciphering Application --\n")
    if args.license != None:
        sys.stdout.write("Received License: %s, ignored\n" % args.license)
    sys.stdout.write("Device: %s\n" % args.device)
    sys.stdout.write("Application Key File: %s\n" % args.key_file)
    sys.stdout.write("Input File: %s\n" % args.input_file)
    sys.stdout.write("Output File: %s\n" % args.output_file)
    if args.no_header:
        sys.stdout.write("Output Header: no\n")
    else:
        sys.stdout.write("Output Header: yes\n")

    if not _check_common_args(args, check_lic=False):
        return False
    ciph = supported_device_families[args.device][args.device]()
    if ciph == None:
        sys.stdout.write("Error: device name not supported.\n")
        return False

    app_key, app_iv, app_cmac_key = _load_application_key_file(args.key_file)
    if app_key == None or app_iv == None or app_cmac_key == None:
        sys.stdout.write("Error: wrong key file format.\n")
        return False

    if not ciph.cipher_application(app_key, app_iv, app_cmac_key,
            args.input_file, args.output_file, args.no_header):
        sys.stdout.write("Error: wrong arguments to cipher application.\n")
        return False

    sys.stdout.write("Ciphered application successfully written to '%s'.\n" % args.output_file)
    return True

def main():
    # create the top-level parser
    parser = argparse.ArgumentParser()
    parser.add_argument('-v', '--version', action='version', version=__version__)
    subparsers = parser.add_subparsers(metavar='<subcommand>')

    common_parser = argparse.ArgumentParser(add_help=False)
    common_parser.add_argument("-d", "--device",
                               metavar='device-name',
                               type=str,
                               default=os.getenv("SECURE_SAM_BA_DEVICE"),
                               help="Name of the device (defaults to environment variable SECURE_SAM_BA_DEVICE)")

    # create the parser for the "request-license" command
    subparser = subparsers.add_parser("request-license",
                                      help="Create a license request file")
    subparser.add_argument("-o", "--output-file",
                           metavar='lic-request.txt',
                           type=str,
                           help="Output file for the license request (optional)")
    subparser.add_argument("-d", "--device",
                           metavar='device-name',
                           type=str,
                           default=os.getenv("SECURE_SAM_BA_DEVICE"),
                           help="Name of the device (defaults to environment variable SECURE_SAM_BA_DEVICE)")
    subparser.add_argument("-n", "--rsa_size",
                           metavar='rsa-modulus-size',
                           type=int,
                           help="RSA modulus size to be generated (optional)")
    subparser.add_argument("-pk", "--priv-key-file",
                           metavar='priv_key.pem',
                           type=str,
                           help="Private key file (optional)")
    subparser.add_argument("-pp", "--priv-key-pass",
                           type=str,
                           help="Private key password (optional)")
    subparser.set_defaults(func=_request_license_command)

    # create the parser for the "customer-key" command
    subparser = subparsers.add_parser("customer-key",
                                      help="Create an encrypted file for setting the customer key",
                                      parents=[common_parser])
    subparser.add_argument("-k", "--key-file",
                           type=str,
                           required=True,
                           help="Customer key file")
    subparser.add_argument("-o", "--output-file",
                           type=str,
                           required=True,
                           help="Output file for the 'set customer key' data")
    subparser.add_argument("-l", "--license",
                            metavar='activation-file',
                            type=str,
                            required=True,
                            default=os.getenv("SECURE_SAM_BA_LICENSE"),
                            help="License activation file (defaults to environment variable SECURE_SAM_BA_LICENSE)")
    subparser.add_argument("-pk", "--priv-key-file",
                           metavar='priv_key.pem',
                           type=str,
                           required=True,
                           help="Private key file")
    subparser.add_argument("-pp", "--priv-key-pass",
                           type=str,
                           help="Private key password (optional)")
    subparser.add_argument("-pad", "--cust-key-padding",
                           type=str,
                           help="Customer Key Padding (optional)")
    subparser.set_defaults(func=_customer_key_command)

    # create the parser for the "full-customer-key" command
    subparser = subparsers.add_parser("full-customer-key",
                                      help="Create an encrypted file for setting the full customer key",
                                      parents=[common_parser])
    subparser.add_argument("-k", "--key-file",
                           type=str,
                           required=True,
                           help="Full Customer key file")
    subparser.add_argument("-o", "--output-file",
                           type=str,
                           required=True,
                           help="Output file for the 'set full customer key' data")
    subparser.add_argument("-l", "--license",
                            metavar='activation-file',
                            type=str,
                            required=True,
                            default=os.getenv("SECURE_SAM_BA_LICENSE"),
                            help="License activation file (defaults to environment variable SECURE_SAM_BA_LICENSE)")
    subparser.add_argument("-pk", "--priv-key-file",
                           metavar='priv_key.pem',
                           type=str,
                           required=True,
                           help="Private key file")
    subparser.add_argument("-pp", "--priv-key-pass",
                           type=str,
                           help="Private key password (optional)")
    subparser.add_argument("-pad", "--cust-key-padding",
                           type=str,
                           help="Customer Key Padding (optional)")
    subparser.set_defaults(func=_customer_key_command)

    # create the parser for the "root-cert-hash" command
    subparser = subparsers.add_parser("root-cert-hash",
                                      help="Create an encrypted file for setting the root certificate hash",
                                      parents=[common_parser])
    subparser.add_argument("-c", "--certs-file",
                           type=str,
                           help="Certificate chain file")
    subparser.add_argument("-o", "--output-file",
                           type=str,
                           required=True,
                           help="Output file for the 'set root certificate hash' data")
    subparser.add_argument("-l", "--license",
                            metavar='activation-file',
                            type=str,
                            required=True,
                            default=os.getenv("SECURE_SAM_BA_LICENSE"),
                            help="License activation file (defaults to environment variable SECURE_SAM_BA_LICENSE)")
    subparser.add_argument("-pk", "--priv-key-file",
                           metavar='priv_key.pem',
                           type=str,
                           required=True,
                           help="Private key file")
    subparser.add_argument("-pp", "--priv-key-pass",
                           type=str,
                           help="Private key password (optional)")
    subparser.set_defaults(func=_root_cert_hash_command)

    # create the parser for the "bootstrap" command
    subparser = subparsers.add_parser("bootstrap",
                                      help="Encrypt/sign a bootstrap binary",
                                      parents=[common_parser])
    subparser.add_argument("-l", "--license",
                            metavar='activation-file',
                            type=str,
                            default=os.getenv("SECURE_SAM_BA_LICENSE"),
                            help="License activation file (defaults to environment variable SECURE_SAM_BA_LICENSE)")
    subparser.add_argument("-k", "--key-file",
                           type=str,
                           required=True,
                           help="Customer key file")
    subparser.add_argument("-pk", "--priv-key-file",
                           type=str,
                           help="Private key file")
    subparser.add_argument("-pp", "--priv-key-pass",
                           type=str,
                           help="Private key password")
    subparser.add_argument("-c", "--certs-file",
                           type=str,
                           help="Certificate chain file")
    subparser.add_argument("-b", "--bypass-root-ca-verif",
                           type=str,
                           help="Bypass Root CA certificate Verification for PIC32CXMT")
    subparser.add_argument("-i", "--input-file",
                           type=str,
                           required=True,
                           help="Input file for the bootstrap")
    subparser.add_argument("-o", "--output-file",
                           type=str,
                           required=True,
                           help="Encrypted output file for the bootstrap")
    subparser.set_defaults(func=_bootstrap_command)

    # create the parser for the "application" command
    subparser = subparsers.add_parser("application",
                                      help="Encrypt an application binary",
                                      parents=[common_parser])
    subparser.add_argument("-l", "--license",
                            metavar='activation-file',
                            type=str,
                            default=os.getenv("SECURE_SAM_BA_LICENSE"),
                            help="License activation file (defaults to environment variable SECURE_SAM_BA_LICENSE)")
    subparser.add_argument("-k", "--key-file",
                           type=str,
                           required=True,
                           help="Application key file")
    subparser.add_argument("-i", "--input-file",
                           type=str,
                           required=True,
                           help="Input file for the application")
    subparser.add_argument("-o", "--output-file",
                           type=str,
                           required=True,
                           help="Encrypted output file for the application")
    subparser.add_argument("--no-header",
                           action="store_true",
                           help="Don't add a header to the encrypted application")
    subparser.set_defaults(func=_application_command)

    args = None
    try:
        args = parser.parse_args()
        if not args.func(args):
            return 1
        return 0
    except:
        if (args != None):
            sys.stdout.write("Error: Command arguments error.\n")

if __name__ == "__main__":
    sys.exit(main())

# vim: tabstop=8 expandtab shiftwidth=4 softtabstop=4
