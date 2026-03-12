// Example Rust file with injection markers for different key types and formats

// Random key in hex array format
const RANDOM_KEY: [u8; 32] =
// @Inject:Begin RandomKey
[0xd8, 0x2d, 0xe2, 0x24, 0x87, 0xf9, 0xc0, 0xbd, 0x55, 0xae, 0x6c, 0xeb, 0x61, 0x8d, 0x7c, 0x20, 0x2c, 0x1b, 0x5a, 0x70, 0x17, 0x84, 0x51, 0x03, 0x7b, 0x9a, 0x7e, 0xcb, 0xeb, 0x7e, 0xad, 0xbf]
// @Inject:End RandomKey
;

// P256 public key in uncompressed hex format (130 hex characters)
const P256_PUBLIC_KEY: &str =
// @Inject:Begin P256PublicKey
04dd4eef55e9bc152229f93c06efdca61d63047e48eb7f96ba681db41cdb22e28b09fb6de955b0aacdd4dbee58a3f59ea7e3d29c8d978358e24b300f365619a91b
// @Inject:End P256PublicKey
;

// P256 public key in compressed hex format (66 hex characters)
const P256_COMPRESSED_PUBLIC_KEY: &str =
// @Inject:Begin P256CompressedPublicKey
03dd4eef55e9bc152229f93c06efdca61d63047e48eb7f96ba681db41cdb22e28b
// @Inject:End P256CompressedPublicKey
;

// secp256k1 private key in hex format
const SECP256K1_PRIVATE_KEY: &str =
// @Inject:Begin Secp256k1PrivateKey
e4ec324685911baa49f92e201828d6a6653ac95f9f3378d7ae6beb07ab1bb408
// @Inject:End Secp256k1PrivateKey
;

// AES key in hex format
const AES_KEY: &str =
// @Inject:Begin AesKey
a0f75c10adbbc45764b8569a9fa07e5f2a481cc4de179a19bda85a2befce6485
// @Inject:End AesKey
;

// Multiple random values as a string array
const MULTIPLE_RANDOM_VALUES: [&str; 3] =
// @Inject:Begin MultipleRandomValues
["86d94551735bad09e9c9e3805d6531d9", "dfdf36dd99720e93d338fdefe2b1ae0a", "69e40e888f28fdfff167f1516344799a"]
// @Inject:End MultipleRandomValues
;
