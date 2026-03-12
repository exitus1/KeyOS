//! Command line arguments.

use std::path::PathBuf;

use cosign2::Header;

#[derive(clap::Parser)]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand)]
pub enum Command {
    /// Dump the header contents to stdout.
    Dump {
        /// The binary file.
        #[clap(short, long)]
        input: PathBuf,
        /// Size of the header in bytes.
        #[clap(long, default_value_t = Header::DEFAULT_SIZE as u16, value_parser = clap::value_parser!(u16).range(Header::MIN_SIZE as i64..=Header::MAX_SIZE as i64))]
        header_size: u16,
    },
    /// Sign a binary file.
    Sign {
        /// The public key in hex, verified against the secret key to avoid
        /// accidental signing.
        #[clap(long)]
        pubkey: Option<String>,
        /// Path to PEM-encoded secret key.
        #[clap(long)]
        secret: Option<PathBuf>,
        /// Path to config file.
        #[clap(long, short)]
        config: Option<PathBuf>,
        /// The binary file.
        #[clap(short, long)]
        input: PathBuf,
        /// Update the binary file in place.
        #[clap(long)]
        in_place: bool,
        /// Path to write the signed binary file.
        #[clap(short, long)]
        output: Option<PathBuf>,
        /// Version to write in the header.
        #[clap(long)]
        binary_version: Option<semver::Version>,
        /// Developer mode, signs with a single key.
        #[clap(long)]
        developer: bool,
        /// Target device. Valid values are "atsama5d27-keyos".
        #[clap(long)]
        target: Option<String>,
        /// Known public keys to accept signatures from, separated by commas.
        #[clap(long)]
        known_pubkey: Option<Vec<String>>,
        /// Size of the header in bytes.
        #[clap(long, default_value_t = Header::DEFAULT_SIZE as u16, value_parser = clap::value_parser!(u16).range(Header::MIN_SIZE as i64..=Header::MAX_SIZE as i64))]
        header_size: u16,
    },
}
