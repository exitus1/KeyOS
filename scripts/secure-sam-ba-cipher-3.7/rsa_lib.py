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
import sys
from Crypto.PublicKey import RSA
from Crypto.Signature.pkcs1_15 import PKCS115_SigScheme
from Crypto.Hash import SHA256, SHA512
from Crypto.Cipher import AES, PKCS1_OAEP
from Crypto.Random import get_random_bytes
import binascii

class rsa(object):
    """ The rsa object contains PKI management and file's
        signature/verification
    """
    def __init__(self):
        self.priv_key = None
        self.pub_key = None
        self.sha_list = {"SHA256": SHA256, "SHA512":SHA512}

    def generate_keys(self, rsa_size = 2048):
        """ Generate new keys, public and private
        """
        self.priv_key = RSA.generate(rsa_size)
        return True

    def get_pub_key(self):
        """ Get public key
        """
        return self.pub_key

    def save_priv_key(self, priv_key_file, passphrase = None, format="PEM"):
        """ Save private key into PEM or DER file
            :param priv_key_file: the path to the file to be created
            :param passphrase: optional passphrase to be used to encrypt private key.
                The encryption is allowed only for PEM format
            :param format: "PEM" or "DER"
        """
        private_key = None

        if (passphrase != None):
            if (format == "DER"):
                sys.stderr.write("Error: PKCS#1 private key encryption is allowed only for PEM format.\n")
                return False
            try:
                private_key = self.priv_key.export_key(format=format, passphrase=passphrase, pkcs=8,
                              protection="scryptAndAES128-CBC")
            except:
                sys.stderr.write("Error: RSA keys not found.\n")
                return False
        else:
            private_key = self.priv_key.export_key()

        try:
            file_out = open(priv_key_file, "wb")
            file_out.write(private_key)
            file_out.close()
        except:
            sys.stderr.write("Error: can't open file %s.\n"%(priv_key_file))
            return False
        return True


    def save_pub_key(self, pub_key_file, format="PEM"):
        """ Save public key into PEM or DER file
            :param pub_key_file: the path to the file to be created
            :param format: "PEM" or "DER"
        """
        try:
            public_key = self.priv_key.publickey().export_key(format=format)
            file_out = open(pub_key_file, "wb")
            file_out.write(public_key)
            file_out.close()
        except:
            sys.stderr.write("Error: can't save public key into %s.\n"%(pub_key_file))
            return False
        return True

    def import_priv_key_from_pem(self, key_pem, passphrase = None):
        """ Import RSA private key from PEM file
            :param key_pem: text in PEM format
            :param passphrase: optional passphrase
        """
        try:
            self.priv_key = RSA.import_key(key_pem, passphrase=passphrase)
        except:
            sys.stderr.write("Error: can't import private key from file %s.\n"%(key_pem))
            return False
        return True

    def import_pub_key(self, pub_key):
        """ Import RSA public key from DER or PEM
            :param pub_key: public key in DER or PEM format
        """
        try:
            self.pub_key = RSA.import_key(pub_key)
        except:
            sys.stderr.write("Error: can't import keys from file %s.\n"%(pub_key))
            return False
        return True

    def sign_message_pkcs1_v1_5(self, message, sha_algo="SHA256"):
        """ Sign the message using the PKCS#1 v1.5 signature scheme (RSASP1)
            :param message: message to sign
            :param sha_algo: hash algorithm used in signature
        """
        signature = 0
        hash_obj = self.sha_list[sha_algo].new(message)
        try :
            signer = PKCS115_SigScheme(self.priv_key)
            signature = signer.sign(hash_obj)
        except:
            sys.stderr.write("Error: RSA sign error, check private key\n")
        return signature

    def verify_signatuture_pkcs1_v1_5(self, message, signature, sha_algo="SHA256"):
        """ Verify signature using PKCS#1 v1.5 signature (RSAVP1)
            :param message: input message
            :param sha_algo: hash algorithm used in signature
        """
        hash_obj = self.sha_list[sha_algo].new(message)
        try :
            verifier = PKCS115_SigScheme(self.pub_key)
            verifier.verify(hash_obj, signature)
        except:
            sys.stderr.write("Error: Signature is invalid\n")
            return False
        return True

    def encrypt_file(self, in_plain_file, out_file, receiver_pub_key_file):
        """ Encrypt file using a random session key
            The session key is encrypted using receiver's public key
            :param in_plain_file:
        """
        recipient_key = RSA.import_key(open(receiver_pub_key_file).read())
        session_key = get_random_bytes(32)
        nonce = get_random_bytes(20)

        # Encrypt the session key with the public RSA key
        cipher_rsa = PKCS1_OAEP.new(recipient_key)
        enc_session_key = cipher_rsa.encrypt(session_key)

        # Read file to be encrypted
        tmp_file = open(in_plain_file, "rb")
        data = tmp_file.read()
        tmp_file.close()

        # Encrypt the data with the AES session key
        tmp_file = open(out_file, "wb")
        cipher_aes = AES.new(session_key, AES.MODE_GCM, nonce)
        ciphertext, tag = cipher_aes.encrypt_and_digest(data)
        [ tmp_file.write(x) for x in (enc_session_key, cipher_aes.nonce, tag, ciphertext) ]
        tmp_file.close()

    def encrypt_data(self, in_plain_data, receiver_pub_key):
        """ Encrypt file using a random session key
            The session key is encrypted using receiver's public key
            :param in_plain_file: input plain data
            :param receiver_pub_key: receiver's public key, used to encrypt session key
        """
        recipient_key = RSA.import_key(receiver_pub_key)
        session_key = get_random_bytes(32)
        nonce = get_random_bytes(20)

        # Encrypt the session key with the public RSA key
        cipher_rsa = PKCS1_OAEP.new(recipient_key)
        enc_session_key = binascii.hexlify(cipher_rsa.encrypt(session_key))

        # Encrypt the data with the AES session key
        cipher_aes = AES.new(session_key, AES.MODE_GCM, nonce)
        ciphertext, tag = cipher_aes.encrypt_and_digest(in_plain_data)
        ciphertext = binascii.hexlify(ciphertext)
        nonce = binascii.hexlify(cipher_aes.nonce)
        tag = binascii.hexlify(tag)

        return (enc_session_key, nonce, tag, ciphertext)

    def decrypt_data(self, enc_session_key, nonce, tag, ciphertext):
        """ decrypt file using private key
            The session key is encrypted using receiver's public key
        """

        # Decrypt the session key with the private RSA key
        cipher_rsa = PKCS1_OAEP.new(self.priv_key)
        session_key = cipher_rsa.decrypt(binascii.unhexlify(enc_session_key))

        # Decrypt the data with the AES session key
        cipher_aes = AES.new(session_key, AES.MODE_GCM, binascii.unhexlify(nonce))
        data = cipher_aes.decrypt_and_verify(binascii.unhexlify(ciphertext), binascii.unhexlify(tag))
        return binascii.hexlify(data)


#############
#   Tests   #
#############
def main():
    rsa_obj = rsa()
    rsa_obj.generate_keys(2048)
    rsa_obj.save_priv_key("priv.pem", passphrase="password", format="PEM")
    tmp_file = open("priv.pem", "rb")
    rsa_obj.import_priv_key_from_pem(tmp_file.read(), passphrase="password")
    signature = rsa_obj.sign_message_pkcs1_v1_5(b'abc', sha_algo="SHA256")
    tmp_file.close()
    rsa_obj.save_pub_key("pub.pem", format="PEM")
    rsa_obj.save_pub_key("pub.der", format="DER")
    tmp_file = open("pub.pem", "rb")
    rsa_obj.import_pub_key(tmp_file.read())
    tmp_file.close()
    tmp_file = open("pub.der", "rb")
    rsa_obj.import_pub_key(tmp_file.read())
    tmp_file.close()
    signature = rsa_obj.sign_message_pkcs1_v1_5(b'abc', sha_algo="SHA256")
    result = rsa_obj.verify_signatuture_pkcs1_v1_5(b'abc', signature, sha_algo="SHA256")
    print(result)

    (enc_session_key, nonce, tag, ciphertext) = rsa_obj.encrypt_data(open("plain.txt", "rb").read(), open("pub.pem", "rb").read())
    res = rsa_obj.decrypt_data(enc_session_key, nonce, tag, ciphertext)
    print(res)

    print("end")

if __name__ == '__main__':
    main()






















