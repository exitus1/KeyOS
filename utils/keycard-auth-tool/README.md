<!--
SPDX-FileCopyrightText: 2025 Foundation Devices, Inc. <hello@foundation.xyz>
SPDX-License-Identifier: GPL-3.0-or-later
-->

# KeyCard Authenticity Provisioning Tool

A Rust CLI tool for provisioning NFC KeyCards (NTAG 216) with HMAC-256 authentication for the Foundation KeyCard system.

## Features

- **Continuous Operation**: Runs continuously, waiting for NFC cards to be presented
- **HMAC-256 Authentication**: Computes HMAC using SHA256("Foundation KeyCard" || UID || data)
- **CBOR Data Structure**: Stores data in CBOR format with HMAC and data fields
- **NDEF Record Writing**: Writes data as NDEF records to NTAG 216 cards
- **Colored Console Output**: Green for success, red for errors, with timestamps
- **Card Verification**: Reads back written data to verify successful programming
- **Duplicate Detection**: Checks if cards are already provisioned to prevent overwriting
- **Erase Mode**: Clear NDEF data from cards to reset them to unpprovisioned state

## Requirements

- **Hardware**: ACR122U NFC reader (or compatible PC/SC reader)
- **Cards**: NTAG 216 NFC cards
- **OS**: macOS, Linux, or Windows with PC/SC support

## Installation

1. Clone the repository:

```bash
git clone <repository-url>
cd keycard-auth-tool
```

2. Build the application:

```bash
cargo build --release
```

## Configuration

Create a `config.toml` file with your secret key:

```toml
# Configuration file for keycard-auth-tool

# The secret key used for HMAC computation (32 bytes in hex format, no 0x prefix)
keycard-authenticity-secret = "2d1261ba057bfa10f2262e3750b776130bffc17bcf7f5f2fb5cd2e704cf89cae"
```

**Important**: The secret key must be exactly 64 hexadecimal characters (32 bytes).

## Usage

Run the tool with your configuration file:

```bash
# Provisioning mode (default)
cargo run -- --config config.toml
# or
./target/release/keycard-auth-tool --config config.toml

# Erase mode - clears NDEF data from cards
cargo run -- --config config.toml --erase
# or
./target/release/keycard-auth-tool --config config.toml --erase
```

### Command Line Options

- `-c, --config <CONFIG>`: Path to the configuration file (required)
- `-e, --erase`: Erase mode - Clear NDEF data from cards instead of provisioning them
- `-h, --help`: Show help information
- `-V, --version`: Show version information

## Operation

### Provisioning Mode (Default)

1. **Startup**: The tool loads the configuration and displays startup messages
2. **Card Detection**: Continuously waits for NFC cards to be presented
3. **Provisioning**: For each new card:
   - Reads the card's UID
   - Checks if already provisioned (prevents overwriting)
   - Computes HMAC-256 for empty data
   - Creates CBOR structure with HMAC and empty data
   - Writes NDEF record to the card
   - Verifies the write was successful
   - Displays success/error message with timestamp and UID

### Erase Mode (`--erase`)

1. **Startup**: Displays warning about erase mode
2. **Card Detection**: Continuously waits for NFC cards to be presented
3. **Erasing**: For each card:
   - Reads the card's UID
   - Checks existing NDEF data
   - Writes empty NDEF structure (03 00 FE 00)
   - Clears additional pages (5-29) with zeros
   - Verifies the erase was successful
   - Displays success message with "ERASED!" status

### Output Examples

**Successful Provisioning:**

```
--------------------------------------------------------------------------------
Jul 1, 2025: 12:18:42 - 0xDE 0xAB 0x12 0x34 0x56 0xEB 0x27 - OK!
```

**Successful Erase:**

```
--------------------------------------------------------------------------------
Jul 1, 2025: 12:25:10 - 0x04 0x5B 0x36 0xAA 0xAF 0x1D 0x90 - ERASED!
```

**Error Cases:**

```
--------------------------------------------------------------------------------
Jul 1, 2025: 12:18:42 - 0xDE 0xAB 0x12 0x34 0x56 0xEB 0x27 - ERROR!
Card is already provisioned!
```

```
--------------------------------------------------------------------------------
Jul 1, 2025: 12:18:42 - ERROR!
Card processing error: Unable to read UID from card
```

## Technical Details

### HMAC Computation

```
H = SHA256("Foundation KeyCard" || UID || data)
MAC = HMAC(secret_key, H)
```

### Data Structure

The tool creates a CBOR-encoded structure:

```rust
struct KeycardData {
   device_id: [u8; 32],
   seed_fingerprint: [u8; 32],
   seed_shamir_share: Vec<u8>,
   seed_shamir_share_index: usize,
   part_of_magic_backup: bool,
   hmac: [u8; 32],    // HMAC-256 result
}
```

### NDEF Format

- Uses external type record: `cbor.io:cbor`
- Stores CBOR data as the NDEF payload
- Writes to NTAG 216 starting at page 4

### Card Memory Layout (NTAG 216)

- Pages 0-1: UID
- Pages 2-3: Lock bytes and capability container
- Pages 4+: User data (NDEF records)
- Page 225: Last user page

## Error Handling

The tool handles various error conditions:

- No NFC reader found
- No card present
- Card read/write failures
- Invalid configuration
- Already provisioned cards
- Verification failures

## Development

### Dependencies

- `clap`: Command-line argument parsing
- `pcsc`: PC/SC smart card interface
- `serde` + `serde_cbor`: CBOR serialization
- `hmac` + `sha2`: Cryptographic functions
- `chrono`: Timestamp formatting
- `colored`: Console output coloring
- `hex`: Hexadecimal encoding/decoding
- `toml`: Configuration file parsing

### Building

```bash
cargo build          # Debug build
cargo build --release # Release build
```

### Testing

```bash
cargo test
```
