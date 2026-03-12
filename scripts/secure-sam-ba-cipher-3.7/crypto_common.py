
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
import os.path
import sys
import rsa_lib
import aes_lib
from conversions import *
from Crypto.Cipher import AES
import binascii
import x509_lib
from Crypto.Random import get_random_bytes
from OpenSSL.crypto import FILETYPE_PEM, load_certificate
from OpenSSL.crypto import X509Store, X509StoreContext, FILETYPE_PEM
from cryptography.hazmat.primitives import serialization
from Crypto.Hash import SHA256, SHA512

def get_passphrase(passphrase):
    priv_key_pass = None
    if (passphrase != None):
        if (len(passphrase) < 6):
            print("Error: Wrong passphrase syntax")
            return False
        if ((passphrase[0:5] != "file:") and (passphrase[0:5] != "pass:")):
            print("Error: Wrong passphrase syntax")
            return False
        if(passphrase[0:5] == "file:"):
            priv_key_pass = open(passphrase[5:], "r").read().replace("\n", "")
        else:
            priv_key_pass = passphrase[5:]
    return priv_key_pass

class device_cipher(object):
    """ license management
        Extract Div_ROM_Key2, encrypted CUST ID, ...
    """
    def __init__(self):
        self.sha_list = {"SHA256": SHA256, "SHA512":SHA512}
        self.div_rom_key2 = None
        self.div_rom_key2_iv = None
        self.div_rom_key2_cmac = None
        self.device_name = None
        self.encrypted_cust_id = None
        self.cust_key_length = None
        self.certificates = []
        self.root_ca_hash = None
        self.vector_6 = 0
        self.bootstrap = None
        self.ciphered_bootstrap = None
        self.label = "4449565f435553" # "DIV_CUS"
        self.idx_cmac_key = 96
        self.bypass_root_ca_verif = False
        self.modulus = 0
        self.pub_exp = 0
        self.modulus_bytes_size = 256

        #default max of RSA signature size supported by all devices
        #except pic32cxmt and gallardo which support a max of 4608
        self.max_rsa_sign_size = 4096 // 8


    def set_bypass_root_ca_verif(self, bypass_root_ca_verif):
        """ Set bypass_root_ca_verif
            :param bypass_root_ca_verif: set the bit "Bypass Root CA Certificate Verif"
                    in the certificates size Vector
        """
        self.bypass_root_ca_verif = bypass_root_ca_verif

    def get_bypass_root_ca_verif(self):
        """ Get bypass_root_ca_verif
                return the boolean self.bypass_root_ca_verif
        """
        return self.bypass_root_ca_verif

    def set_bootstrap(self, bootstrap):
        """ Set bootstrap
            :param bootstrap: the plain bootstrap
        """
        self.bootstrap = bootstrap

    def extract_license_args(self, license_data, priv_key_file, passphrase = None):
        """ Read license file and extract arguments en hex format
            :param license_data: device's license data
            :param priv_key_file: path to the customer's private key file
            :param passphrase: customer's passphrase
        """
        self.encrypted_cust_id = license_data[1]
        # encrypted customer ID = 16 bytes + its CMAC = 16 bytes
        # 32 bytes in hexa == 64 characters
        if (len(self.encrypted_cust_id) != 64):
            return False

        enc_session_key = license_data[2]
        nonce = license_data[3]
        tag = license_data[4]
        ciphertext = license_data[5]

        # decrypt data
        rsa_obj = rsa_lib.rsa()
        try:
            priv_key_pass = get_passphrase(passphrase)
            if (priv_key_pass == False):
                return False
            rsa_obj.import_priv_key_from_pem(open(priv_key_file, "rb").read(), priv_key_pass)
        except:
            print("Error: Can't extract private key from: " + priv_key_file)
            return False

        full_div_rom_key2 = rsa_obj.decrypt_data(enc_session_key.encode('latin-1'),
                                                nonce.encode('latin-1'),
                                                tag.encode('latin-1'),
                                                ciphertext.encode('latin-1')).decode('latin-1')

        self.div_rom_key2 = full_div_rom_key2[0:64]
        self.div_rom_key2_iv = full_div_rom_key2[64:96]
        self.div_rom_key2_cmac = full_div_rom_key2[96:160]
        return True

    def compute_cmac(key, data):
        """ Compute CMAC on given data
            :param key: CMAC key
            :param data: data in bytes format
            result returned in hex format
        """
        # compute CMAC
        aes_prf = aes_lib.AES_PRF()
        aes_prf.Set__K__(key)
        return aes_prf.CMAC_Generation(binascii.hexlify(data).decode("latin-1"))

    def generate_cust_message(self, data, outputfile):
        """ Encrypt given data with div_rom_key2
			Add bootstrap encryption, CMAC computation and RSA signature for SAMA5D2x
            concatenate already encrypted cust ID with the encrypted key before
            writing it into given output file
            :param data: data to be encrypted
            :param outputfile: path to the output file
        """
        # Encrypt the data with the AES div rom key2
        cipher_aes = AES.new(binascii.unhexlify(self.div_rom_key2.encode('latin-1')), AES.MODE_CBC, binascii.unhexlify(self.div_rom_key2_iv.encode('latin-1')))
        ciphertext= binascii.hexlify(cipher_aes.encrypt(binascii.unhexlify(data.encode('latin-1')))).decode("latin-1")

        # compute CMAC
        aes_prf = aes_lib.AES_PRF()
        aes_prf.Set__K__(self.div_rom_key2_cmac)
        ciphertext_cmac = aes_prf.CMAC_Generation(ciphertext)

        # encrypted cust id concatenated with encrypted data
        result = self.encrypted_cust_id + ciphertext + ciphertext_cmac

        tmp_file = open(outputfile, 'wb')
        tmp_file.write(binascii.unhexlify(result.encode('latin-1')))
        tmp_file.close()
        sys.stdout.write("Customer Key successfully written to '%s'.\n" % outputfile)

    def generate_cust_key_payload(self, data, outputfile):
        """ Encrypt given data with transport key, then encrypt transprt key
            with the device's public key
            :param data: data to be encrypted
            :param outputfile: path to the output file
        """
        # Generate random transport key
        transport_key = get_random_bytes(32)
        transport_iv = get_random_bytes(16)
        transport_cmac = get_random_bytes(32)

        # Encrypt transport key with public key
        transport = binascii.hexlify(transport_key).decode("latin-1").zfill(64) + binascii.hexlify(transport_iv).decode("latin-1").zfill(32) + binascii.hexlify(transport_cmac).decode("latin-1").zfill(64)
        transport_int = int(transport, 16)
        encrypted_transport = hex(pow(transport_int, self.pub_exp, self.modulus))[2:].zfill(self.modulus_bytes_size * 2)

        # Encrypt the data with the transport key
        cipher_aes = AES.new(transport_key, AES.MODE_CBC, transport_iv)
        ciphertext= binascii.hexlify(cipher_aes.encrypt(binascii.unhexlify(data.encode('latin-1')))).decode("latin-1")

        # compute CMAC
        aes_prf = aes_lib.AES_PRF()
        aes_prf.Set__K__(binascii.hexlify(transport_cmac).decode("latin-1"))
        ciphertext_cmac = aes_prf.CMAC_Generation(ciphertext)

        # encrypted cust id concatenated with encrypted data
        result = encrypted_transport + ciphertext + ciphertext_cmac

        tmp_file = open(outputfile, 'wb')
        tmp_file.write(binascii.unhexlify(result.encode('latin-1')))
        tmp_file.close()
        sys.stdout.write("Customer Key successfully written to '%s'.\n" % outputfile)

    def set_certificates(self, certs):
        """ Extract certificates from given certificates chain
            :param certs: certificates chain in PEM format
        """
        tmp_certs = certs.split("-----BEGIN CERTIFICATE-----")
        for i in range(len(tmp_certs)):
            if (len(tmp_certs[i]) != 0):
                cert = "-----BEGIN CERTIFICATE-----" + tmp_certs[i]
                self.certificates.append(cert.encode('latin-1'))

    def verify_cert2_by_cert1(self, cert1_pem, cert2_pem):
        """ Verify certificate 2 with certificate 2 public key
            :param cert1_pem: Certificate 1 in PEM format
            :param cert2_pem: Certificate 2 in PEM format
        """
        # extract public key from certificate 1
        x509_obj_1 = x509_lib.x509_rsa()
        x509_obj_1.set_cert_pem(cert1_pem)
        x509_obj_1.extract_pub_key()

        # Get TBS certificate and signature from certificate 2
        # and verify signature
        x509_obj_2 = x509_lib.x509_rsa()
        x509_obj_2.set_cert_pem(cert2_pem)
        x509_obj_2.extract_signature_hash_algo()

        # Initialize RSA object for verification
        rsa_obj = rsa_lib.rsa()
        rsa_obj.import_pub_key(x509_obj_1.pub_key.public_bytes(serialization.Encoding.PEM, serialization.PublicFormat.PKCS1))

        return rsa_obj.verify_signatuture_pkcs1_v1_5(x509_obj_2.get_tbscertificate(),
                                                     x509_obj_2.extract_signature_hash_algo(),
                                                     x509_obj_2.signature_hash_algo)

    def verify_certificates(self):
        """ Verify certificates chain
        """
        store = X509Store()
        for i in range (len(self.certificates) - 1):
            store.add_cert(load_certificate(FILETYPE_PEM, self.certificates[i]))
        store_ctx = X509StoreContext(store, load_certificate(FILETYPE_PEM, self.certificates[-1]))
        return store_ctx.verify_certificate()

        """
        for i in range(len(self.certificates)):
            if (i == 0):
                if not(self.verify_cert2_by_cert1(self.certificates[0], self.certificates[0])):
                    return False
            else:
                if not(self.verify_cert2_by_cert1(self.certificates[i-1], self.certificates[i])):
                    return False
        return True
        """

    def hash_root_cert_pub_key(self, sha_algo="SHA256"):
        """ compute hash on the root certificate's public key
            :param sha_algorithm: hash algorithm
        """
        x509_obj = x509_lib.x509_rsa()
        x509_obj.set_cert_pem(self.certificates[0])
        x509_obj.extract_pub_key()
        self.root_ca_hash = x509_obj.compute_hash_on_pub_key(sha_algo)

    def hash_full_root_cert(self, sha_algo="SHA512"):
        """ compute hash on the root certificate
            :param sha_algorithm: hash algorithm
        """
        x509_obj = x509_lib.x509_rsa()
        x509_obj.set_cert_pem(self.certificates[0])
        cert_der = x509_obj.get_cert_der()
        cert_der_hash = self.sha_list[sha_algo].new(cert_der)
        self.root_ca_hash = binascii.hexlify(cert_der_hash.digest()).decode('utf8')

    def cipher_root_ca_hash(self, outputfile):
        """ Encrypt root ca hash with div_rom_key2
            :param outputfile: path to the output file
        """
        # Encrypt the root ca hash with the AES div rom key2
        cipher_aes = AES.new(binascii.unhexlify(self.div_rom_key2.encode('latin-1')), AES.MODE_CBC, binascii.unhexlify(self.div_rom_key2_iv.encode('latin-1')))
        ciphertext= binascii.hexlify(cipher_aes.encrypt(binascii.unhexlify(self.root_ca_hash.encode('latin-1')))).decode("latin-1")

        # compute CMAC
        aes_prf = aes_lib.AES_PRF()
        aes_prf.Set__K__(self.div_rom_key2_cmac)
        ciphertext_cmac = aes_prf.CMAC_Generation(ciphertext)

        # concatenate ciphered hash and its cmac
        result = self.encrypted_cust_id + ciphertext + ciphertext_cmac

        tmp_file = open(outputfile, 'wb')
        tmp_file.write(binascii.unhexlify(result.encode('latin-1')))
        tmp_file.close()

    def update_sizes_vectors_cipher_bootstrap(self, key, sign_algo, signature_size, certificates_size):
        """ Compute the 6th vector
            :param sign_algo: CMAC or RSA
            :param signature_size: signature size in bytes
            :param certificates_size: certificates size
        """
        if ((len(self.bootstrap) + signature_size) > 0x20000):
            sys.stderr.write("Error: data + signature exceeds %d bytes\n" % 0x20000)
            return False

        # in case of CMAC, the 6th vector = signature size + bootstrap length
        self.vector_6 = signature_size + len(self.bootstrap)

        # in case of RSA, we need to update 6th vector
        if (sign_algo == "RSA"):
            if (certificates_size > 0x20000):
                sys.stderr.write("Error: certificate chain exceeds %d bytes\n" % 0x2000);
                return False
            self.vector_6 = self.vector_6 | (certificates_size << 19)

        # update 6th vector
        bootstrap_hex = binascii.hexlify(self.bootstrap).decode('latin-1')
        vect6_hex_msb = hex(self.vector_6)[2:].zfill(8)
        bootstrap_hex = bootstrap_hex[0:40] + vect6_hex_msb[6:8] + vect6_hex_msb[4:6] + vect6_hex_msb[2:4] + vect6_hex_msb[0:2] + bootstrap_hex[48:]

        self.bootstrap = binascii.unhexlify(bootstrap_hex.encode('latin-1'))

        # Prepare keys
        cbc_key = key[0:64]
        cbc_iv = key[64:96]

        # Encrypt bootstrap
        cipher_aes = AES.new(binascii.unhexlify(cbc_key.encode('latin-1')), AES.MODE_CBC, binascii.unhexlify(cbc_iv.encode('latin-1')))
        self.ciphered_bootstrap = cipher_aes.encrypt(self.bootstrap)

        return True

    def compute_rsa_signature(self, priv_key_file, priv_key_pass, hash_algo, data):
        """ Compute RSA signature on given data
            :param priv_key_file: Private key file in PEM format
            :param priv_key_pass: Private key passphrase
            :param hash_algo: hash algorithm used for signature
            :param data: data to be signed
        """
        rsa_obj = rsa_lib.rsa()
        try:
            rsa_obj.import_priv_key_from_pem(open(priv_key_file, 'rb').read(), passphrase=priv_key_pass)
        except:
            print("Error: Can't import private key from file: " + priv_key_file)
            return ""
        signature = binascii.hexlify(rsa_obj.sign_message_pkcs1_v1_5(data, sha_algo=hash_algo.upper())).decode('latin-1')
        return signature

    def cmac_tag_bootstrap(self, key):
        """ Update the 6th vector, encrypt bootstrap and compute CMAC on it
        """
        sign_algo = "CMAC"
        signature_size = 16
        if not(self.update_sizes_vectors_cipher_bootstrap(key, sign_algo, signature_size, 0)):
            return ""

        # compute CMAC
        aes_prf = aes_lib.AES_PRF()
        aes_prf.Set__K__(key[self.idx_cmac_key:self.idx_cmac_key + 64])
        bootstrap_cmac = aes_prf.CMAC_Generation(binascii.hexlify(self.ciphered_bootstrap).decode('latin-1'))
        ciphered_bootstrap_bundle = binascii.unhexlify(binascii.hexlify(self.ciphered_bootstrap).decode('latin-1') + bootstrap_cmac)
        return ciphered_bootstrap_bundle

    def rsa_sign_bootstrap(self, key, priv_key_file, priv_key_pass, certs_file, hash_algo=""):
        """ sign a bootstrap program.
            priv_key_fn and certs_fn contain
            the private key and the certificate chain file names. If the
            private key is encrypted, its password must be supplied in
            priv_key_pass with one the following formats (without quotes):
            - "pass:XXX"
            - "file:/path/to/password_file.txt"

            bootstrap_fn and output_fn are the filenames for the input and
            output files.

            The file size (including certificates and signature) is written in
            the 6th vector (i.e. at offset 20).
        """
        sign_algo = "RSA"
        certificates_size = 0
        x509_obj = None
        passphrase = None
        certs_der = []

        if (priv_key_pass != None):
            if(priv_key_pass[0:5] == "file:"):
                passphrase = open(priv_key_pass[5:], "r").read()
            else:
                passphrase = priv_key_pass[5:]
        print("DEBUG: rsa_sign_bootstrap - certs_file:", certs_file)
        try:
            self.set_certificates(open(certs_file, "r").read())
        except:
            print("Error: Can't read certificates from file: '" + str(certs_file) + "'")
            return ""
        for i in range(len(self.certificates)):
            x509_obj = None
            x509_obj = x509_lib.x509_rsa()
            x509_obj.set_cert_pem(self.certificates[i])
            # Convert certificates to DER format
            certs_der.append(x509_obj.get_cert_der())
            certificates_size = certificates_size + len(certs_der[i])
            # Get needed arguments from the last certificate
            if (i == len(self.certificates) - 1):
                x509_obj.extract_signature_hash_algo()

            # do check sizes for all certificate, not only for the last one in the chain
            signature_size = x509_obj.get_public_key_bits() // 8
            # you can check the size here to not be bigger than 4096
            # for gallardo and mistral should not exceed 4608
            if signature_size > self.max_rsa_sign_size:
                print("Error: Not supported RSA public key size of " + str(signature_size * 8) + ", " \
                    "maximum supported size is " + str(self.max_rsa_sign_size * 8))
                return ""

        if not(self.update_sizes_vectors_cipher_bootstrap(key, sign_algo, signature_size, certificates_size)):
            return ""

        if (hash_algo == ""):
            hash_to_use = x509_obj.signature_hash_algo
        else:
            hash_to_use = hash_algo
        signature_hex = self.compute_rsa_signature(priv_key_file, passphrase, hash_to_use, self.ciphered_bootstrap)

        if (signature_hex != ""):
            # prepare bundle
            ciphered_bootstrap_bundle = binascii.unhexlify(binascii.hexlify(self.ciphered_bootstrap).decode('latin-1') + signature_hex)
            for i in range(len(certs_der)):
                ciphered_bootstrap_bundle = ciphered_bootstrap_bundle + certs_der[i]
            return ciphered_bootstrap_bundle
        else:
            return ""


    def cipher_and_sign_bootstrap(self, key, priv_key_file, priv_key_pass, certs_file, input_file, output_file, hash_algo=""):
        """ Encryptand sign a bootstrap program.

            The Customer Key must be identical to the one used in
            cipher_customer_key.

            In case of asymmetric signature, priv_key_fn and certs_fn contain
            the private key and the certificate chain file names. If the
            private key is encrypted, its password must be supplied in
            priv_key_pass with one the following formats (without quotes):
            - "pass:XXX"
            - "file:/path/to/password_file.txt"

            bootstrap_fn and output_fn are the filenames for the input and
            output files.

            The file size (including CMAC) is written in the 6th vector
            (i.e. at offset 20).

            Returns True if the file has been saved successfully, False
            otherwise.
        """
        # Read bootstrap
        try:
            bootstrap = open(input_file, "rb").read()
        except:
            print("Error: Can't open file:" + input_file)
            return False

        # Padd bootstrap if needed
        if ((len(bootstrap)%16) != 0):
            tmp = binascii.hexlify(bootstrap).decode('latin-1') + binascii.hexlify(get_random_bytes(16 - (len(bootstrap)%16))).decode('latin-1')
            bootstrap = binascii.unhexlify(tmp.encode('latin-1'))

        # Set bootstrap
        self.set_bootstrap(bootstrap)

        # intialize ciphered_bootstrap_bundle
        ciphered_bootstrap_bundle = ""

        if ((priv_key_file == None) and (certs_file == None) and (priv_key_pass == None)):
            ciphered_bootstrap_bundle = self.cmac_tag_bootstrap(key)
        else:
            ciphered_bootstrap_bundle = self.rsa_sign_bootstrap(key, priv_key_file, priv_key_pass, certs_file, hash_algo=hash_algo)

        if (ciphered_bootstrap_bundle != ""):
            tmp_file =open(output_file, 'wb')
            tmp_file.write(ciphered_bootstrap_bundle)
            tmp_file.close()
            return True
        else:
            return False

    def cipher_application(self, app_key, app_iv, app_cmac_key, input_file, output_file, no_header):
        """ Encrypt application with given keys. Add and encrypt header if needed
            :param app_key: key
            :param app_iv: IV
            :param app_cmac_key: CMAC Key
            :param input_file: file to be encrypted
            :param output_file: output file
            :param no_header: boolean specifies if header is needed
        """
        # ciphered header to be concatenated with application
        ciphered_hdr = ""
        # Read application
        app = open(input_file, "rb").read()

        # Padd application if needed
        if ((len(app)%16) != 0):
            app =  binascii.hexlify(app).decode('latin-1') + binascii.hexlify(get_random_bytes(16 - (len(app)%16))).decode('latin-1')
            app = binascii.unhexlify(app.encode('latin-1'))

        if(not(no_header)):
            len_app = hex(len(app))[2:].zfill(8)
            header = "55aa0000" + len_app[6:8] + len_app[4:6] + len_app[2:4] + len_app[0:2] + "00"*8
            cipher_aes = AES.new(binascii.unhexlify(app_key.encode('latin-1')), AES.MODE_CBC, binascii.unhexlify(app_iv.encode('latin-1')))
            ciphered_hdr = binascii.hexlify(cipher_aes.encrypt(binascii.unhexlify(header.encode('latin-1')))).decode("latin-1")

        cipher_aes = AES.new(binascii.unhexlify(app_key.encode('latin-1')), AES.MODE_CBC, binascii.unhexlify(app_iv.encode('latin-1')))
        ciphered_app = binascii.hexlify(cipher_aes.encrypt(app)).decode("latin-1")
        # compute CMAC
        aes_prf = aes_lib.AES_PRF()
        aes_prf.Set__K__(app_cmac_key)
        ciphered_app_cmac = aes_prf.CMAC_Generation(ciphered_app)

        result = ciphered_hdr + ciphered_app + ciphered_app_cmac
        open(output_file, 'wb').write(binascii.unhexlify(result.encode('latin-1')))
        return True


class sama5d2x_cipher(device_cipher):
    """ sama5d2x cipher class
        encrypt customer key with div rom key 2
        generate cust key message
    """
    def __init__(self):
        super().__init__()
        self.device_name = "sama5d2x"
        self.cust_key_length = None
        self.ctx = "be23a497"
        self.__L__ = "00000280"
        self.__padding__ = "00"
        self.deriv_iv = ""
        self.deriv_key = ""

    def cipher_customer_key(self, key, outputfile, cust_key_padding=None):
        """ Encrypt given customer key with div_rom_key2
            :param key: customer key to be encrypted, its length is 32 bytes or 48 bytes
            :param outputfile: path to the output file
        """
        # Check Cust Key length
        if (len(key) == 64):
            self.cust_key_length = 32
            padding = ""
        elif (len(key) == 96):
            self.cust_key_length = 48
            padding = "FF" * 16
        else:
            print("Error: Wrong key size.\n")
            return False
        self.generate_cust_message(key + padding, outputfile)
        return True

    def cipher_bootstrap(self, key, priv_key_file, priv_key_pass, certs_file, input_file, output_file, bypass_root_ca_verif=False):
        """ Encryptand sign a bootstrap program.

            The Customer Key must be identical to the one used in
            cipher_customer_key.

            In case of asymmetric signature, priv_key_fn and certs_fn contain
            the private key and the certificate chain file names. If the
            private key is encrypted, its password must be supplied in
            priv_key_pass with one the following formats (without quotes):
            - "pass:XXX"
            - "file:/path/to/password_file.txt"

            bootstrap_fn and output_fn are the filenames for the input and
            output files.

            The file size (including CMAC) is written in the 6th vector
            (i.e. at offset 20).

            Returns True if the file has been saved successfully, False
            otherwise.
        """
        try:
            if (len(key) == 96):
                fixed_input = key[64:]
                self.label = fixed_input[0:14]
                self.__padding__ = fixed_input[14:16]
                self.ctx = fixed_input[16:24]
                self.__L__ = fixed_input[24:32]
            elif(len(key) != 64):
                return False

            key_deriv_obj = aes_lib.Key_Derivation( __KeyI__ = key[0:32],
                                                        __L__ = self.__L__,
                                                        __Context__= self.ctx,
                                                        __Label__= self.label,
                                                        __IV__= key[32:64],
                                                        __Nb_Loops__ = 5,
                                                        __padding__ = self.__padding__)
            div_cust_key = key_deriv_obj.Compute_Derivations()
            if(self.cipher_and_sign_bootstrap(div_cust_key, priv_key_file, priv_key_pass, certs_file, input_file, output_file, hash_algo="SHA256")):
                return True
            else:
                return False
        except:
            return False

    def cipher_root_cert_hash(self, certs_file, output_file):
        """ Verify certificates chain then compute hash on the root certtificate public key
            :param
        """
        try:
            tmp_file = open(certs_file, 'r')
            certs = tmp_file.read()
            tmp_file.close()
            self.set_certificates(certs)
        except:
            print("Error: Can't read data from certificates file: '" + certs_file + "'")
            return False

        try:
            #if (not(self.verify_certificates())):
            #    return False

            # Compute hash on the root certificate's public key
            self.hash_root_cert_pub_key("SHA256")

            # Encrypt root ca hash with div rom key 2
            self.cipher_root_ca_hash(output_file)

            return True

        except:
            return False

class sama5d2nk_cipher(sama5d2x_cipher):
    """ sama5d3x cipher class
        encrypt customer key with div rom key 2
        generate cust key message
    """
    def __init__(self):
        super().__init__()
        self.device_name = "sama5d2x_nk"

class sama5d29_cipher(sama5d2x_cipher):
    """ sama5d29 cipher class
        encrypt customer key with div rom key 2
        generate cust key message
    """
    def __init__(self):
        super().__init__()
        self.device_name = "sama5d29"

class sama5d3x_cipher(device_cipher):
    """ sama5d3x cipher class
        encrypt customer key with div rom key 2
        generate cust key message
    """
    def __init__(self):
        super().__init__()
        self.device_name = "sama5d3x"
        self.cust_key_length = 24
        self.ctx = "fefefefe"
        self.__L__ = "00000280"
        self.__padding__ = "00"
        self.deriv_iv = "00"*16

    def cipher_customer_key(self, key, outputfile, cust_key_padding=None):
        """ Encrypt given customer key with div_rom_key2
            :param key: customer key to be encrypted, its length is 32 bytes or 48 bytes
            :param outputfile: path to the output file
        """
        # Check Cust Key length
        if (len(key) != 48):
            return False
        try:
            # hexify and unhexify to check if given padding is in hex format
            self.cust_key_padding = binascii.hexlify(binascii.unhexify(cust_key_padding.encode('uf-8'))).decode('latin-1').zfill(16)
        except:
            self.cust_key_padding = binascii.hexlify(get_random_bytes(8)).decode('latin-1').zfill(16)

        self.generate_cust_message(key + self.cust_key_padding, outputfile)
        return True

    def cipher_bootstrap(self, key, priv_key_file, priv_key_pass, certs_file, input_file, output_file, bypass_root_ca_verif=False):
        """ Encryptand sign a bootstrap program.

            The Customer Key must be identical to the one used in
            cipher_customer_key.

            In case of asymmetric signature, priv_key_fn and certs_fn contain
            the private key and the certificate chain file names. If the
            private key is encrypted, its password must be supplied in
            priv_key_pass with one the following formats (without quotes):
            - "pass:XXX"
            - "file:/path/to/password_file.txt"

            bootstrap_fn and output_fn are the filenames for the input and
            output files.

            The file size (including CMAC) is written in the 6th vector
            (i.e. at offset 20).

            Returns True if the file has been saved successfully, False
            otherwise.
        """
        try:
            key_deriv_obj = aes_lib.Key_Derivation( __KeyI__ = key[0:32],
                                                        __L__ = self.__L__,
                                                        __Context__= self.ctx,
                                                        __Label__= self.label,
                                                        __IV__= self.deriv_iv,
                                                        __Nb_Loops__ = 5,
                                                        __padding__ = self.__padding__)
            div_cust_key = key_deriv_obj.Compute_Derivations()
            if(self.cipher_and_sign_bootstrap(div_cust_key, priv_key_file, priv_key_pass, certs_file, input_file, output_file, hash_algo="SHA256")):
                return True
            else:
                return False
        except:
            return False

class sama5d3nk_cipher(sama5d3x_cipher):
    """ sama5d3x cipher class
        encrypt customer key with div rom key 2
        generate cust key message
    """
    def __init__(self):
        super().__init__()
        self.device_name = "sama5d3x_nk"

class sama5d4x_cipher(device_cipher):
    """ sama5d2x cipher class
        encrypt customer key with div rom key 2
        generate cust key message
    """
    def __init__(self):
        super().__init__()
        self.device_name = "sama5d4x"
        self.cust_key_length = 32
        self.ctx = "25844bb5"
        self.__L__ = "00000280"
        self.__padding__ = "00"
        self.deriv_iv = ""
        self.deriv_key = ""

    def cipher_customer_key(self, key, outputfile, cust_key_padding=None):
        """ Encrypt given customer key with div_rom_key2
            :param key: customer key to be encrypted, its length is 32 bytes or 48 bytes
            :param outputfile: path to the output file
        """
        # Check Cust Key length
        if (len(key) != 64):
            print("Error: Wrong key size.\n")
            return False
        self.generate_cust_message(key, outputfile)
        return True

    def cipher_bootstrap(self, key, priv_key_file, priv_key_pass, certs_file, input_file, output_file, bypass_root_ca_verif=False):
        """ Encryptand sign a bootstrap program.

            The Customer Key must be identical to the one used in
            cipher_customer_key.

            In case of asymmetric signature, priv_key_fn and certs_fn contain
            the private key and the certificate chain file names. If the
            private key is encrypted, its password must be supplied in
            priv_key_pass with one the following formats (without quotes):
            - "pass:XXX"
            - "file:/path/to/password_file.txt"

            bootstrap_fn and output_fn are the filenames for the input and
            output files.

            The file size (including CMAC) is written in the 6th vector
            (i.e. at offset 20).

            Returns True if the file has been saved successfully, False
            otherwise.
        """
        try:
            key_deriv_obj = aes_lib.Key_Derivation( __KeyI__ = key[0:32],
                                                        __L__ = self.__L__,
                                                        __Context__= self.ctx,
                                                        __Label__= self.label,
                                                        __IV__= key[32:64],
                                                        __Nb_Loops__ = 5,
                                                        __padding__ = self.__padding__)
            div_cust_key = key_deriv_obj.Compute_Derivations()
            if(self.cipher_and_sign_bootstrap(div_cust_key, priv_key_file, priv_key_pass, certs_file, input_file, output_file, hash_algo="SHA256")):
                return True
            else:
                return False
        except:
            return False

class sama7g5_cipher(device_cipher):
    """ sama7g5 cipher class
        encrypt customer key with div rom key 2
        generate cust key message
    """
    def __init__(self):
        super().__init__()
        self.device_name = "sama7g5"
        self.cust_key_length = None
        self.ctx = "00000000"
        self.__L__ = "00000280"
        self.__padding__ = "00"
        self.deriv_iv = ""
        self.deriv_key = ""
        self.max_rsa_sign_size = 4608 // 8

    def cipher_customer_key(self, key, outputfile, cust_key_padding=None):
        """ Encrypt given customer key with div_rom_key2
            :param key: customer key to be encrypted, its length is 32 bytes or 48 bytes
            :param outputfile: path to the output file
        """
        self.generate_cust_message(key, outputfile)
        return True

    def cipher_bootstrap(self, key, priv_key_file, priv_key_pass, certs_file, input_file, output_file, bypass_root_ca_verif=False):
        """ Encryptand sign a bootstrap program.

            The Customer Key must be identical to the one used in
            cipher_customer_key.

            In case of asymmetric signature, priv_key_fn and certs_fn contain
            the private key and the certificate chain file names. If the
            private key is encrypted, its password must be supplied in
            priv_key_pass with one the following formats (without quotes):
            - "pass:XXX"
            - "file:/path/to/password_file.txt"

            bootstrap_fn and output_fn are the filenames for the input and
            output files.

            The file size (including CMAC) is written in the 6th vector
            (i.e. at offset 20).

            Returns True if the file has been saved successfully, False
            otherwise.
        """
        try:
            if (len(key) != 160):
                print("Error: Wrong key size.\n")
                return False
            if(self.cipher_and_sign_bootstrap(key, priv_key_file, priv_key_pass, certs_file, input_file, output_file, hash_algo="")):
                return True
            else:
                return False
        except:
            return False

    def cipher_root_cert_hash(self, certs_file, output_file):
        """ Verify certificates chain then compute hash on the root certtificate public key
            :param
        """
        try:
            tmp_file = open(certs_file, 'r')
            certs = tmp_file.read()
            tmp_file.close()
        except:
            print("Error: Can't open " + certs_file)
            return False

        try:
            self.set_certificates(certs)

            #if (not(self.verify_certificates())):
            #    return False

            # Compute hash on the root certificate's public key
            self.hash_full_root_cert("SHA512")
        except:
            print("Error: Wrong certificate format.")
            return False

        try:
            # Encrypt root ca hash with div rom key 2
            self.cipher_root_ca_hash(output_file)

            return True

        except:
            print("Error: Error occured during encryption.")
            return False

class sam9x60_cipher(device_cipher):
    """ sam9x60 cipher class
        encrypt customer key with div rom key 2
        generate cust key message
    """
    def __init__(self):
        super().__init__()
        self.device_name = "sam9x60"
        self.cust_key_length = None
        self.ctx = "00000000"
        self.__L__ = "00000280"
        self.__padding__ = "00"
        self.deriv_iv = ""
        self.deriv_key = ""

    def cipher_customer_key(self, key, outputfile, cust_key_padding=None):
        """ Encrypt given customer key with div_rom_key2
            :param key: customer key to be encrypted, its length is 32 bytes or 48 bytes
            :param outputfile: path to the output file
        """
        self.generate_cust_message(key, outputfile)
        return True

    def cipher_bootstrap(self, key, priv_key_file, priv_key_pass, certs_file, input_file, output_file, bypass_root_ca_verif=False):
        """ Encryptand sign a bootstrap program.

            The Customer Key must be identical to the one used in
            cipher_customer_key.

            In case of asymmetric signature, priv_key_fn and certs_fn contain
            the private key and the certificate chain file names. If the
            private key is encrypted, its password must be supplied in
            priv_key_pass with one the following formats (without quotes):
            - "pass:XXX"
            - "file:/path/to/password_file.txt"

            bootstrap_fn and output_fn are the filenames for the input and
            output files.

            The file size (including CMAC) is written in the 6th vector
            (i.e. at offset 20).

            Returns True if the file has been saved successfully, False
            otherwise.
        """
        try:
            if (len(key) != 160):
                print("Error: Wrong key size.\n")
                return False
            if (self.cipher_and_sign_bootstrap(key, priv_key_file, priv_key_pass, certs_file, input_file, output_file, hash_algo="")):
                return True
            else:
                return False
        except:
            return False

    def cipher_root_cert_hash(self, certs_file, output_file):
        """ Verify certificates chain then compute hash on the root certtificate public key
            :param
        """
        try:
            tmp_file = open(certs_file, 'r')
            certs = tmp_file.read()
            tmp_file.close()

            self.set_certificates(certs)

            #if (not(self.verify_certificates())):
            #    return False

            # Compute hash on the root certificate's public key
            self.hash_root_cert_pub_key("SHA256")

            # Encrypt root ca hash with div rom key 2
            self.cipher_root_ca_hash(output_file)

            return True

        except:
            return False

class pic32cxmt_cipher(device_cipher):
    """ pic32cxmt cipher class
        encrypt customer key with div rom key 2
        generate cust key message
    """
    def __init__(self):
        super().__init__()
        self.device_name = "pic32cxmt"
        self.cust_key_length = None
        self.ctx = "00000000"
        self.__L__ = "00000280"
        self.__padding__ = "00"
        self.deriv_iv = ""
        self.deriv_key = ""
        self.idx_cmac_key = 64
        self.vector_8 = 0
        self.vector_9 = 0
        #biggest supported size for internal flash
        self.int_flash_limit = 0x200000
        self.max_rsa_sign_size = 4608 // 8

    def cipher_customer_key(self, key, outputfile, cust_key_padding=None):
        """ Encrypt given customer key with div_rom_key2
            :param key: customer key to be encrypted, its length is 134 bytes
            :param outputfile: path to the output file
        """
        self.generate_cust_message(key, outputfile)
        return True

    def update_sizes_vectors_cipher_bootstrap(self, key, sign_algo, signature_size, certificates_size):
        """ Compute the 8th and 9th vectors
            :param sign_algo: CMAC or RSA
            :param signature_size: signature size in bytes
            :param certificates_size: certificates size
        """
        # Use 0x200000 (2MB) as upper limit for bootstrap plus signature size
        # There are derivatives with 512KB and 1MB flash size also, but using a
        # limit of 2M will handle these derivatives also.
        if ((len(self.bootstrap) + signature_size) > self.int_flash_limit):
            sys.stderr.write("Error: data + signature exceeds %d bytes\n" % self.int_flash_limit)
            return False

        # Update the 8th vector = signature size + bootstrap length
        self.vector_8 = signature_size + len(self.bootstrap)

        # in case of public key, we need to update 9th vector with the certificates chain size
        if (sign_algo == "RSA"):
            if (certificates_size > self.int_flash_limit):
                sys.stderr.write("Error: certificate chain exceeds %d bytes\n" % self.int_flash_limit);
                return False
            self.vector_9 = certificates_size
            if self.get_bypass_root_ca_verif():
                self.vector_9 = self.vector_9 | (1<<31)

        # update vectors
        bootstrap_hex = binascii.hexlify(self.bootstrap).decode('latin-1')
        vect8_hex_msb = hex(self.vector_8)[2:].zfill(8)
        vect9_hex_msb = hex(self.vector_9)[2:].zfill(8)

        bootstrap_hex = bootstrap_hex[0:56] + vect8_hex_msb[6:8] + vect8_hex_msb[4:6] + vect8_hex_msb[2:4] + vect8_hex_msb[0:2] + \
            vect9_hex_msb[6:8] + vect9_hex_msb[4:6] + vect9_hex_msb[2:4] + vect9_hex_msb[0:2] + \
            bootstrap_hex[56 + 16:]

        self.bootstrap = binascii.unhexlify(bootstrap_hex.encode('latin-1'))

        self.ciphered_bootstrap = self.bootstrap

        return True


    def cipher_bootstrap(self, key, priv_key_file, priv_key_pass, certs_file, input_file, output_file, bypass_root_ca_verif=False):
        """ Sign a bootstrap program.

            The Customer Key must be identical to the one used in
            cipher_customer_key.

            In case of asymmetric signature, priv_key_fn and certs_fn contain
            the private key and the certificate chain file names. If the
            private key is encrypted, its password must be supplied in
            priv_key_pass with one the following formats (without quotes):
            - "pass:XXX"
            - "file:/path/to/password_file.txt"

            bootstrap_fn and output_fn are the filenames for the input and
            output files.

            The file size (including CMAC) is written in the 6th vector
            (i.e. at offset 20).

            Returns True if the file has been saved successfully, False
            otherwise.
        """
        try:
            self.set_bypass_root_ca_verif(bypass_root_ca_verif)
            if (len(key) != 288):
                print("Error: Wrong key size.\n")
                return False
            if (self.cipher_and_sign_bootstrap(key, priv_key_file, priv_key_pass, certs_file, input_file, output_file, hash_algo="")):
                return True
            else:
                return False
        except:
            return False

class sam9x7x_cipher(device_cipher):
    """ sam9x7x cipher class
        encrypt customer key payload with div rom key 2 and device's public key
    """
    def __init__(self):
        super().__init__()
        self.device_name = "sam9x7x_cipher"
        self.cust_key_length = None
        self.ctx = "00000000"
        self.__L__ = "00000280"
        self.__padding__ = "00"
        self.deriv_iv = ""
        self.deriv_key = ""
        self.modulus = 0x9688700abcb8fbc4c48723eac0d3208d2d808d8db086a0edd52aa3100b682d6d2eb22a4447c5adba2d90eb0a359aa36d509650c1759be99d855902ae0f615bfe2de505a734e244d114822fcbf96e31881827d8b2e9d22fbffcefb5a5f539a36053580c3b79975a4762448accd4bf73524754006d8783846afd00e2edd535dadf17f64d84ada648fbf0d36823f4a06a9a8fbe4cd2932ce90ef36caab79dec24addd7517a972c0058a3e5f98ada7a9990c97cb468969ea5d6ddbe9356d2d5712751653e299b262fc6f8150a60078ebc0f7401af880c8570195b293be5b7a45d6a247218d93fe692aece2e74b627970b060dd030ee605d6c40790fc768576dd4645
        self.pub_exp = 0x10001
        self.modulus_bytes_size = 256

    def cipher_customer_key(self, key, outputfile, cust_key_padding=None):
        """ Encrypt given customer key payload with div_rom_key2 and public key devices
            :param key: customer key to be encrypted, its length is 32 bytes or 48 bytes
            :param outputfile: path to the output file
        """
        # Generate customer message encrypted with AES Div ROM Key 2
        self.generate_cust_message(key, outputfile.replace('.cip', ('_aes.cip')))

        # Generate customer message encrypted with device's RSA Public Key
        self.generate_cust_key_payload(key, outputfile.replace('.cip', ('_rsa.cip')))
        return True

    def cipher_bootstrap(self, key, priv_key_file, priv_key_pass, certs_file, input_file, output_file, bypass_root_ca_verif=False):
        """ Encryptand sign a bootstrap program.

            The Customer Key must be identical to the one used in
            cipher_customer_key.

            In case of asymmetric signature, priv_key_fn and certs_fn contain
            the private key and the certificate chain file names. If the
            private key is encrypted, its password must be supplied in
            priv_key_pass with one the following formats (without quotes):
            - "pass:XXX"
            - "file:/path/to/password_file.txt"

            bootstrap_fn and output_fn are the filenames for the input and
            output files.

            The file size (including CMAC) is written in the 6th vector
            (i.e. at offset 20).

            Returns True if the file has been saved successfully, False
            otherwise.
        """
        try:
            if (len(key) != 160):
                print("Error: Wrong key size.\n")
                return False
            if (self.cipher_and_sign_bootstrap(key, priv_key_file, priv_key_pass, certs_file, input_file, output_file, hash_algo="")):
                return True
            else:
                return False
        except:
            return False

def generate_license_request(device, out_priv_file = "private_key.pem", passphrase = None, rsa_size=4096):
    """
        Generate license request for specific device:
            Generate RSA keys
            Save private key with or without passphrase
        :param device: device for which the license request will be generated
        :param output private key:
        :param passphrase: (optional) passphrase for private key pem file
        :param rsa_size: RSA key size (in bits)

    """
    priv_key_pass = get_passphrase(passphrase)
    if (priv_key_pass == False):
        return None
    application_id = 'SecureSamba_CK__'
    request_header = 'LIC_REQ___0350__'
    rsa_obj = rsa_lib.rsa()
    if (os.path.exists(out_priv_file)):
        rsa_obj.import_priv_key_from_pem(open(out_priv_file, 'rb').read(), passphrase = priv_key_pass)
    else:
        rsa_obj.generate_keys(rsa_size)
        rsa_obj.save_priv_key(out_priv_file, passphrase=priv_key_pass, format="PEM")
    public_key = rsa_obj.priv_key.publickey().export_key()
    req = application_id + request_header + device + "\n" + public_key.decode('latin-1')
    return req


def main():
    req = generate_license_request(device="sam9x6", out_priv_file = "private_key.pem", passphrase = None, rsa_size = 2048)
    print(req)

if __name__ == '__main__':
    main()
