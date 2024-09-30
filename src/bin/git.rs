use std::{
    ffi::CStr,
    fs,
    io::{self, copy, BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
};

use anyhow::Context;
use clap::{Parser, Subcommand};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    CatFile {
        pretty: bool,
        hash: String,
    },
    HashObject {
        #[clap(short = 'w')]
        write: bool,
        path: PathBuf,
    },
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
        Commands::HashObject { write, path } => {
            //read file into memory, grab stats
            //use compressor to write headers
            //hash as writing

            fn write_blob<W>(path: &Path, writer: W) -> anyhow::Result<String>
            where
                W: Write,
            {
                let stats =
                    fs::metadata(&path).with_context(|| format!("stat {}", path.display()))?;
                let z = ZlibEncoder::new(writer, Compression::default());
                let mut writer = HashWriter {
                    hasher: Sha1::new(),
                    writer: z,
                };
                write!(writer, "blob {}\0", stats.len())?;
                let mut file = fs::File::open(&path)
                    .with_context(|| format!("open source file {}", path.display()))?;
                copy(&mut file, &mut writer).context("copy stream data into blob")?;
                writer.writer.finish()?;
                let hash = writer.hasher.finalize();
                Ok(hex::encode(hash))
            }
            let hash = if write {
                let tmp = "temporay";
                let hash = write_blob(
                    &path,
                    fs::File::create(tmp).context("create temporaty file")?,
                )
                .context("hash blob object")?;
                fs::create_dir_all(format!(".git/objects/{}", &hash[..2]))
                    .context("create subdir git/objects")?;
                fs::rename(tmp, format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
                    .context("move blob into .git/objects")?;
                hash
            } else {
                write_blob(&path, io::sink())?
            };

            println!("{hash}")
        }
    }

    Ok(())
}

struct HashWriter<W> {
    writer: W,
    hasher: Sha1,
}

impl<W> Write for HashWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.writer.write(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        todo!()
    }
}
