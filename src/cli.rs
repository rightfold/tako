// Tako -- Take container image.
// Copyright 2018 Arian van Putten, Ruud van Asseldonk, Tako Marks.

//! Command-line argument parser.
//!
//! There do exist Rust libraries for this, but they either bring along too many
//! dependencies, or they only support flags and not commands.

use std::env;
use std::env::Args;
use std::path::PathBuf;
use std::fmt;

const USAGE: &'static str = "
Tako -- Take container image.

Usage:
  tako <command> [<args>...]
  tako -h | --help
  tako --version

Commands:
  fetch      Download or update an image.
  store      Add a new image version to a server directory.
  gen-key    Generate a key pair for signing manifests.

Options:
  -h --help  Show this screen, or help about a command.
  --version  Show version.

See 'tako <command> --help' for information on a specific command.
";

const USAGE_FETCH: &'static str = "
tako fetch -- Download or update an image.

Usage:
  tako fetch [--init] [--] <config>...

Options:
  --init    Download image only if none exists already.

Arguments:
  <config>  Path to a config file that determines what to fetch.
";

const USAGE_STORE: &'static str = "
tako store -- Add a new image version to a server directory.

Usage:
  tako store [-k <key> | -f <file>] --output <dir> [--] <image> <version>

Options:
  -k --key <key>        Secret key to sign the manifest with. Can alternatively
                        be read from the TAKO_SECRET_KEY environment variable.
  -f --key-file <file>  File to read the secret key from.
  -o --output <dir>     Server directory.

Arguments:
  <image>               Path to image file to be stored.
  <version>             Version to store the image under.
";

const USAGE_GEN_KEY: &'static str = "
tako gen-key -- Generate a key pair for signing manifests.

Usage:
  tako gen-key
";

pub struct Store {
    secret_key: Option<String>,
    secret_key_path: Option<PathBuf>,
    output_path: PathBuf,
    version: String,
    image_path: PathBuf,
}

pub enum Cmd {
    Fetch(Vec<String>),
    Init(Vec<String>),
    Store(Store),
    GenKey,
    Help(String),
    Version,
}

pub fn print_usage(cmd: String) {
    match &cmd[..] {
        "tako" => print!("{}", USAGE),
        "fetch" => print!("{}", USAGE_FETCH),
        "store" => print!("{}", USAGE_STORE),
        "gen-key" => print!("{}", USAGE_GEN_KEY),
        _ => print!("'{}' is not a Tako command. See 'tako --help'.", cmd),
    }
}

pub fn print_version() {
    println!("0.0.0");
    // TODO: Licenses and stuff.
}

enum Arg<T> {
    Plain(T),
    Short(T),
    Long(T),
}

impl Arg<String> {
    fn as_ref(&self) -> Arg<&str> {
        match *self {
            Arg::Plain(ref x) => Arg::Plain(&x[..]),
            Arg::Short(ref x) => Arg::Short(&x[..]),
            Arg::Long(ref x) => Arg::Long(&x[..]),
        }
    }

    fn into_string(self) -> String {
        match self {
            Arg::Plain(x) => x,
            Arg::Short(x) => x,
            Arg::Long(x) => x,
        }
    }
}

impl fmt::Display for Arg<String> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Arg::Plain(ref x) => write!(f, "{}", x),
            Arg::Short(ref x) => write!(f, "-{}", x),
            Arg::Long(ref x) => write!(f, "--{}", x),
        }
    }
}

struct ArgIter {
    /// Underlying `std::env` args iterator.
    args: Args,

    /// Whether we have observed a `--` argument.
    is_raw: bool,

    /// Leftover to return after an `--foo=bar` or `-fbar`-style argument.
    ///
    /// `--foo=bar` is returned as `Long(foo)` followed by `Plain(bar)`.
    /// `-fbar` is returned as `Short(f)` followed by `Plain(bar)`.
    leftover: Option<String>,
}

impl ArgIter {
    pub fn new() -> ArgIter {
        ArgIter {
            args: env::args(),
            is_raw: false,
            leftover: None,
        }
    }
}

impl Iterator for ArgIter {
    type Item = Arg<String>;

    fn next(&mut self) -> Option<Arg<String>> {
        if self.leftover.is_some() {
            return self.leftover.take().map(Arg::Plain)
        }

        let arg = self.args.next()?;

        if self.is_raw {
            return Some(Arg::Plain(arg))
        }

        if &arg == "--" {
            self.is_raw = true;
            return self.next()
        }

        if arg.starts_with("--") {
            let mut flag = String::from(&arg[2..]);
            if let Some(i) = flag.find('=') {
                self.leftover = Some(flag.split_off(i + 1));
                flag.truncate(i);
            }
            return Some(Arg::Long(flag))
        }

        if arg.starts_with("-") {
            let mut flag = String::from(&arg[1..]);
            if flag.len() > 1 {
                self.leftover = Some(flag.split_off(1));
                flag.truncate(1);
            }
            return Some(Arg::Short(arg))
        }

        Some(Arg::Plain(arg))
    }
}

pub fn parse() -> Result<Cmd, String> {
    let mut args = ArgIter::new();

    // Skip executable name.
    args.next();

    let arg = match args.next() {
        Some(a) => a,
        None => return Err("No command provided. See --help.".to_string()),
    };

    match arg.as_ref() {
        Arg::Plain("fetch") => parse_fetch(args),
        Arg::Plain("store") => parse_store(args),
        Arg::Plain("gen-key") => drain(args).and(Ok(Cmd::GenKey)),
        Arg::Long("version") => drain(args).and(Ok(Cmd::Version)),
        Arg::Short("h") | Arg::Long("help") => parse_help(args),
        _ => return unexpected(arg),
    }
}

fn parse_fetch(mut args: ArgIter) -> Result<Cmd, String> {
    let mut fnames = Vec::new();
    let mut is_init = false;
    while let Some(arg) = args.next() {
        match arg.as_ref() {
            Arg::Plain(..) => fnames.push(arg.into_string()),
            Arg::Long("init") => is_init = true,
            Arg::Short("h") | Arg::Long("help") => return drain_help(args, "fetch"),
            _ => return unexpected(arg),
        }
    }

    if fnames.len() == 0 {
        return Err("Expected at least one fetch config filename.".to_string())
    }

    if is_init {
        Ok(Cmd::Init(fnames))
    } else {
        Ok(Cmd::Fetch(fnames))
    }
}

fn parse_store(mut args: ArgIter) -> Result<Cmd, String> {
    let mut output_path = None;
    let mut secret_key = None;
    let mut secret_key_path = None;
    let mut image_path = None;
    let mut version = None;

    while let Some(arg) = args.next() {
        match arg.as_ref() {
            Arg::Short("k") | Arg::Long("key") => {
                let msg = "Expected secret key after --key.";
                secret_key = Some(expect_plain(&mut args, msg)?);
            }
            Arg::Short("f") | Arg::Long("key-file") => {
                let msg = "Expected key path after --key-file.";
                secret_key_path = Some(expect_plain(&mut args, msg)?);
            }
            Arg::Short("o") | Arg::Long("output") => {
                let msg = "Expected server directory after --output.";
                output_path = Some(expect_plain(&mut args, msg)?);
            }
            Arg::Short("h") | Arg::Long("help") => {
                return drain_help(args, "store")
            }
            Arg::Plain(..) if image_path.is_none() => {
                image_path = Some(arg.into_string());
            }
            Arg::Plain(..) if version.is_none() => {
                version = Some(arg.into_string());
            }
            _ => return unexpected(arg)
        }
    }

    // If --key nor --key-file are provided, check the TAKO_SECRET_KEY
    // environment variable.
    if secret_key.is_none() && secret_key_path.is_none() {
        match env::var("TAKO_SECRET_KEY") {
            Ok(v) => secret_key = Some(v),
            Err(..) => {
                let msg = "Secret key not provided. Pass it via --key, \
                           read if from a key file with --key-file, \
                           or set the TAKO_SECRET_KEY environment variable.";
                return Err(msg.to_string())
            }
        }
    }

    let msg = "Server directory not provided. Pass it via --output.";
    let output_path = output_path.ok_or(msg.to_string())?;

    let msg = "Image path not provided. See 'tako store --help' for usage.";
    let image_path = image_path.ok_or(msg.to_string())?;

    let msg = "Version not provided. See 'tako store --help' for usage.";
    let version = version.ok_or(msg.to_string())?;

    let store = Store {
        secret_key: secret_key,
        secret_key_path: secret_key_path.map(PathBuf::from),
        output_path: PathBuf::from(output_path),
        version: version,
        image_path: PathBuf::from(image_path),
    };

    Ok(Cmd::Store(store))
}

fn parse_help(mut args: ArgIter) -> Result<Cmd, String> {
    match args.next() {
        Some(Arg::Plain(cmd)) => drain(args).and(Ok(Cmd::Help(cmd))),
        Some(arg) => unexpected(arg),
        None => Ok(Cmd::Help("tako".to_string())),
    }
}

fn drain_help(args: ArgIter, cmd: &'static str) -> Result<Cmd, String> {
    drain(args).and(Ok(Cmd::Help(cmd.to_string())))
}

fn expect_plain(args: &mut ArgIter, msg: &'static str) -> Result<String, String> {
    match args.next() {
        Some(Arg::Plain(a)) => Ok(a),
        Some(arg) => Err(format!("Unexpected argument '{}'. {}", arg, msg)),
        None => Err(msg.to_string()),
    }
}

fn drain(args: ArgIter) -> Result<(), String> {
    for arg in args {
        return unexpected::<()>(arg);
    }

    Ok(())
}

fn unexpected<T>(arg: Arg<String>) -> Result<T, String> {
    Err(format!("Unexpected argument '{}'.", arg))
}
