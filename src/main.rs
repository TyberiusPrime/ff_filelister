use anyhow::{bail, Context, Result};
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};
use std::process;
use std::str::FromStr;
use std::time::SystemTime;

fn list_dir(path: impl AsRef<Path>) -> Result<String> {
    let rg_args = &[
        "--type",
        "js",
        "--type",
        "lua",
        "--type",
        "vim",
        "--type-add",
        "tera:*.tera",
        "--type-add",
        "cfg:*.cfg",
        "--type",
        "cfg",
        "--type-add",
        "toml:*.toml",
        "--type",
        "toml",
        "--type",
        "rust",
        "--type",
        "py",
        "--type",
        "cython",
        "--type",
        "rst",
        "--type",
        "md",
        "--type",
        "html",
        "--type",
        "txt",
        "--type",
        "nix",
        "-g!code/venv",
        "-g!results",
        "-g!cache",
        "-g!incoming",
        "-g!manual_code",
        "-g!*.egg-info",
        "-j",
        "40",
        "--files",
    ];
    let p = process::Command::new("rg")
        .args(rg_args)
        .current_dir(path)
        .output().context("Failed to run rg")?;
    let res = std::str::from_utf8(&p.stdout).context("rg output was not utf8")?.to_string();

    let lines = res.split("\n");
    let (no_test, test): (Vec<&str>, Vec<&str>) = lines.partition(|x| !x.contains("test"));
    let res: String = no_test.join("\n") + &test.join("\n");

    Ok(res)
}

fn main() {
    let r = inner_main();
    match r {
        Ok(_) => {}
        Err(x) => println!("An error occured {:?}", x),
    };
}
fn inner_main() -> Result<()> {
    let cache_dir = dirs::home_dir()
        .expect("No home")
        .join(".cache/ff_filelister");
    std::fs::create_dir_all(&cache_dir)
        .with_context(|| format!("Could not create cache dir {:?}", &cache_dir))?;

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("Wrong number of arguments. Needs to be [timeout_in_seconds] [folder]");
        std::process::exit(0);
    }
    let timeout: u32 = str::parse(&args[1]).context("timeout not a number")?;
    let target = &args[2];
    if !PathBuf::from_str(&target).context("could not cerate path from argument")?.exists() {
        bail!("Could not find target directory {}", target);
    }

    let mut hasher = Sha1::new();
    // process input message
    hasher.update(target.as_bytes());
    let result = format!("{:x}", hasher.finalize());

    let cache_file = cache_dir.join(result);
    let rebuild = if !cache_file.exists() {
        let raw = list_dir(&args[1]).context("failed to listdir")?;
        std::fs::write(&cache_file, raw).context("failed to write cache_file")?;
        false
    } else {
        true
    };
    println!("{}", std::fs::read_to_string(&cache_file).context("failed to read cache_file")?);

    //make telescope allow
    unsafe {
        libc::close(1);
    }
    if rebuild {
        if cache_file.metadata().context("no cache file metadata")?.modified().context("no cachefile modified time")?
            < SystemTime::now() - std::time::Duration::from_secs(timeout.into())
        {
            //update it
            let raw = list_dir(&args[1]).context("failed to listdir")?;
            std::fs::write(&cache_file, raw)?;
        }
    }

    Ok(())
}
