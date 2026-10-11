use super::{VideoOptions,capabilities::CapabilityMatrix};
use serde::Serialize;
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct VideoPreset{pub id:String,pub name:String,pub description:String,pub available:bool,pub reason:Option<String>,pub options:VideoOptions}
pub fn presets(caps:&CapabilityMatrix)->Vec<VideoPreset>{
    let mut out=vec![];
    for (id,name,encoder,mode,crf,description) in [
        ("quick","快速","h264_videotoolbox","abr",None,"速度与兼容优先；实际大小取决于素材"),
        ("balanced","均衡","auto","abr",None,"HEVC 硬件优先；若不可用会明确显示实际编码器"),
        ("quality","高质量压缩","libx265","crf",Some(28.0),"软件 HEVC；保留分辨率与时间戳，不保证更小"),
        ("smallest","极致体积","libsvtav1","crf",Some(35.0),"软件 AV1；兼容性有限，编码较慢"),
        ("compatible","兼容优先","libx264","crf",Some(23.0),"H.264 / AAC / MP4 / 8-bit SDR"),
        ("target","自定义目标大小","libx264","targetSize",None,"两遍码率估算；无法保证精确大小")]{
        let mut options=VideoOptions::default();options.encoder=encoder.into();options.mode=mode.into();options.crf=crf;
        if id=="quick"||id=="compatible"{options.audio_policy="aac".into();}
        if id=="target"{options.target_mi_b=Some(20.0);}
        let available=caps.available&&(encoder=="auto"||caps.encoder(encoder).map(|c|c.modes.iter().any(|m|m==mode)).unwrap_or(false));
        out.push(VideoPreset{id:id.into(),name:name.into(),description:description.into(),available,
            reason:if available{None}else{Some(caps.reason.clone().unwrap_or_else(||format!("{encoder} unavailable in this edition")))},options});
    }out
}
