# Implementation Plan for secrets-gen2

## Project Structure

```
utils/secrets-gen2/
├── Cargo.toml
├── src/
│   ├── main.rs                 # Entry point and CLI handling
│   ├── config.rs               # Configuration file parsing
│   ├── key_generation/         # Key generation modules
│   │   ├── mod.rs              # Module exports
│   │   ├── p256.rs             # P256 key generation
│   │   ├── secp256k1.rs        # secp256k1 key generation
│   │   ├── aes.rs              # AES key generation
│   │   └── random.rs           # Random bytes generation
│   ├── injectors/              # File injection modules
│   │   ├── mod.rs              # Module exports
│   │   ├── toml.rs             # TOML file injection
│   │   ├── env.rs              # .env file injection
│   │   ├── rs.rs               # Rust file injection
│   │   └── raw.rs              # Raw file injection
│   ├── naming.rs               # Naming convention utilities
│   └── error.rs                # Error handling
├── README.md                   # Documentation
└── example-config.toml         # Example configuration
```

## Implementation Steps

### 1. Project Setup

1. Create a new Rust project in the `utils/secrets-gen2` directory
2. Set up the necessary dependencies in `Cargo.toml`
3. Create the basic file structure

### 2. Configuration Parsing

1. Define the configuration file structure in `config.rs`
2. Implement parsing of the TOML configuration file
3. Create data structures to represent keys and injectors

### 3. Key Generation

1. Implement a trait for key generation in `key_generation/mod.rs`
2. Implement key generation for each supported key type:
   - P256 keys
   - secp256k1 keys
   - AES keys
   - Random bytes
3. Implement formatting functions for each key type

### 4. Naming Conventions

1. Implement functions to convert between naming conventions:
   - PascalCase
   - camelCase
   - snake_case
   - SCREAMING_SNAKE_CASE
   - kebab-case
2. Implement automatic naming based on file type

### 5. File Injectors

1. Implement a trait for file injection in `injectors/mod.rs`
2. Implement file injection for each supported file type:
   - TOML files
   - .env files
   - Rust files
   - Raw files
3. Ensure non-destructive updates for each file type

### 6. CLI Interface

1. Implement command-line argument parsing
2. Add support for dry-run mode
3. Add support for specifying output directory

### 7. Main Program Flow

1. Parse command-line arguments
2. Parse configuration file
3. Generate keys
4. Inject keys into files
5. Provide clear output to the user

### 8. Testing

1. Add unit tests for key generation
2. Add unit tests for naming conventions
3. Add unit tests for file injection
4. Add integration tests

### 9. Documentation

1. Update README.md with final details
2. Add code documentation

## Dependencies

- `clap` - Command-line argument parsing
- `serde` - Serialization/deserialization
- `toml` - TOML file parsing
- `anyhow` - Error handling
- `thiserror` - Error definitions
- `p256` - P256 key generation
- `k256` - secp256k1 key generation
- `rand` - Random number generation
- `hex` - Hex encoding/decoding
- `base64` - Base64 encoding/decoding
- `regex` - Regular expressions for file parsing
- `dirs` - Home directory resolution

## Detailed Component Specifications

### Key Generation Trait

```rust
trait KeyGenerator {
    fn generate(&self, params: Option<&toml::Value>) -> Result<Box<dyn Key>>;
}

trait Key {
    fn format(&self, format: &str) -> Result<String>;
}
```

### File Injector Trait

```rust
trait FileInjector {
    fn inject(&self, file_path: &Path, injections: &[Injection], keys: &HashMap<String, Box<dyn Key>>, dry_run: bool) -> Result<()>;
}

struct Injection {
    key: String,
    name: Option<String>,
    format: Option<String>,
    inject: Option<String>,
    replace: Option<String>,
}
```

### Naming Convention Functions

```rust
fn to_pascal_case(s: &str) -> String;
fn to_camel_case(s: &str) -> String;
fn to_snake_case(s: &str) -> String;
fn to_screaming_snake_case(s: &str) -> String;
fn to_kebab_case(s: &str) -> String;
fn convert_name(name: &str, convention: NamingConvention, original: &str) -> String;
```

### Configuration Structures

```rust
struct Config {
    keys: HashMap<String, KeyConfig>,
    injectors: InjectorConfig,
}

struct KeyConfig {
    key_type: String,
    params: Option<toml::Value>,
}

struct InjectorConfig {
    files: Vec<FileConfig>,
}

struct FileConfig {
    file: String,
    file_type: Option<String>,
    injections: Vec<InjectionConfig>,
}

struct InjectionConfig {
    key: String,
    name: Option<String>,
    format: Option<String>,
    inject: Option<String>,
    replace: Option<String>,
}
```

## Implementation Details

### Key Generation

1. **P256 Keys**:

   - Use the `p256` crate to generate P256 keypairs
   - Implement formatting functions for hex, compressed hex, and PEM formats

2. **secp256k1 Keys**:

   - Use the `k256` crate to generate secp256k1 keypairs
   - Implement formatting functions for hex, compressed hex, and PEM formats

3. **AES Keys**:

   - Generate random bytes of the specified length
   - Implement formatting functions for hex and base64 formats

4. **Random Bytes**:
   - Generate random bytes using the `rand` crate
   - Support generating multiple random values
   - Implement formatting functions for hex, base64, and CSV formats

### File Injection

1. **TOML Files**:

   - Parse the TOML file using the `toml` crate
   - Update the specified keys
   - Write the updated TOML back to the file, preserving structure

2. **.env Files**:

   - Parse the .env file line by line
   - Update the specified keys
   - Write the updated .env file back, preserving structure and comments

3. **Rust Files**:

   - Parse the Rust file line by line
   - Look for marker strings specified in the `inject` or `replace` attributes
   - Inject or replace code based on the attribute type
   - Write the updated Rust file back, preserving structure

4. **Raw Files**:
   - Write the formatted key value directly to the file

### Naming Conventions

1. Implement functions to detect the current naming convention of a string
2. Implement functions to convert between naming conventions
3. Implement automatic naming based on file type:
   - TOML files: kebab-case
   - .env files: SCREAMING_SNAKE_CASE
   - Rust files: snake_case

### Error Handling

1. Define custom error types for different failure scenarios
2. Provide clear error messages to the user
3. Use the `anyhow` crate for error propagation

## Testing Strategy

1. **Unit Tests**:

   - Test key generation for each key type
   - Test formatting functions for each key type
   - Test naming convention conversions
   - Test file injection for each file type

2. **Integration Tests**:
   - Test the entire workflow with sample configuration files
   - Test dry-run mode
   - Test error handling

## Future Enhancements

1. Support for additional key types (Ed25519, RSA, etc.)
2. Support for additional file types (JSON, YAML, etc.)
3. Support for more complex injection patterns
4. Support for key derivation from existing keys
5. Support for importing existing keys
