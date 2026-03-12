// Example Rust file with injection markers

// Constants for FIDO
const FIDO_PRIVATE_KEY: [u8; 32] = 
// @Inject:Begin FidoPrivateKey
[0x45, 0x49, 0xdc, 0x18, 0x24, 0xd6, 0x61, 0xdd, 0x87, 0x60, 0xa9, 0x1c, 0x91, 0x85, 0x53, 0x92, 0x36, 0x0a, 0x06, 0xee, 0xbd, 0x3f, 0xa1, 0x90, 0xf0, 0x79, 0x64, 0x56, 0x0d, 0x94, 0x34, 0x12]
// @Inject:End FidoPrivateKey
;

// Public key for provisioning
const PROVISIONING_PUBLIC_KEY: &str = 
// @Inject:Begin ProvisioningPublicKey
04790375290731cea7c185e3a958537fd746415085c92dda12c0d908d9002eca967b06cc400089d1d2cbeb38b4be84c62e6891fc94b595dcecee3ef37c942b84e9
// @Inject:End ProvisioningPublicKey
;
