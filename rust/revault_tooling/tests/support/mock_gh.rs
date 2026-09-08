//! Subprocess fixture: no network requests or remote mutations are possible.
use std::{env,fs,io::{Read,Write},path::PathBuf};
fn main(){
 let args:Vec<String>=env::args().skip(1).collect();
 let root=PathBuf::from(env::var_os("RELEASE_FIXTURE").expect("fixture root"));
 writeln!(fs::OpenOptions::new().create(true).append(true).open(root.join("calls")).unwrap(),"{}",args.join(" ")).unwrap();
 let get=|name:&str|print!("{}",fs::read_to_string(root.join(name)).unwrap());
 match args[0].as_str(){
 "repo"=>println!("org/repo"),
 "api"=>{
  if args.iter().any(|a|a=="POST") {let mut body=String::new();std::io::stdin().read_to_string(&mut body).unwrap();fs::write(root.join("dispatch.json"),body).unwrap();if root.join("dispatch-response.json").exists(){fs::copy(root.join("dispatch-response.json"),root.join("runs.json")).unwrap();}return;}
  let path=&args[1];
  if path.contains("/jobs?"){if root.join("jobs.json").exists(){get("jobs.json");}else{println!("{{\"jobs\":[]}}");}}
  else if path.ends_with("/logs"){if root.join("empty-logs").exists(){return;}println!("error: lock file needs to be updated but --locked was passed");}
  else if path.contains("/artifacts?"){get("artifacts.json");}
  else if path.ends_with("/runs/99"){println!("{{\"status\":\"completed\",\"conclusion\":\"success\"}}");}
  else if path.ends_with("/runs/42"){get("run.json");}
  else if path.contains("/workflows/"){get("runs.json");}
  else if path.contains("git/ref/heads/"){get("ref.json");}
  else {panic!("unexpected API: {path}");}
 },
 "run"=> match args[1].as_str(){
  "download"=>{let i=args.iter().position(|a|a=="--dir").unwrap();fs::copy(root.join("candidate.json"),PathBuf::from(&args[i+1]).join("candidate.json")).unwrap();},
  "watch"=>{if root.join("watch-fails").exists(){std::process::exit(1);}},
  "rerun"=>{},
  "cancel"=>{},
  _=>panic!("unexpected run command")
 },
 _=>panic!("unexpected gh invocation")
 }
}
