//! Direct FFmpeg engine: streaming IO, bounded diagnostics, independent cancellation.
use std::{fs,io::{Read,Write,BufRead,BufReader},path::Path,sync::{Arc,atomic::{AtomicBool,Ordering}},time::{Instant,Duration},process::Stdio};
use super::{VideoOptions,VideoResult,capabilities::CapabilityMatrix,probe,plan,progress::{VideoProgress,ProgressParser},transaction};
use crate::{history::HistoryStore,output_transaction::TransactionStore};
pub fn encode(task_id:&str,source:&Path,options:&VideoOptions,caps:&CapabilityMatrix,session:&str,revision:u64,roots:&[String],history:&HistoryStore,transactions:&TransactionStore,retention:u32,budget:usize,logs:&Path,cancel:Arc<AtomicBool>,emit:&(dyn Fn(VideoProgress)+Sync))->VideoResult{
    let started=Instant::now();let mut result=VideoResult::pending(task_id,source,options);
    let outcome=(||->Result<(),String>{
        let source=source.canonicalize().map_err(|e|e.to_string())?;
        let identity=transaction::SourceIdentity::read(&source)?;
        let source_probe=probe::probe(&source)?;
        let plan=plan::build(options,&source_probe,caps,budget)?;
        result.encoder_actual=plan.encoder.clone();result.warnings=plan.warnings.clone();
        let target=transaction::destination(&source,options,roots)?;
        let parent=target.parent().ok_or("Output directory missing")?;
        let estimated=plan.bitrate_kbps.zip(source_probe.duration_us).map(|(b,t)|b.saturating_mul(1000).saturating_mul(t/1000000)/8).unwrap_or(source_probe.size_bytes);
        let needed=estimated.max(source_probe.size_bytes/4).saturating_mul(2).saturating_add(64*1024*1024);
        if transaction::available_space(parent)?<needed{return Err(format!("Insufficient output disk space: reserve at least {} bytes",needed));}
        if options.output_mode=="replace" {
            // Reserve on the persistent backup volume independently; never put originals in temp.
            let backup_root=logs.parent().ok_or("Application data directory missing")?;
            let mut needed_backup=source_probe.size_bytes.saturating_add(64*1024*1024);
            #[cfg(unix)]{use std::os::unix::fs::MetadataExt;if fs::metadata(parent).map_err(|e|e.to_string())?.dev()==fs::metadata(backup_root).map_err(|e|e.to_string())?.dev(){needed_backup=needed_backup.saturating_add(needed);}}
            if transaction::available_space(backup_root)?<needed_backup{return Err("Insufficient disk space for the persistent original backup and encoded output".into());}
            result.warnings.push(format!("Replace keeps a persistent original backup of {} bytes",source_probe.size_bytes));
        }
        fs::create_dir_all(logs).map_err(|e|e.to_string())?;
        let id=crate::history::HistoryEntry::new_id(&source);
        let log_path=logs.join(format!("{id}.log"));
        result.diagnostics_log_path=Some(log_path.to_string_lossy().into());
        let job=tempfile::Builder::new().prefix(&format!(".sizqo-video-{id}-")).tempdir_in(parent).map_err(|e|e.to_string())?;
        let partial=job.path().join(format!("output.partial.{}",plan.container));
        let stats=job.path().join("pass-stats");
        for pass in 1..=plan.passes {
            if cancel.load(Ordering::Acquire){return Err("Cancelled by user".into());}
            let args=plan.argv(&source,&partial,pass,&stats);
            run(&args,&log_path,&cancel,|parser|{
                let mut event=VideoProgress::new(task_id,session,revision,"encoding");
                parser.decorate(&mut event,pass,plan.passes,source_probe.duration_us);emit(event);
            })?;
        }
        if cancel.load(Ordering::Acquire){return Err("Cancelled by user".into());}
        let encoded=probe::probe(&partial)?;
        super::validation::validate(&source_probe,&encoded,&plan)?;
        // Decode the entire output to null. Exit 0 and a progress=end record alone are insufficient.
        let mut verify=super::process::command("ffmpeg")?;
        verify.args(["-hide_banner","-v","error","-xerror","-nostdin","-protocol_whitelist","file,pipe","-threads"]).arg(plan.threads.to_string())
            .arg("-i").arg(&partial).args(["-map","0:v","-map","0:a?","-threads"]).arg(plan.threads.to_string()).args(["-f","null","-"]);
        // Full read validation uses the same cancellation-aware process pump.
        run_command(verify,&log_path,&cancel,|_|{})?;
        if cancel.load(Ordering::Acquire){return Err("Cancelled by user".into());}
        result.output_bytes=Some(encoded.size_bytes);result.savings=Some((1.0-encoded.size_bytes as f64/source_probe.size_bytes as f64)*100.0);
        if encoded.size_bytes>=source_probe.size_bytes&&!options.allow_larger_output{
            result.stage="skippedLarger".into();result.reason=Some("Valid output is not smaller; original left unchanged".into());return Ok(());
        }
        if encoded.size_bytes>=source_probe.size_bytes{result.warnings.push("Larger output retained by explicit professional-mode choice".into());}
        result.elapsed_ms=started.elapsed().as_millis()as u64;
        transaction::commit(&source,&partial,&target,options,&source_probe,&mut result,history,transactions,retention,&identity)?;
        result.success=true;result.stage="completed".into();Ok(())
    })();
    result.elapsed_ms=started.elapsed().as_millis()as u64;
    if let Err(error)=outcome {result.stage=if cancel.load(Ordering::Acquire){"cancelled"}else{"failed"}.into();result.reason=Some(error);}
    result
}
fn run(args:&[String],log:&Path,cancel:&AtomicBool,emit:impl Fn(&ProgressParser)+Sync)->Result<(),String>{let mut cmd=super::process::command("ffmpeg")?;cmd.args(args);run_command(cmd,log,cancel,emit)}
fn run_command(mut cmd:std::process::Command,log:&Path,cancel:&AtomicBool,emit:impl Fn(&ProgressParser)+Sync)->Result<(),String>{
    let mut child=cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e|e.to_string())?;
    let stdout=child.stdout.take().ok_or("Missing progress pipe")?;
    let stderr=child.stderr.take().ok_or("Missing diagnostic pipe")?;
    std::thread::scope(|scope|{
        let progress=scope.spawn(||{let mut parser=ProgressParser::default();let mut reader=BufReader::new(stdout);let mut line=String::new();loop{line.clear();if reader.read_line(&mut line).unwrap_or(0)==0{break;}if line.len()<=4096&&parser.feed(&line){emit(&parser);}}});
        let diagnostics=scope.spawn(||{
            let mut reader=stderr;let mut buffer=[0u8;8192];let mut ring=std::collections::VecDeque::with_capacity(65536);
            let mut log=fs::OpenOptions::new().create(true).append(true).open(log).ok();
            let mut written=log.as_ref().and_then(|f|f.metadata().ok()).map(|m|m.len()).unwrap_or(0);
            loop{let n=reader.read(&mut buffer).unwrap_or(0);if n==0{break;}
                if let Some(file)=log.as_mut(){let take=(1024*1024u64).saturating_sub(written).min(n as u64)as usize;if take>0{let _=file.write_all(&buffer[..take]);written+=take as u64;}}
                for b in &buffer[..n]{if ring.len()==65536{ring.pop_front();}ring.push_back(*b);}
            }if let Some(file)=log{let _=file.sync_all();}ring.into_iter().collect::<Vec<_>>()
        });
        let status=loop{
            if cancel.load(Ordering::Acquire){
                #[cfg(unix)]unsafe{libc::kill(child.id()as i32,libc::SIGTERM);}
                #[cfg(not(unix))]let _=child.kill();
                let deadline=Instant::now()+Duration::from_secs(2);
                loop{match child.try_wait(){Ok(Some(_))=>break,_=>{}}if Instant::now()>deadline{let _=child.kill();let _=child.wait();break;}std::thread::sleep(Duration::from_millis(30));}
                break Err("Cancelled by user".into());
            }
            match child.try_wait(){Ok(Some(s))=>break Ok(s),Ok(None)=>{},Err(e)=>{let _=child.kill();let _=child.wait();break Err(e.to_string());}}
            std::thread::sleep(Duration::from_millis(50));
        };
        progress.join().map_err(|_|"Progress reader failed")?;
        let errors=diagnostics.join().map_err(|_|"Diagnostic reader failed")?;
        if !status?.success(){return Err(format!("FFmpeg failed: {}",String::from_utf8_lossy(&errors).chars().take(8192).collect::<String>()));}Ok(())
    })
}
