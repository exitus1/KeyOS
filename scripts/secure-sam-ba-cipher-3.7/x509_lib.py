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
from OpenSSL.crypto import FILETYPE_PEM, load_certificate
from OpenSSL.crypto import X509Store, X509StoreContext, FILETYPE_PEM
from cryptography.hazmat.primitives import serialization
from Crypto.Hash import SHA256, SHA512
import binascii

class x509_rsa(object):
    """ The x509 object contains certificates management
    """
    def __init__(self):
        self.cert = None
        self.pub_key = None
        self.signature_hash_algo = None
        self.sha_list = {"SHA256": SHA256, "SHA512":SHA512}

    def set_cert_pem(self, pem_data):
        """ get certificate from pem data
        """
        self.cert = load_certificate(FILETYPE_PEM, pem_data)

    def extract_pub_key(self):
        """ extract public key from certificate
        """
        self.pub_key = self.cert.to_cryptography().public_key()

    def get_cert_der(self):
        """ Get certificate in DER format
        """
        return self.cert.to_cryptography().public_bytes(serialization.Encoding.DER)

    def get_public_key_bits(self):
        """ Get public key bits number
        """
        return self.cert.to_cryptography().public_key().key_size

    def extract_signature_hash_algo(self):
        """ extract signature hash algorithm from certificate
        """
        signature_algo = self.cert.get_signature_algorithm().decode('latin-1')
        self.signature_hash_algo = signature_algo[:signature_algo.find("With")].upper()

    def compute_hash_on_pub_key(self, sha_algo="SHA256"):
        """ Compute hash on the public key
            :param sha_algo: Hash algorithm
        """
        n = hex(self.pub_key.public_numbers().n)[2:].zfill(self.pub_key.key_size // 4)
        e = hex(self.pub_key.public_numbers().e)[2:].zfill(6)
        data_to_hash = n + e

        pub_key_hash = self.sha_list[sha_algo].new(binascii.unhexlify(data_to_hash.encode('latin-1')))
        return binascii.hexlify(pub_key_hash.digest()).decode('utf8')

    def get_tbscertificate(self):
        """ Get TBS Certificate from certificate in der format
        """
        return self.cert.to_cryptography().tbs_certificate_bytes

    def get_tbscertificate(self):
        """ Get signature certificate in der format
        """
        return self.cert.to_cryptography().signature

def main():
    tmp_file = open("root-ca.crt", "rb")
    pem_cert = tmp_file.read()
    tmp_file.close()
    x509_obj = x509_rsa()
    x509_obj.set_cert_pem(pem_cert)
    x509_obj.extract_pub_key()
    print(x509_obj.extract_signature_hash_algo())
    print(x509_obj.compute_hash_on_pub_key())

    print("end")

if __name__ == '__main__':
    main()
