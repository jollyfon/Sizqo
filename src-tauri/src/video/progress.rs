use serde::Serialize;
use super::VideoResult;
#[derive(Debug,Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct VideoProgress {
    pub task_id:String,pub session_id:String,pub queue_revision:u64,pub stage:String,
    pub pass_index:Option<u32>,pub pass_count:Option<u32>,pub media_time_us:Option<u64>,
    pub total_time_us:Option<u64>,pub fraction:Option<f64>,pub speed:Option<f64>,
    pub fps:Option<f64>,pub eta_seconds:Option<f64>,pub bytes_written:Option<u64>,pub result:Option<VideoResult>,
}
impl VideoProgress {
    pub fn new(task:&str,session:&str,revision:u64,stage:&str)->Self{Self{task_id:task.into(),session_id:session.into(),queue_revision:revision,stage:stage.into(),pass_index:None,pass_count:None,media_time_us:None,total_time_us:None,fraction:None,speed:None,fps:None,eta_seconds:None,bytes_written:None,result:None}}
}
#[derive(Default)]pub struct ProgressParser {pub time_us:Option<u64>,pub speed:Option<f64>,pub fps:Option<f64>,pub bytes:Option<u64>}
impl ProgressParser {
    pub fn feed(&mut self,line:&str)->bool{
        let Some((k,v))=line.trim().split_once('=')else{return false};
        match k {
            "out_time_us"=>self.time_us=v.parse::<i64>().ok().map(|t|t.max(0)as u64),
            "out_time"=>{if self.time_us.is_none(){self.time_us=parse_time(v);}},
            "speed"=>self.speed=v.trim_end_matches('x').parse::<f64>().ok().filter(|v|v.is_finite()&&*v>0.0),
            "fps"=>self.fps=v.parse::<f64>().ok().filter(|v|v.is_finite()&&*v>=0.0),
            "total_size"=>self.bytes=v.parse().ok(),"progress"=>return true,_=>{}
        }false
    }
    pub fn decorate(&self,p:&mut VideoProgress,pass:u32,passes:u32,total:Option<u64>){
        p.pass_index=Some(pass);p.pass_count=Some(passes);p.media_time_us=self.time_us;p.total_time_us=total;p.speed=self.speed;p.fps=self.fps;p.bytes_written=self.bytes;
        p.fraction=total.filter(|d|*d>0).zip(self.time_us).map(|(d,t)|((pass-1)as f64+(t as f64/d as f64).min(1.0))/passes as f64).map(|x|x.min(0.99));
        p.eta_seconds=total.zip(self.time_us).zip(self.speed).map(|((d,t),s)|(d.saturating_sub(t)as f64+(passes-pass)as f64*d as f64)/1e6/s);
    }
}
fn parse_time(v:&str)->Option<u64>{let parts:Vec<_>=v.split(':').collect();if parts.len()!=3{return None;}let seconds=parts[0].parse::<f64>().ok()?*3600.0+parts[1].parse::<f64>().ok()?*60.0+parts[2].parse::<f64>().ok()?;if seconds.is_finite()&&seconds>=0.0{Some((seconds*1e6)as u64)}else{None}}
#[cfg(test)]mod tests{use super::*;
    #[test]fn progress_does_not_claim_success(){let mut p=ProgressParser::default();p.feed("out_time_us=5000000");p.feed("speed=2.5x");assert!(p.feed("progress=end"));let mut e=VideoProgress::new("t","s",8,"encoding");p.decorate(&mut e,1,2,Some(10000000));assert_eq!(e.fraction,Some(0.25));assert_eq!(e.eta_seconds,Some(6.0));assert!(e.result.is_none());}
    #[test]fn unknown_duration_and_bad_values(){let mut p=ProgressParser::default();p.feed("speed=NaNx");p.feed("out_time_us=-100");let mut e=VideoProgress::new("t","s",0,"encoding");p.decorate(&mut e,1,1,None);assert_eq!(e.fraction,None);assert_eq!(e.speed,None);assert_eq!(e.media_time_us,Some(0));}
}
