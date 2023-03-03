fn main() -> std::io::Result<()> {
    let out_dir = std::path::PathBuf::from(std::env::var_os("OUT_DIR").ok_or(std::io::ErrorKind::NotFound)?);

    let cmd = clap::Command::new("mybin")
        .arg(clap::arg!(-n --name <NAME>))
        .arg(clap::arg!(-c --count <NUM>));

    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;

    std::fs::write(out_dir.join("mybin.1"), buffer)?;

    Ok(())
}

// expample from: https://crates.io/crates/clap_mangen
// mybin.1 is write to some deep directory:
// ../../target/debug/build/mangen-1731a10319f90a96/out/mybin.1
