//! Streaming, atomic output commit; journal/backup/restore use the image safety service.
use std::{fs,path::{Path,PathBuf}};
use super::{VideoOptions,VideoResult,probe::VideoProbe};
use crate::{history::{HistoryStore,HistoryEntry,now_millis},output_transaction::{TransactionStore,ReplaceTransaction},engine::CompressResult};
#[derive(Debug,Clone,PartialEq)]pub struct SourceIdentity {pub size:u64,pub modified:Option<std::time::SystemTime>,pub identity:Option<(u64,u64)>}
impl SourceIdentity {
    pub fn read(path:&Path)->Result<Self,String>{let m=fs::metadata(path).map_err(|e|e.to_string())?;
        #[cfg(unix)]let identity={use std::os::unix::fs::MetadataExt;Some((m.dev(),m.ino()))};
        #[cfg(not(unix))]let identity=None;
        Ok(Self{size:m.len(),modified:m.modified().ok(),identity})}
}
pub fn available_space(path:&Path)->Result<u64,String>{
    #[cfg(unix)]{use std::{ffi::CString,os::unix::ffi::OsStrExt};
        let c=CString::new(path.as_os_str().as_bytes()).map_err(|e|e.to_string())?;
        let mut stats=std::mem::MaybeUninit::<libc::statvfs>::uninit();
        if unsafe{libc::statvfs(c.as_ptr(),stats.as_mut_ptr())}!=0{return Err(std::io::Error::last_os_error().to_string());}
        let stats=unsafe{stats.assume_init()};return Ok((stats.f_bavail as u64).saturating_mul(stats.f_frsize as u64));
    }
    #[cfg(not(unix))]{let _=path;Err("Disk-space verification is unavailable on this platform".into())}
}
pub fn destination(source:&Path,o:&VideoOptions,roots:&[String])->Result<PathBuf,String>{
    if o.output_mode=="replace"{
        let ext=source.extension().and_then(|s|s.to_str()).unwrap_or("").to_lowercase();
        if ext!=o.container && !(ext=="m4v"&&o.container=="mp4"){return Err("Safe replace currently requires the source container extension; use suffix for format conversion".into());}
        return Ok(source.into());
    }
    let stem=source.file_stem().ok_or("Source has no file name")?.to_string_lossy();
    let suffix:String=o.output_suffix.trim().chars().map(|c|if c=='/'||c=='\\'||c.is_control(){'_'}else{c}).take(64).collect();
    let suffix=if suffix.is_empty(){"_compressed".to_owned()}else{suffix};
    let parent=if o.output_mode=="directory"||o.output_mode=="folder" {
        let root=PathBuf::from(o.output_dir.as_deref().ok_or("Select output directory")?);
        let rel=roots.iter().filter_map(|r|Path::new(r).canonicalize().ok()).filter(|r|r.is_dir()).filter_map(|r|source.strip_prefix(r).ok().map(Path::to_path_buf)).min_by_key(|p|p.components().count());
        root.join(rel.and_then(|r|r.parent().map(Path::to_path_buf)).unwrap_or_default())
    }else{source.parent().ok_or("Source has no parent")?.to_owned()};
    fs::create_dir_all(&parent).map_err(|e|e.to_string())?;
    for n in 0..10000 {let tail=if n==0{String::new()}else{format!("_{n}")};let p=parent.join(format!("{stem}{suffix}{tail}.{}",o.container));if !p.exists(){return Ok(p);}}
    Err("Too many output filename conflicts".into())
}
pub fn sync_dir(path:&Path)->Result<(),String>{fs::File::open(path).and_then(|f|f.sync_all()).map_err(|e|e.to_string())}
pub fn commit(source:&Path,partial:&Path,target:&Path,o:&VideoOptions,probe:&VideoProbe,result:&mut VideoResult,history:&HistoryStore,transactions:&TransactionStore,retention:u32,identity:&SourceIdentity)->Result<(),String>{
    if &SourceIdentity::read(source)?!=identity{return Err("Source changed during encoding; original was not overwritten".into());}
    let size=fs::metadata(partial).map_err(|e|e.to_string())?.len();
    fs::File::open(partial).and_then(|f|f.sync_all()).map_err(|e|e.to_string())?;
    let replace=o.output_mode=="replace";
    let backup=if replace{Some(history.ensure_backup(source)?)}else{None};
    if let Some(ref backup)=backup {fs::File::open(backup).and_then(|f|f.sync_all()).map_err(|e|e.to_string())?;sync_dir(backup.parent().ok_or("Backup parent missing")?)?;}
    let id=HistoryEntry::new_id(source);
    let txn=backup.as_ref().map(|b|ReplaceTransaction{id:id.clone(),history_id:id.clone(),source_path:source.to_string_lossy().into(),output_path:target.to_string_lossy().into(),backup_path:b.to_string_lossy().into(),temp_output_path:Some(partial.to_string_lossy().into()),original_size:probe.size_bytes,expected_output_size:size,created_at:now_millis(),cross_format:false});
    if let Some(t)=txn.as_ref(){transactions.prepare_durable(t)?;}
    let install=(||{
        // Check again after a potentially long backup copy.
        if &SourceIdentity::read(source)?!=identity{return Err("Source changed while saving backup".into());}
        if replace {fs::rename(partial,target).map_err(|e|e.to_string())?;}else{
            // Hard-link is atomic and fails if an external actor claimed the target.
            fs::hard_link(partial,target).map_err(|e|format!("Cannot atomically claim output path: {e}"))?;
            fs::remove_file(partial).map_err(|e|e.to_string())?;
        }
        sync_dir(target.parent().ok_or("Output parent missing")?)?;
        let image_contract=CompressResult{success:true,file:source.to_string_lossy().into(),original_size:probe.size_bytes,compressed_size:size,savings:result.savings.unwrap_or(0.0),original_size_formatted:crate::engine::format_bytes(probe.size_bytes),compressed_size_formatted:crate::engine::format_bytes(size),out_type:o.container.clone(),algorithm:result.encoder_actual.clone(),error:None,output_path:Some(target.to_string_lossy().into()),backup_path:backup.as_ref().map(|p|p.to_string_lossy().into()),output_mode:Some(if o.output_mode=="directory"{"folder".into()}else{o.output_mode.clone()})};
        let mut entry=HistoryEntry::record_with_id(id.clone(),source,&image_contract,target,backup.as_deref(),retention);
        entry.media_kind="video".into();entry.video_metadata=Some(serde_json::json!({"probe":probe,"encoderActual":result.encoder_actual,"warnings":result.warnings,"elapsedMs":result.elapsed_ms,"diagnosticsLogPath":result.diagnostics_log_path}));
        history.add(entry)?;
        Ok::<(),String>(())
    })();
    if let Err(error)=install {
        if let Some(t)=txn.as_ref(){
            // Only roll back if replacement actually happened. A changed source before rename must not be clobbered.
            if !partial.exists() {if let Err(rollback)=crate::output_transaction::rollback(t){return Err(format!("{error}; rollback failed, backup retained: {rollback}"));}}
            transactions.finish(&id);
        }else if target.exists()&&!partial.exists(){let _=fs::remove_file(target);}
        return Err(error);
    }
    if let Some(t)=txn{transactions.finish(&t.id);}
    result.output_path=Some(target.to_string_lossy().into());result.backup_path=backup.map(|p|p.to_string_lossy().into());result.history_id=Some(id);Ok(())
}
#[cfg(test)]mod tests{use super::*;
    #[test]fn suffix_conflicts_never_claim_source(){let d=tempfile::tempdir().unwrap();let s=d.path().join("clip.mp4");fs::write(&s,b"source").unwrap();fs::write(d.path().join("clip_compressed.mp4"),b"existing").unwrap();let p=destination(&s,&VideoOptions::default(),&[]).unwrap();assert_eq!(p.file_name().unwrap(),"clip_compressed_1.mp4");assert_eq!(fs::read(s).unwrap(),b"source");}
    #[test]fn source_identity_detects_changes(){let d=tempfile::tempdir().unwrap();let s=d.path().join("clip");fs::write(&s,b"original").unwrap();let id=SourceIdentity::read(&s).unwrap();fs::write(&s,b"changed and longer").unwrap();assert_ne!(id,SourceIdentity::read(&s).unwrap());}
}
