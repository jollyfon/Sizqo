use serde::{Deserialize,Serialize};
pub const NATIVE_UNAVAILABLE:&str="Sizqo App Store video encoding is unavailable in this build. No third-party CLI is launched in the sandbox. Use the Direct edition; native video support requires validation before enabling.";
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct EncoderCapability {pub id:String,pub label:String,pub modes:Vec<String>,pub presets:Vec<String>}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct CapabilityMatrix {pub backend:String,pub available:bool,pub reason:Option<String>,pub encoders:Vec<EncoderCapability>,pub muxers:Vec<String>,pub audio_encoders:Vec<String>,pub diagnostics:Vec<String>}
impl CapabilityMatrix {
    pub fn encoder(&self,id:&str)->Option<&EncoderCapability>{self.encoders.iter().find(|e|e.id==id)}
    pub fn unavailable(reason:String)->Self{Self {backend:"unavailable".into(),available:false,reason:Some(reason),encoders:vec![],muxers:vec![],audio_encoders:vec![],diagnostics:vec![]}}
}
#[cfg(feature="inproc-backends")]
pub fn capabilities()->CapabilityMatrix{CapabilityMatrix::unavailable(NATIVE_UNAVAILABLE.into())}
#[cfg(not(feature="inproc-backends"))]
pub fn capabilities()->CapabilityMatrix{
    static CACHE:std::sync::OnceLock<CapabilityMatrix>=std::sync::OnceLock::new();
    CACHE.get_or_init(||discover().unwrap_or_else(CapabilityMatrix::unavailable)).clone()
}
#[cfg(not(feature="inproc-backends"))]
fn discover()->Result<CapabilityMatrix,String>{
    use super::process::{command,capture};use std::time::Duration;
    let mut cmd=command("ffmpeg")?;cmd.args(["-hide_banner","-encoders"]);
    let enc=String::from_utf8_lossy(&capture(cmd,Duration::from_secs(20),2*1024*1024)?).into_owned();
    let mut cmd=command("ffmpeg")?;cmd.args(["-hide_banner","-muxers"]);
    let mux=String::from_utf8_lossy(&capture(cmd,Duration::from_secs(20),1024*1024)?).into_owned();
    let exists=|name:&str|enc.lines().any(|s|s.split_whitespace().nth(1)==Some(name));
    let mut encoders=vec![];let mut diagnostics=vec![];
    for (id,label) in [("h264_videotoolbox","H.264 · Apple hardware"),("hevc_videotoolbox","HEVC · Apple hardware"),("libx264","H.264 · software"),("libx265","HEVC · software"),("libsvtav1","AV1 · software")] {
        if !exists(id) {diagnostics.push(format!("{id}: not compiled into FFmpeg"));continue;}
        let mut trial=command("ffmpeg")?;
        trial.args(["-hide_banner","-v","error","-nostdin","-f","lavfi","-i","color=c=black:s=128x128:r=1","-frames:v","1","-threads","1","-c:v",id]);
        if id.ends_with("videotoolbox"){trial.args(["-allow_sw","0","-b:v","500k"]);}
        else if id=="libsvtav1"{trial.args(["-preset","12","-svtav1-params","lp=1"]);}
        else if id=="libx265"{trial.args(["-preset","ultrafast","-x265-params","pools=1:frame-threads=1"]);}
        trial.args(["-pix_fmt","yuv420p","-f","null","-"]);
        match capture(trial,Duration::from_secs(30),65536){
            Ok(_)=>{let hardware=id.ends_with("videotoolbox");let two_pass=id=="libx264";
                let mut modes=vec!["abr".into()];if !hardware {modes.push("crf".into());} if two_pass {modes.extend(["twoPass".into(),"targetSize".into()]);}
                encoders.push(EncoderCapability{id:id.into(),label:label.into(),modes,presets:if hardware {vec![]} else {vec!["ultrafast".into(),"veryfast".into(),"medium".into(),"slow".into()]}});},
            Err(e)=>diagnostics.push(format!("{id}: trial failed: {e}")),
        }
    }
    let muxers=[("mp4","mp4"),("mov","mov"),("mkv","matroska"),("webm","webm")].iter().filter(|(_,real)|mux.lines().any(|s|s.split_whitespace().nth(1)==Some(*real))).map(|(ui,_)|ui.to_string()).collect();
    let audio_encoders=["aac","libopus"].iter().filter(|s|exists(s)).map(|s|s.to_string()).collect();
    Ok(CapabilityMatrix{backend:"ffmpeg".into(),available:!encoders.is_empty(),reason:if encoders.is_empty(){Some("No encoder passed the runtime trial".into())}else{None},encoders,muxers,audio_encoders,diagnostics})
}
