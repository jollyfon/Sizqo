//! Media IPC uses one execution session and one frozen input snapshot.
use std::{collections::{HashMap,HashSet},path::{Path,PathBuf},sync::{Arc,Mutex,atomic::{AtomicBool,Ordering}}};
use serde::{Serialize,Deserialize};
use tauri::{AppHandle,State,Manager,Emitter};
use crate::{commands::{AppState,AcquireOutcome},video::{self,VideoOptions,VideoResult,probe::VideoProbe,progress::VideoProgress}};
#[derive(Default)]pub struct VideoRuntime {pub cancellations:Mutex<HashMap<(String,String),Arc<AtomicBool>>>}
#[derive(Debug,Clone,Deserialize)]#[serde(rename_all="camelCase")]
pub struct VideoTask {pub task_id:String,pub path:String}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct ImportedMedia {path:String,name:String,size:u64,media_kind:String,video_probe:Option<VideoProbe>,warning:Option<String>}
fn image_header(path:&Path)->bool {
    use std::io::Read;let mut b=[0u8;64];let n=std::fs::File::open(path).and_then(|mut f|f.read(&mut b)).unwrap_or(0);let b=&b[..n];
    image::guess_format(b).is_ok() || b.starts_with(&[0xff,0x0a]) || b.starts_with(b"\0\0\0\x0cJXL ") ||
        (b.get(4..8)==Some(b"ftyp")&&b.get(8..12).map(|s|[b"avif" as &[u8],b"avis",b"heic",b"heix",b"mif1"].contains(&s)).unwrap_or(false))
}
fn collect(path:&Path,seen:&mut HashSet<PathBuf>,out:&mut Vec<PathBuf>){
    let Ok(real)=path.canonicalize()else{return};if !seen.insert(real.clone()){return;}
    if real.is_dir(){if let Ok(children)=std::fs::read_dir(&real){let mut children:Vec<_>=children.flatten().map(|e|e.path()).collect();children.sort();for p in children{collect(&p,seen,out);}}}else if real.is_file(){out.push(real);}
}
#[tauri::command]
pub async fn probe_media(paths:Vec<String>,state:State<'_,AppState>)->Result<Vec<ImportedMedia>,String>{
    state.access.remember_all(&paths);let access=state.access.clone();
    tauri::async_runtime::spawn_blocking(move||{
        let mut guards=vec![];for path in &paths{guards.push(access.acquire(Path::new(path),&mut||false).ok_or_else(||format!("Select the folder again to grant access: {path}"))?);}
        let mut files=vec![];let mut seen=HashSet::new();for p in paths{collect(Path::new(&p),&mut seen,&mut files);}
        let mut out=vec![];for path in files {
            let size=std::fs::metadata(&path).map(|m|m.len()).unwrap_or(0);let name=path.file_name().unwrap_or_default().to_string_lossy().into_owned();
            if image_header(&path){out.push(ImportedMedia{path:path.to_string_lossy().into(),name,size,media_kind:"image".into(),video_probe:None,warning:None});}
            else {match video::probe::probe(&path){Ok(p)=>out.push(ImportedMedia{path:path.to_string_lossy().into(),name,size,media_kind:"video".into(),video_probe:Some(p),warning:None}),Err(error)=>{
                if video::is_video_extension(&path){out.push(ImportedMedia{path:path.to_string_lossy().into(),name,size,media_kind:"video".into(),video_probe:None,warning:Some(error)});}
            }}}
        }Ok::<_,String>(out)
    }).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub async fn get_video_capabilities()->Result<video::capabilities::CapabilityMatrix,String>{tauri::async_runtime::spawn_blocking(video::capabilities::capabilities).await.map_err(|e|e.to_string())}
#[tauri::command]
pub async fn get_video_presets(app:AppHandle)->Result<Vec<video::presets::VideoPreset>,String>{
    let c=get_video_capabilities().await?;let mut out=video::presets::presets(&c);
    let path=app.path().app_data_dir().map_err(|e|e.to_string())?.join("video-presets.json");
    if path.exists() {
        let entries:HashMap<String,VideoOptions>=serde_json::from_slice(&std::fs::read(path).map_err(|e|e.to_string())?).map_err(|e|format!("Preset file is damaged; preserved unchanged: {e}"))?;
        for (name,options) in entries {let available=c.available&&(options.encoder=="auto"||c.encoder(&options.encoder).is_some());
            out.push(video::presets::VideoPreset{id:format!("user:{name}"),name,description:"Saved custom preset; options are validated against each source".into(),available,reason:if available{None}else{Some("Encoder unavailable in this edition".into())},options});
        }
    }Ok(out)
}
#[tauri::command]
pub async fn validate_video_options(options:VideoOptions,probe:VideoProbe,state:State<'_,AppState>)->Result<video::plan::EncodePlan,String>{let c=get_video_capabilities().await?;video::plan::build(&options,&probe,&c,state.compression.max_parallelism())}
#[tauri::command]
pub fn save_video_preset(name:String,options:VideoOptions,app:AppHandle)->Result<(),String>{
    // Names never become filesystem paths; all presets share one atomic store.
    if name.trim().is_empty()||name.chars().count()>80{return Err("Preset name must contain 1…80 characters".into());}
    let path=app.path().app_data_dir().map_err(|e|e.to_string())?.join("video-presets.json");
    static LOCK:Mutex<()>=Mutex::new(());let _lock=LOCK.lock().map_err(|e|e.to_string())?;
    let mut entries:HashMap<String,VideoOptions>=match std::fs::read(&path){Ok(b)=>serde_json::from_slice(&b).map_err(|e|format!("Preset file is damaged; preserved unchanged: {e}"))?,Err(e)if e.kind()==std::io::ErrorKind::NotFound=>HashMap::new(),Err(e)=>return Err(e.to_string())};
    if entries.len()>=100&&!entries.contains_key(&name){return Err("Preset limit reached".into());}entries.insert(name,options);crate::history::write_atomic(&path,&serde_json::to_vec_pretty(&entries).map_err(|e|e.to_string())?)
}
#[tauri::command]
pub fn cancel_video_task(task_id:String,session_id:String,app:AppHandle,state:State<'_,AppState>)->bool{
    let runtime=app.state::<VideoRuntime>();let tokens=runtime.cancellations.lock().unwrap();
    if let Some(token)=tokens.get(&(session_id,task_id)){token.store(true,Ordering::Release);state.compression.wake_waiters();true}else{false}
}
async fn video_batch(app:&AppHandle,state:&AppState,tasks:Vec<VideoTask>,options:VideoOptions,session:String,revision:u64,roots:Vec<String>)->Result<Vec<VideoResult>,String>{
    let runtime=app.state::<VideoRuntime>();
    let mut seen=HashSet::new();let mut tokens=vec![];
    for task in tasks{let path=PathBuf::from(&task.path).canonicalize().map_err(|e|e.to_string())?;
        if !seen.insert(path.clone()){continue;}let token=Arc::new(AtomicBool::new(false));
        runtime.cancellations.lock().unwrap().insert((session.clone(),task.task_id.clone()),token.clone());tokens.push((task,path,token));}
    let caps=get_video_capabilities().await?;
    let mut results=vec![];
    for (task,path,cancel) in tokens {
        let emit=|event|{let _=app.emit("video-progress",event);};
        emit(VideoProgress::new(&task.task_id,&session,revision,"queued"));
        let permit=match state.compression.acquire(||cancel.load(Ordering::Acquire)||state.cancel_queue.lock().unwrap().contains(&task.path)).await{
            AcquireOutcome::Acquired(p)=>p,
            AcquireOutcome::Stopped=>{emit(VideoProgress::new(&task.task_id,&session,revision,"deferred"));runtime.cancellations.lock().unwrap().remove(&(session.clone(),task.task_id.clone()));continue;},
            AcquireOutcome::Cancelled=>{emit(VideoProgress::new(&task.task_id,&session,revision,"cancelled"));runtime.cancellations.lock().unwrap().remove(&(session.clone(),task.task_id.clone()));continue;},
        };
        emit(VideoProgress::new(&task.task_id,&session,revision,"starting"));
        #[cfg(not(feature="inproc-backends"))]
        let result={
            let history=state.history_store.clone();let transactions=state.transactions.clone();let retention=state.settings_store.load().original_retention_days;let budget=state.compression.max_parallelism();
            let logs=app.path().app_data_dir().map_err(|e|e.to_string())?.join("video-logs");let options=options.clone();let caps=caps.clone();let session=session.clone();let roots=roots.clone();let app=app.clone();let task_id=task.task_id.clone();let access=state.access.clone();
            tauri::async_runtime::spawn_blocking(move||{
                let _scope=access.acquire(&path,&mut||false);video::ffmpeg_backend::encode(&task_id,&path,&options,&caps,&session,revision,&roots,&history,&transactions,retention,budget,&logs,cancel,&|event|{let _=app.emit("video-progress",event);})
            }).await.map_err(|e|e.to_string())?
        };
        #[cfg(feature="inproc-backends")]
        let result={let _=(&caps,&roots);let mut r=VideoResult::pending(&task.task_id,&path,&options);r.backend_actual="unavailable".into();r.reason=Some(video::capabilities::NATIVE_UNAVAILABLE.into());r};
        drop(permit);let mut event=VideoProgress::new(&task.task_id,&session,revision,&result.stage);event.fraction=if result.success{Some(1.0)}else{None};event.result=Some(result.clone());emit(event);
        results.push(result);runtime.cancellations.lock().unwrap().remove(&(session.clone(),task.task_id));
    }Ok(results)
}
struct SessionGuard {app:AppHandle,scheduler:Arc<crate::commands::CompressionScheduler>}
impl SessionGuard {fn begin(app:&AppHandle,state:&AppState)->Result<Self,String>{
    if !state.compression.try_begin_batch(){return Err("Another media session is already running".into());}
    let settings=state.settings_store.load();state.compression.set_max_parallelism(crate::app_settings::effective_cpu_limit(settings.cpu_thread_limit,state.cpu_info.budget_ceiling()));crate::commands::emit_compression_state(app,&state.compression);Ok(Self{app:app.clone(),scheduler:state.compression.clone()})}}
impl Drop for SessionGuard{fn drop(&mut self){self.scheduler.end_batch();crate::commands::emit_compression_state(&self.app,&self.scheduler);self.app.state::<VideoRuntime>().cancellations.lock().unwrap().clear();self.app.state::<AppState>().cancel_queue.lock().unwrap().clear();}}
#[tauri::command]
pub async fn start_video_session(app:AppHandle,state:State<'_,AppState>,tasks:Vec<VideoTask>,options:VideoOptions,session_id:String,queue_revision:u64,source_roots:Vec<String>)->Result<Vec<VideoResult>,String>{let _session=SessionGuard::begin(&app,state.inner())?;video_batch(&app,state.inner(),tasks,options,session_id,queue_revision,source_roots).await}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct MediaSessionResult {session_id:String,image_results:Vec<crate::engine::CompressResult>,video_results:Vec<VideoResult>}
#[tauri::command]
pub async fn start_media_session(app:AppHandle,state:State<'_,AppState>,images:Vec<String>,video_tasks:Vec<VideoTask>,image_options:crate::engine::CompressOptions,video_options:VideoOptions,session_id:String,queue_revision:u64,source_roots:Vec<String>,use_smart:bool)->Result<MediaSessionResult,String>{
    let _access=crate::commands::prepare_compression_access(&app,state.inner(),&images,&image_options)?;
    let _session=SessionGuard::begin(&app,state.inner())?;
    let image_results=if images.is_empty(){vec![]}else{crate::commands::compress_batch_with_session(&app,state.inner(),session_id.clone(),queue_revision,images,image_options,use_smart,false).await.results};
    let video_results=video_batch(&app,state.inner(),video_tasks,video_options,session_id.clone(),queue_revision,source_roots).await?;
    Ok(MediaSessionResult{session_id,image_results,video_results})
}
#[tauri::command]
pub async fn open_media_file(path:String,app:AppHandle,state:State<'_,AppState>)->Result<(),String>{use tauri_plugin_opener::OpenerExt;
    let path=Path::new(&path).canonicalize().map_err(|e|e.to_string())?;let _scope=state.access.acquire(&path,&mut||false).ok_or("File authorization expired; select it again")?;
    if !path.is_file(){return Err("Media file no longer exists".into());}app.opener().open_path(path.to_string_lossy(),None::<&str>).map_err(|e|e.to_string())
}
#[tauri::command]
pub async fn preview_video_frame(path:String,time_us:u64,state:State<'_,AppState>)->Result<String,String>{
    let access=state.access.clone();tauri::async_runtime::spawn_blocking(move||frame(&path,time_us,access.as_ref())).await.map_err(|e|e.to_string())?
}
#[cfg(not(feature="inproc-backends"))]
fn frame(path:&str,time_us:u64,access:&dyn crate::sandbox_access::FileAccess)->Result<String,String>{
    use base64::Engine;let path=Path::new(path).canonicalize().map_err(|e|e.to_string())?;let _scope=access.acquire(&path,&mut||false).ok_or("Video access denied")?;
    let p=video::probe::probe(&path)?;let t=time_us.min(p.duration_us.map(|d|d.saturating_sub(1000)).unwrap_or(time_us));
    let mut cmd=video::process::command("ffmpeg")?;
    cmd.args(["-hide_banner","-v","error","-nostdin","-protocol_whitelist","file,pipe","-threads","1","-ss"]).arg(format!("{:.6}",t as f64/1e6)).arg("-i").arg(path)
        .args(["-map","0:v:0","-frames:v","1","-filter_threads","1","-vf","scale=1280:1280:force_original_aspect_ratio=decrease","-threads","1","-f","image2pipe","-vcodec","mjpeg","pipe:1"]);
    let b=video::process::capture(cmd,std::time::Duration::from_secs(30),4*1024*1024)?;if b.is_empty(){return Err("Preview frame unavailable".into());}Ok(format!("data:image/jpeg;base64,{}",base64::engine::general_purpose::STANDARD.encode(b)))
}
#[cfg(feature="inproc-backends")]
fn frame(_path:&str,_time:u64,_access:&dyn crate::sandbox_access::FileAccess)->Result<String,String>{Err(video::capabilities::NATIVE_UNAVAILABLE.into())}
#[tauri::command]
pub async fn compare_video_frames(source_path:String,output_path:String,time_us:u64,state:State<'_,AppState>)->Result<serde_json::Value,String>{
    let access=state.access.clone();tauri::async_runtime::spawn_blocking(move||{
        let a=video::probe::probe(Path::new(&source_path))?;let b=video::probe::probe(Path::new(&output_path))?;
        let time=time_us.min(a.duration_us.unwrap_or(time_us).min(b.duration_us.unwrap_or(time_us)).saturating_sub(1000));
        let source_frame=frame(&source_path,time,access.as_ref())?;let output_frame=frame(&output_path,time,access.as_ref())?;
        Ok::<_,String>(serde_json::json!({"sourceFrame":source_frame,"outputFrame":output_frame,"timeUs":time}))
    }).await.map_err(|e|e.to_string())?
}
