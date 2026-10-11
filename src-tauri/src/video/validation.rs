use super::{plan::EncodePlan,probe::VideoProbe};
pub fn validate(source:&VideoProbe,output:&VideoProbe,plan:&EncodePlan)->Result<(),String>{
    if output.size_bytes==0{return Err("Empty encoded output".into());}
    let v=output.video().ok_or("Output has no video stream")?;
    if (v.width,v.height)!=(plan.width,plan.height){return Err("Output dimensions do not match encode plan".into());}
    if v.pixel_format!="yuv420p"{return Err(format!("Unexpected pixel format {}",v.pixel_format));}
    let expected_codec=if plan.encoder=="libsvtav1"{"av1"}else if plan.encoder=="libx265"||plan.encoder=="hevc_videotoolbox"{"hevc"}else{"h264"};
    if v.codec_name!=expected_codec{return Err("Output encoder differs from requested encoder".into());}
    let mux=match plan.container.as_str(){"mp4"|"mov"=>output.format.contains("mov")||output.format.contains("mp4"),"mkv"=>output.format.contains("matroska"),"webm"=>output.format.contains("webm"),_=>false};
    if !mux{return Err("Output container differs from plan".into());}
    if let Some(original)=source.duration_us {
        let actual=output.duration_us.ok_or("Output duration is unavailable")?;
        if (actual as i128-original as i128).abs()as u64>(original/100).max(250000){return Err("Output duration differs from source".into());}
    }
    let expected_audio=if plan.options.audio_policy=="none"{0}else{source.audio().count()};
    if output.audio().count()!=expected_audio{return Err("Encoded output lost audio tracks".into());}
    if plan.options.audio_policy!="none" {for (a,b) in source.audio().zip(output.audio()){
        if a.channels!=b.channels{return Err("Encoded output changed audio channel count".into());}
        if a.tags["language"]!=b.tags["language"] && a.tags["language"].as_str().is_some_and(|s|s!="und"){return Err("Encoded output changed audio language tags".into());}
        if plan.options.audio_policy=="copy"&&a.codec_name!=b.codec_name{return Err("Copied audio codec changed".into());}
    }}
    let expected_subs=if plan.options.subtitle_policy=="copy"{source.subtitles().count()}else{0};
    if output.subtitles().count()!=expected_subs{return Err("Encoded output lost subtitle tracks".into());}
    if plan.options.metadata_policy=="preserve" && source.chapters.len()!=output.chapters.len(){return Err("Encoded output lost chapters".into());}
    if plan.container=="mkv"&&source.streams.iter().filter(|s|s.codec_type=="attachment").count()!=output.streams.iter().filter(|s|s.codec_type=="attachment").count(){return Err("Encoded output lost attachments".into());}
    if output.rotation.rem_euclid(360)!=0{return Err("Unexpected output rotation".into());}
    Ok(())
}
