use clap::{Parser, Subcommand};
use shadow_rs::shadow;
shadow!(build);

const VERSION: &str = build::LAST_TAG;

const LONG_VERSION: &str = shadow_rs::formatcp!(r#"{}
Arch        : {}
Branch      : {}
Commit      : {}
Build time  : {}
Mode        : {}"#, 
build::LAST_TAG, 
build::BUILD_TARGET_ARCH, 
build::BRANCH,  
build::SHORT_COMMIT, 
build::BUILD_TIME,
build::BUILD_RUST_CHANNEL);

#[derive(Parser, Debug)]
#[command(author, version = VERSION, about, long_about = None, long_version = LONG_VERSION)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// calculate the md5 of a physical memory
    Md5 {
        /// physical address
        #[arg(short, long, value_parser = addr_validator)]
        addr: u64,

        /// size
        #[arg(short, long, value_parser = size_validator)]
        size: usize,
    },

    /// write
    Write {
        /// physical address
        #[arg(short, long, value_parser = addr_validator)]
        addr: u64,

        /// size
        #[arg(short, long, value_parser = size_validator)]
        size: usize,

        /// write value, if ignored, use random value
        #[arg(short, long)]
        value: Option<u8>,
    },

    /// clear
    Clear {
        /// physical address
        #[arg(short, long, value_parser = addr_validator)]
        addr: u64,

        /// size
        #[arg(short, long, value_parser = size_validator)]
        size: usize,

        /// clear value, if ignored, use 0
        #[arg(short, long)]
        value: Option<u8>,
    },

    /// read
    Read {
        /// physical address
        #[arg(short, long, value_parser = addr_validator)]
        addr: u64,

        /// size
        #[arg(short, long, value_parser = size_validator)]
        size: usize,
    },

    /// memory dump
    MD {
        /// physical address
        #[arg(short, long, value_parser = addr_validator)]
        addr: u64,

        /// unit, valid value: 1|2|4|8
        #[arg(short, long, value_parser = unit_validator)]
        unit: usize,

        /// unit count
        #[arg(short, long, value_parser = size_validator)]
        count: usize,
    },

    /// memory set
    MS {
        /// physical address
        #[arg(short, long, value_parser = addr_validator)]
        addr: u64,

        /// size, valid value: 1|2|4|8
        #[arg(short, long, value_parser = unit_validator)]
        size: usize,

        /// set value
        #[arg(short, long, value_parser = addr_validator)]
        value: u64,
    },
}

const FACTOR_K: usize = 1024;
const FACTOR_M: usize = 1024 * FACTOR_K;
const FACTOR_G: usize = 1024 * FACTOR_M;
const FACTOR_T: usize = 1024 * FACTOR_G;

fn str2u64(s: &str) -> anyhow::Result<u64> {
    let mut start = 0;
    let mut radix = 10;

    if s.starts_with("0x") || s.starts_with("0X") {
        start = 2;
        radix = 16;
    } else if s.starts_with("0b") || s.starts_with("0B") {
        start = 2;
        radix = 2;
    }

    let value = u64::from_str_radix(&s[start..], radix)?;

    Ok(value)
}

fn str2usize(s: &str) -> anyhow::Result<usize> {
    let mut start = 0;
    let mut radix = 10;

    if s.starts_with("0x") || s.starts_with("0X") {
        start = 2;
        radix = 16;
    } else if s.starts_with("0b") || s.starts_with("0B") {
        start = 2;
        radix = 2;
    }

    let value = usize::from_str_radix(&s[start..], radix)?;

    Ok(value)
}

fn addr_validator(s: &str) -> anyhow::Result<u64> {
    let addr =  match str2u64(s) {
        Ok(v) => v,
        Err(e) => {
            return Err(anyhow::format_err!(": {e}"));
        },
    };

    Ok(addr)
}

fn size_validator(s: &str) -> anyhow::Result<usize> {
    let mut factor = 1;
    let mut len = s.len();

    if s.ends_with('T') || s.ends_with('t') {
        factor = FACTOR_T;
        len -= 1;
    } else if s.ends_with('G') || s.ends_with('g') {
        factor = FACTOR_G;
        len -= 1;
    } else if s.ends_with('M') || s.ends_with('m') {
        factor = FACTOR_M;
        len -= 1;
    } else if s.ends_with('K') || s.ends_with('k') {
        factor = FACTOR_K;
        len -= 1;
    }

    if len == 0 {
        return Err(anyhow::format_err!("no value found"));
    }

    let size =  match str2usize(&s[0..len]) {
        Ok(v) => v,
        Err(e) => {
            return Err(anyhow::format_err!("{e}"));
        },
    };

    Ok(size * factor)
}

fn unit_validator(s: &str) -> anyhow::Result<usize> {
    match s {
        "1" | "2" | "4" | "8" => {},
        _ => {
            return Err(anyhow::format_err!("please input 1|2|4|8"));
        },
    }

    let unit =  match str2usize(s) {
        Ok(v) => v,
        Err(e) => {
            return Err(anyhow::format_err!("{e}"));
        },
    };

    Ok(unit)
}