# secrets-gen2

A flexible, configuration-driven utility for generating cryptographic keys and injecting them into various file types.

## Features

- Generate various types of cryptographic keys (P256, secp256k1, AES, random bytes, etc.)
- Inject generated keys into different file types (.toml, .env, .rs, .hex, etc.)
- Preserve file structure when updating existing files
- Automatic conversion between naming conventions (PascalCase, snake_case, SCREAMING_SNAKE_CASE, kebab-case)
- Configuration-driven approach using TOML

## Usage

```bash
# Generate keys and inject them into files based on the configuration
cargo run -- --config secrets-config.toml

# Dry run mode - generate keys but don't modify any files
cargo run -- --config secrets-config.toml --dry-run

# Generate keys and save them to a specific output directory
cargo run -- --config secrets-config.toml --output-dir ./generated-keys
```

## Configuration File Format

The configuration file is a TOML file with two main sections:

1. `keys` - Defines the cryptographic keys to generate
2. `injectors` - Defines where and how to inject the generated keys into files

### Keys Section

Each key is defined with a unique PascalCase name and has the following properties:

- `type` - The type of key to generate (P256, secp256k1, AES, random, etc.)
- `params` - Optional parameters specific to the key type

### Injectors Section

Each injector defines a file to inject keys into and has the following properties:

- `file` - The path to the file
- `type` - The type of file (determined by extension if not specified)
- `injections` - A list of injections to perform

Each injection has:

- `key` - The name of the key to inject (must match a key defined in the `keys` section)
- `name` - Optional name to use in the file (if not provided, the key name is converted based on file type)
- `format` - Optional format specifier for the key value

## Example Configuration Files

### Basic Example

```toml
# secrets-config.toml

[keys]
# P256 keypair for provisioning
ProvisioningKey = { type = "P256" }

# Random 32-byte key for FIDO
FidoPrivateKey = { type = "random", params = { length = 32 } }

# AES-256 key for encryption
EncryptionKey = { type = "AES", params = { bits = 256 } }

[injectors]
# Inject into provision.toml
[[injectors.files]]
file = "~/dev/config/provision.toml"
type = "toml"
injections = [
  { key = "ProvisioningKey", format = "private-hex" },
  { key = "FidoPrivateKey", name = "fido-private-key" },
]

# Inject into .env file
[[injectors.files]]
file = "~/dev/sc-server/.env"
type = "env"
injections = [
  { key = "ProvisioningKey", format = "public-hex", name = "SC_PROVISIONING_PUBKEY" },
  { key = "EncryptionKey", name = "$screaming-snake" },
]
```

### Complex Example with Multiple Key Types and File Formats

```toml
# complex-config.toml

[keys]
# P256 keypair for provisioning
ProvisioningKey = { type = "P256" }

# secp256k1 keypair for blockchain
BlockchainKey = { type = "secp256k1" }

# Random 32-byte key for FIDO
FidoPrivateKey = { type = "random", params = { length = 32 } }

# AES-256 key for encryption
EncryptionKey = { type = "AES", params = { bits = 256 } }

# Random entropy for bootloader
BootloaderEntropy = { type = "random", params = { length = 32, count = 2 } }

[injectors]
# Inject into provision.toml
[[injectors.files]]
file = "~/dev/config/provision.toml"
type = "toml"
injections = [
  { key = "ProvisioningKey", format = "private-hex", name = "provisioning-secret" },
  { key = "ProvisioningKey", format = "public-hex", name = "sc-server-pubkey" },
  { key = "FidoPrivateKey", name = "fido-private-key" },
]

# Inject into .env file
[[injectors.files]]
file = "~/dev/sc-server/.env"
type = "env"
injections = [
  { key = "ProvisioningKey", format = "public-hex", name = "SC_PROVISIONING_PUBKEY" },
  { key = "EncryptionKey", name = "SC_ENCRYPTION_KEY" },
  { key = "BootloaderEntropy", format = "csv", name = "SC_EXTRA_ENTROPY" },
]

# Inject into Rust file
[[injectors.files]]
file = "~/dev/keyOS/src/constants.rs"
type = "rs"
injections = [
  { key = "EncryptionKey", format = "hex-array", inject = "EncryptionKey" },
  { key = "BlockchainKey", format = "public-compressed-hex", replace = "BlockchainPublicKey" },
]

# Save keys to individual files
[[injectors.files]]
file = "~/dev/keyOS/generated-keys/provisioning-private.pem"
type = "raw"
injections = [
  { key = "ProvisioningKey", format = "private-pem" },
]

[[injectors.files]]
file = "~/dev/keyOS/generated-keys/provisioning-public.pem"
type = "raw"
injections = [
  { key = "ProvisioningKey", format = "public-pem" },
]
```

## Supported Key Types

### P256

Generates an ECDSA P-256 keypair.

```toml
MyP256Key = { type = "P256" }
```

Formats:

- `private-hex` - Private key in hex format (32 bytes)
- `public-hex` - Public key in uncompressed hex format (65 bytes with 0x04 prefix)
- `public-compressed-hex` - Public key in compressed hex format (33 bytes)
- `private-pem` - Private key in PKCS#8 PEM format (BEGIN PRIVATE KEY)
- `ec-private-pem` - Private key in traditional EC PEM format (BEGIN EC PRIVATE KEY)
- `public-pem` - Public key in PEM format

### secp256k1

Generates a secp256k1 keypair (used in Bitcoin and other cryptocurrencies).

```toml
MySecp256k1Key = { type = "secp256k1" }
```

Formats:

- `private-hex` - Private key in hex format (32 bytes)
- `public-hex` - Public key in uncompressed hex format (65 bytes with 0x04 prefix)
- `public-compressed-hex` - Public key in compressed hex format (33 bytes)
- `private-pem` - Private key in PKCS#8 PEM format (BEGIN PRIVATE KEY)
- `ec-private-pem` - Private key in traditional EC PEM format (BEGIN EC PRIVATE KEY)
- `public-pem` - Public key in PEM format

### AES

Generates an AES key.

```toml
MyAesKey = { type = "AES", params = { bits = 256 } }
```

Parameters:

- `bits` - Key size in bits (128, 192, or 256)

Formats:

- `hex` - Key in hex format
- `base64` - Key in base64 format

### Random

Generates random bytes.

```toml
# Generate a single 32-byte random value
MyRandomKey = { type = "random", params = { length = 32 } }

# Generate two separate 32-byte random values
BootloaderEntropy = { type = "random", params = { length = 32, count = 2 } }
```

Parameters:

- `length` - Length of each random value in bytes
- `count` - Number of separate random values to generate (default: 1)

When `count` is greater than 1, multiple separate random values are generated. This is useful for cases like bootloader entropy where you need multiple independent random values.

Formats:

- `hex` - Random bytes in hex format (default)
- `base64` - Random bytes in base64 format
- `csv` - Multiple random values as comma-separated hex strings (when count > 1)

## Supported File Types

### TOML (.toml)

Injects keys into TOML files. Keys are automatically converted to kebab-case if no name is provided.

### Environment (.env)

Injects keys into .env files. Keys are automatically converted to SCREAMING_SNAKE_CASE if no name is provided.

### Rust (.rs)

Injects keys into Rust files using begin and end markers to define the injection region.

Injection attributes:

- `inject` - Specifies the marker name to look for in the file

Example:

```toml
injections = [
  { key = "EncryptionKey", format = "hex-array", inject = "EncryptionKey" },
  { key = "BlockchainKey", format = "public-compressed-hex", inject = "BlockchainPublicKey" },
]
```

In your Rust file:

```rust
// Constants for encryption
// @Inject:Begin EncryptionKey
[0; 32]
// @Inject:End EncryptionKey

// Public key for blockchain
// @Inject:Begin BlockchainPublicKey
"0000000000000000000000000000000000000000000000000000000000000000"
// @Inject:End BlockchainPublicKey
```

After injection, the file would look like:

```rust
// Constants for encryption
// @Inject:Begin EncryptionKey
[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20]
// @Inject:End EncryptionKey

// Public key for blockchain
// @Inject:Begin BlockchainPublicKey
"04a5d01b633ef5c4fdbbb2fad7a7cf14bb0fc92457d961ec379ebef5ad532e537321a1084edfe580eb7c14176299ac2ab7b450ec60bc5e32f1ea564c88dbdbae62"
// @Inject:End BlockchainPublicKey
```

The markers are preserved in the file, allowing future runs to find and update the values again. Everything between the begin and end markers is replaced with the new value.

### Raw

Writes the key value directly to the file without any processing.

## Naming Conventions

The following naming conventions are supported:

- `$pascal` - PascalCase (e.g., MyTestKey)
- `$camel` - camelCase (e.g., myTestKey)
- `$snake` - snake_case (e.g., my_test_key)
- `$screaming-snake` - SCREAMING_SNAKE_CASE (e.g., MY_TEST_KEY)
- `$kebab` - kebab-case (e.g., my-test-key)

If no name is provided, the key name is automatically converted based on the file type:

- TOML files: kebab-case
- .env files: SCREAMING_SNAKE_CASE
- Rust files: snake_case

## Development

To build the project:

```bash
cargo build
```

To run tests:

```bash
cargo test
```
