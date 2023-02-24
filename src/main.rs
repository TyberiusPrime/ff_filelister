use anyhow::{bail, Context, Result};
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};
use std::process;
use std::str::FromStr;
use std::time::SystemTime;

fn list_dir(path: impl AsRef<Path>, rg_args: Option<Vec<String>>) -> Result<String> {
    let rg_args = match rg_args {
        Some(x) => x,
        None => vec![
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
            "--type",
            "ts",
            "--type-add",
            "svelte:*.svelte",
            "--type",
            "svelte",
            "-g!code/venv",
            "-g!results",
            "-g!cache",
            "-g!incoming",
            "-g!manual_code",
            "-g!*.egg-info",
            "-j",
            "40",
            "--files",
        ].iter().map(|x| x.to_string()).collect()
    };
    let p = process::Command::new("/usr/bin/rg")
        .args(rg_args)
        .current_dir(path)
        .output()
        .context("Failed to run /usr/bin/rg")?;
    let res = std::str::from_utf8(&p.stdout)
        .context("rg output was not utf8")?
        .to_string();

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
    if args.len() < 3 {
        println!("Wrong number of arguments. Needs to be at least [timeout_in_seconds] [folder]");
        println!("Optionally pass in '--' after, followed by rg arguments");
        std::process::exit(0);
    }
    let timeout: u32 = str::parse(&args[1]).context("timeout not a number")?;
    let target = &args[2];
    if !PathBuf::from_str(&target)
        .context("could not cerate path from argument")?
        .exists()
    {
        bail!("Could not find target directory {}", target);
    }
    let rg_args = if args.len() > 3 {
        if &args[3] != "--" {
            let r: Vec<String> = (&args[4..]).iter().map(|x| x.to_string()).collect();
            Some(r)
        } else {
            bail!("if passing in rg arguments, third argument must be \"--\"");
        }
    } else {
        None
    };

    let mut hasher = Sha1::new();
    // process input message
    hasher.update(target.as_bytes());
    let result = format!("{:x}", hasher.finalize());

    let cache_file = cache_dir.join(result);
    let rebuild = if !cache_file.exists() {
        let raw = list_dir(&target, rg_args.clone()).context("failed to listdir")?;
        std::fs::write(&cache_file, raw).context("failed to write cache_file")?;
        false
    } else {
        true
    };
    println!(
        "{}",
        std::fs::read_to_string(&cache_file).context("failed to read cache_file")?
    );

    //make telescope allow
    unsafe {
        libc::close(1);
    }
    if rebuild {
        if cache_file
            .metadata()
            .context("no cache file metadata")?
            .modified()
            .context("no cachefile modified time")?
            < SystemTime::now() - std::time::Duration::from_secs(timeout.into())
        {
            //update it
            let raw = list_dir(&target, rg_args).context("failed to listdir")?;
            std::fs::write(&cache_file, raw)?;
        }
    }

    Ok(())
}
