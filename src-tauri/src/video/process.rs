//! Direct only. Typed argv, bounded reads, deadlines, child reaping.
use std::{path::{Path,PathBuf}, process::{Command,Stdio}, io::Read, time::{Duration,Instant}};
pub fn tool(name: &str) -> Result<PathBuf,String> {
    let exe = std::env::current_exe().map_err(|e|e.to_string())?;
    if let Some(macos) = exe.parent() {
        let bundled = macos.join("../Resources/video/bin").join(name);
        if bundled.is_file() { return Ok(bundled); }
        // Bundled applications must never silently depend on the host's PATH.
        if macos.file_name().and_then(|s|s.to_str()) == Some("MacOS") {
            return Err(format!("Sizqo bundle is missing video/bin/{name}"));
        }
    }
    let dev = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/video/bin").join(name);
    if dev.is_file() { return Ok(dev); }
    let paths = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&paths).map(|p|p.join(name)).find(|p|p.is_file())
        .ok_or_else(||format!("{name} unavailable; build the bundled video runtime"))
}
pub fn command(name: &str) -> Result<Command,String> {
    let path=tool(name)?;
    let mut cmd=Command::new(&path);
    if let Some(bin)=path.parent() {
        let lib=bin.join("../lib");
        if lib.is_dir() { cmd.env("DYLD_FALLBACK_LIBRARY_PATH",lib); }
    }
    cmd.stdin(Stdio::null()); Ok(cmd)
}
pub fn drain(mut input: impl Read, limit: usize) -> std::io::Result<Vec<u8>> {
    let mut saved=Vec::new(); let mut chunk=[0u8;8192];
    loop { let n=input.read(&mut chunk)?; if n==0 { break; }
        let take=n.min(limit.saturating_sub(saved.len())); saved.extend_from_slice(&chunk[..take]); }
    Ok(saved)
}
pub fn capture(mut cmd: Command, timeout: Duration, limit: usize) -> Result<Vec<u8>,String> {
    let mut child=cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e|e.to_string())?;
    let stdout=child.stdout.take().ok_or("stdout unavailable")?;
    let stderr=child.stderr.take().ok_or("stderr unavailable")?;
    let out=std::thread::spawn(move||drain(stdout,limit));
    let err=std::thread::spawn(move||drain(stderr,65536));
    let start=Instant::now();
    let status=loop {
        match child.try_wait() {
            Ok(Some(s))=>break Ok(s), Ok(None)=>{}, Err(e)=>{ let _=child.kill(); let _=child.wait(); break Err(e.to_string()); }
        }
        if start.elapsed()>timeout { let _=child.kill(); let _=child.wait(); break Err("Media operation timed out".into()); }
        std::thread::sleep(Duration::from_millis(30));
    };
    let output=out.join().map_err(|_|"stdout reader failed")?.map_err(|e|e.to_string())?;
    let errors=err.join().map_err(|_|"stderr reader failed")?.map_err(|e|e.to_string())?;
    if !status?.success() { return Err(String::from_utf8_lossy(&errors).chars().take(4096).collect()); }
    if output.len()>=limit { return Err("Media metadata exceeds safety limit".into()); }
    Ok(output)
}
