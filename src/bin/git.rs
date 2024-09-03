use std::{
    ffi::CStr,
    fs,
    io::{self, BufRead, BufReader, Read, Write},
};

use anyhow::{Context, Ok};
use clap::{Parser, Subcommand};
use flate2::{read::ZlibDecoder, GzHeader};
use futures::lock;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    CatFile { pretty: bool, hash: String },
}

enum GitObject {
    Blob,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.commands {
        Commands::Init => {
            fs::create_dir(".git").context("could start git")?;
            fs::create_dir(".git/objects").context("could start git")?;
            fs::create_dir(".git/refs").context("could start git")?;
            fs::write(".git/HEAD", "refs: ref/main\n").context("could start git")?;
        }
        Commands::CatFile { pretty, hash } => {
            let path = format!(".git/objects/{}/{}", &hash[..2], &hash[2..]);
            let file = fs::File::open(path).context("Blob field with given hash not found")?;
            let z = ZlibDecoder::new(file);
            let mut z = BufReader::new(z);
            let mut buf = vec![];
            z.read_until(0, &mut buf).context("Invalid Filed format")?;

            let header = CStr::from_bytes_with_nul(&buf).context("Invalid Format")?;
            let header = header.to_str().context("Invalid format")?;
            let Some((kind, size)) = header.split_once(" ") else {
                anyhow::bail!("Invalid format");
            };

            let size = size.parse::<u64>().context("invalid blob size")?;
            let mut z = z.take(size);
            match kind {
                "blob" => {
                    let mut stdout = io::stdout().lock();
                    let n = io::copy(&mut z, &mut stdout).context(".git/Object copy to stdout")?;
                    anyhow::ensure!(
                        n == size,
                        "unexpected object size, expected:{size},got {n})"
                    )
                }
                _ => anyhow::bail!("Invalid format"),
            };
        }
    }

    Ok(())
}
