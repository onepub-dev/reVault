#! /usr/bin/env dcli
import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'package:dcli/dcli.dart';
const worktree='/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance';
const evidence='$worktree/docs/evidence/explicit-file-extraction-2026-10-12';
const fixtureEvidence='$worktree/docs/evidence/native-public-extraction-2026-10-11/raw';
const raw='$evidence/raw';
const owned='/home/bsutton/.cache/rv-single-20261012';
const cli='$worktree/rust/target/credential-open-20261012/bin/lockbox-new';
const expectedCli='b0fb323753105217dd780e6bc3cdfb8e29af4b5f9044ef6a9835c63d913d994b';
final wall=Stopwatch()..start();int counter=0;
void require(bool b,String m){if(!b)throw StateError(m);}
Map<String,dynamic> object(String p)=>jsonDecode(File(p).readAsStringSync()) as Map<String,dynamic>;
void save(String p,Object v){File(p).parent.createSync(recursive:true);File(p).writeAsStringSync('${const JsonEncoder.withIndent('  ').convert(v)}\n',flush:true);}
String sha(String p){final r=Process.runSync('/usr/bin/sha256sum',['--',p]);require(r.exitCode==0,'hash');return (r.stdout as String).split(' ').first;}
Map<String,String> environment(int c){final p='$owned/c$c';final e=<String,String>{'PATH':'/usr/bin:/bin','TMPDIR':'/home/bsutton/.cache/rv4-security-20261011','HOME':'$p/home','XDG_CONFIG_HOME':'$p/config','XDG_CACHE_HOME':'$p/cache','XDG_STATE_HOME':'$p/state','XDG_DATA_HOME':'$p/data','LOCKBOX_SESSION_AGENT_DIR':'$p/agent','LOCKBOX_SESSION_AGENT_LOG':'$p/agent.log','LOCKBOX_VAULT_DIR':'$p/vault','LOCKBOX_PLATFORM_SECRET_STORE':'disabled','LOCKBOX_ADD_PROGRESS':'off'};for(final k in ['HOME','XDG_CONFIG_HOME','XDG_CACHE_HOME','XDG_STATE_HOME','XDG_DATA_HOME','LOCKBOX_SESSION_AGENT_DIR','LOCKBOX_VAULT_DIR'])Directory(e[k]!).createSync(recursive:true);require(utf8.encode('${e['LOCKBOX_SESSION_AGENT_DIR']}/agent.sock').length<=100,'socket');return e;}
ProcessResult run(List<String> argv,Map<String,String> env,String label,{int maximum=180,bool mustPass=true,bool cleanup=false}){final remaining=cleanup?maximum:1800-wall.elapsed.inSeconds-60;require(remaining>0,'30minute cap');final seconds=remaining<maximum?remaining:maximum;final command=['--kill-after=10s','$seconds',...argv];final r=Process.runSync('/usr/bin/timeout',command,environment:env,includeParentEnvironment:false);final stem='$raw/commands/${(counter++).toString().padLeft(5,'0')}-$label';File(stem).parent.createSync(recursive:true);File('$stem.stdout').writeAsStringSync(r.stdout as String);File('$stem.stderr').writeAsStringSync(r.stderr as String);save('$stem.json',{'argv':['/usr/bin/timeout',...command],'exit_code':r.exitCode,'timed_out':r.exitCode==124||r.exitCode==137});if(mustPass)require(r.exitCode==0,'Command failed: $label, retainedlogs');return r;}
int diskBytes(String path){final d=Directory(path);if(!d.existsSync())return 0;var n=0;for(final f in d.listSync(recursive:true,followLinks:false)){if(f is File)n+=f.lengthSync();}return n;}
void budget(int next){final roots=[owned,'$worktree/rust/target/explicit-file-extraction-20261012/bin','/home/bsutton/.cache/rv-many-20261012','/home/bsutton/.cache/rv-many-zstd-20261012','/home/bsutton/.cache/revault-native-public-20261011','/home/bsutton/.cache/rv-credential-20261012','$worktree/rust/target/credential-open-20261012/bin','$worktree/rust/target/parallel-extraction-202612/bin','$worktree/rust/target/native-public-20261011/bin','$worktree/rust/target/native-extract-cache-budget-20261011/bin',evidence];final total=roots.map(diskBytes).fold<int>(0,(a,b)=>a+b);require(total+next<=24*1024*1024*1024,'24GiB ownedcap');final r=Process.runSync('/usr/bin/df',['-Pk',owned]);require(r.exitCode==0,'df');final fields=(r.stdout as String).trim().split('\n').last.split(RegExp(r'\s+'));require(int.parse(fields[3])*1024>=next+2*1024*1024*1024,'headroom');}
void agent(int c,{required bool start}){final e=environment(c);final socket='${e['LOCKBOX_SESSION_AGENT_DIR']}/agent.sock';if(start){final r=run([cli,'session','--format','json'],e,'session$c',maximum:20);final rows=(r.stdout as String).split('\n').where((v)=>v.trim().isNotEmpty).map(jsonDecode);require(rows.any((v)=>v is Map&&v['kind']=='agent'&&v['state']=='running'&&v['value']=='yes'),'agent roundtrip');run(['/usr/bin/test','-S',socket],e,'socket$c',maximum:5);}else{final present=run(['/usr/bin/test','-S',socket],e,'cleanup-present$c',maximum:5,mustPass:false,cleanup:true);if(present.exitCode==1)return;require(present.exitCode==0,'Socket inspection failed');run([cli,'session','stop'],e,'stop$c',maximum:20,cleanup:true);run(['/usr/bin/test','!','-S',socket],e,'stopped$c',maximum:5,cleanup:true);}}
void cleanupAgents(List<int> cases){final results=<Map<String,dynamic>>[];for(final c in cases){try{agent(c,start:false);results.add({'case':c,'stopped':true});}catch(error){results.add({'case':c,'stopped':false,'error':'$error'});}}save('$raw/cleanup-${DateTime.now().microsecondsSinceEpoch}.json',{'rows':results,'passed':results.every((r)=>r['stopped']==true)});require(results.every((r)=>r['stopped']==true),'Owned cleanup incomplete');}
List<Map<String,dynamic>> cases()=>(jsonDecode(File('$raw/cases.json').readAsStringSync()) as List).cast<Map<String,dynamic>>();
void identities(){for(final e in (object('$raw/baseline-identities.json')['hashes'] as Map).entries)require(sha(e.key)==e.value,'Changed identity');}
void cold(String archive,Map<String,String> env,String id){for(var a=0;a<3;a++){run(['/usr/bin/sync','-d','--',archive],env,'$id-sync$a',maximum:30);run(['/usr/bin/fadvise','--advice','dontneed',archive],env,'$id-evict$a',maximum:30);final r=run(['/usr/bin/fincore','--json','--bytes','--output','RES,FILE',archive],env,'$id-res$a',maximum:30);final rows=jsonDecode(r.stdout)['fincore'] as List;if(int.parse('${rows.single['res']}')==0)return;}throw StateError('Cold gate failed');}
void batch(){identities();final output=File('$raw/baseline-samples.jsonl');require(!output.existsSync(),'Existing batch');final specs=cases();final started=<int>[];var accepted=0;try{for(var c=0;c<2;c++){started.add(c);agent(c,start:true);final spec=specs[c];final env=environment(c);for(var r=0;r<4;r++){final id='c$c-r$r';final destination='${spec['root']}/output-$id';require(FileSystemEntity.typeSync(destination,followLinks:false)==FileSystemEntityType.notFound,'Output exists');budget(spec['bytes'] as int);cold(spec['archive'] as String,env,id);final timing='$raw/$id.time';run(['/usr/bin/time','-f','%e %U %S %M','-o',timing,'/usr/bin/taskset','-c','0-4',cli,spec['archive'] as String,'extract',spec['stored_path'] as String,destination],env,id);final metrics=File(timing).readAsStringSync().trim().split(RegExp(r'\s+')).map(double.parse).toList();require(metrics.length==4&&metrics.every((v)=>v.isFinite&&v>=0)&&metrics[0]>0&&metrics[3]>0,'Metrics');verifyFile(spec,destination,env,id);File(destination).deleteSync();output.writeAsStringSync('${jsonEncode({'case':spec['case'],'round':r,'preparation':r==0,'elapsed_seconds':metrics[0],'user_seconds':metrics[1],'system_seconds':metrics[2],'rss_bytes':metrics[3]*1024,'persisted_bytes_mode_verified':true,'cold_archive':true,'output_drained':true,'executable':cli})}\n',mode:FileMode.append,flush:true);accepted++;}}}finally{cleanupAgents(started);}identities();require(accepted==8,'Exact8');save('$raw/baseline-complete.json',{'accepted':accepted,'identities_verified_after':true,'owned_agents_stopped':true,'wall_seconds':wall.elapsedMilliseconds/1000});}
void verifyFile(Map<String,dynamic> spec,String output,Map<String,String> env,String id){
 require(FileSystemEntity.typeSync(output,followLinks:false)==FileSystemEntityType.file,'Regular output');
 final f=File(output);require(f.lengthSync()==spec['bytes'],'Exact output size');require((f.statSync().mode&0x1ff)==0x1a4,'Default file0644');
 run(['/usr/bin/cmp','-s','--',spec['source'] as String,output],env,'$id-byte-compare');
 run(['/usr/bin/sync','-d','--',output,f.parent.path],env,'$id-drain',maximum:60);
}
Future<void> setup()async{
 require(!File('$raw/cases.json').existsSync(),'Existing setup');Directory(owned).createSync(recursive:true);require(sha(cli)==expectedCli,'Pinned restored CLI');
 final previous=(jsonDecode(File('$fixtureEvidence/cases.json').readAsStringSync()) as List).where((v)=>v['bytes']==1073741824).toList();require(previous.length==2,'Two large source/raw cases');
 final inventory=object('$fixtureEvidence/pre-timing-identities.json')['archive_inventory'] as List;
 final records=<Map<String,dynamic>>[];final started=<int>[];
 try{for(var c=0;c<2;c++){
  final input=previous[c];final sourceRow=(input['variants'] as List).singleWhere((v)=>v['variant']=='A');final pin=inventory.singleWhere((v)=>v['case']==input['name']&&v['variant']=='A');
  final archive=pin['path'] as String;final source=sourceRow['source_path'] as String;
  require(sha(archive)==pin['sha256']&&File(archive).lengthSync()==pin['bytes'],'Pinned archive');require(sha(source)==sourceRow['source_sha256']&&File(source).lengthSync()==input['bytes'],'Pinned source');require(sha('${sourceRow['root']}/case.json')==sourceRow['case_json_sha256'],'Pinned case metadata');
  final fixture='$owned/fixture$c';require(!Directory(fixture).existsSync(),'Existing partial setup');Directory(fixture).createSync(recursive:true);final env=environment(c);started.add(c);agent(c,start:true);
  final listing=run([cli,archive,'list','--recursive','--format','json'],env,'list$c');final rows=(listing.stdout as String).split('\n').where((v)=>v.trim().isNotEmpty).map(jsonDecode).where((v)=>v['kind']=='file').toList();require(rows.length==1&&int.parse('${rows.single['len']}')==input['bytes'],'Public single-file listing');final path=rows.single['path'] as String;require(path.startsWith('/'),'Canonical listed path');
  final spec=<String,dynamic>{'case':input['name'],'root':fixture,'archive':archive,'archive_sha256':pin['sha256'],'archive_bytes':pin['bytes'],'source':source,'source_sha256':sourceRow['source_sha256'],'bytes':input['bytes'],'stored_path':path,'stored_path_provenance':'public CLI recursive JSON listing','prior_case_sha256':sourceRow['case_json_sha256']};
  budget(input['bytes'] as int);final output='$fixture/setup-output';run([cli,archive,'extract',path,output],env,'setup-extract$c');verifyFile(spec,output,env,'setup$c');File(output).deleteSync();records.add(spec);
 }}finally{cleanupAgents(started);}save('$raw/cases.json',records);print(green('Pinned explicit-file fixtures independently verified.'));
}
void freeze(){
 require(!File('$raw/baseline-identities.json').existsSync(),'Existing freeze');require(sha(cli)==expectedCli,'Pinned baseline');final hashes=<String,String>{};
 for(final p in [cli,Platform.script.toFilePath(),'$evidence/README.md','$raw/cases.json','$fixtureEvidence/pre-timing-identities.json','$fixtureEvidence/cases.json','$worktree/docs/evidence/credential-open-2026-10-12/raw/build-new/manifest.txt'])hashes[p]=sha(p);
 for(final spec in cases()){for(final pair in [['archive','archive_sha256'],['source','source_sha256']]){final path=spec[pair[0]] as String;require(sha(path)==spec[pair[1]],'Fixture changed');hashes[path]=spec[pair[1]] as String;}}
 save('$raw/baseline-identities.json',{'hashes':hashes,'planned_processes':8,'umask':'0022','runtime':'Restored a881d15d semantics after37788a78 worker revert; preserved b0fb CLI and original credential build provenance, no rebuild','branch':'public extract stored_path destination -> File -> ReadMostly'});
}

const trialCli='$worktree/rust/target/explicit-file-extraction-20261012/bin/lockbox-new';
const expectedTrial='85caddce82608beace258c8f507f8d33b73d27541d0ccbbfea0ea5ff648a273d';
const runtimeFiles=['revault_cli/src/commands/files.rs','revault_cli/Cargo.toml'];
void freezePaired(){
 require(!File('$raw/paired-identities.json').existsSync(),'Existing paired freeze');
 require(sha(cli)==expectedCli&&sha(trialCli)==expectedTrial,'Pinned old/new executables');
 final hashes=<String,String>{};
 for(final e in (object('$raw/baseline-identities.json')['hashes'] as Map).entries){
  if(e.key==Platform.script.toFilePath()||e.key=='$evidence/README.md')continue;
  require(sha(e.key)==e.value,'Baseline fixture identity changed');hashes[e.key]=e.value;
 }
 final manifest='$raw/build-new/manifest.txt';final build=File(manifest).readAsStringSync();
 require(build.contains('$expectedTrial  target/explicit-file-extraction-20261012/bin/lockbox-new'),'Build executable identity');
 for(final relative in runtimeFiles){final p='$worktree/rust/$relative';final digest=sha(p);require(build.contains('$digest  $relative'),'Build/runtime source mismatch');hashes[p]=digest;}
 for(final p in [trialCli,Platform.script.toFilePath(),'$evidence/README.md',manifest,'$raw/baseline-identities.json','$worktree/tools/summarize-explicit-file-extraction.dart','$worktree/tools/summarize-adaptive-workers.dart','$raw/baseline-runner.dart.gz','$raw/baseline-protocol.md'])hashes[p]=sha(p);
 save('$raw/paired-identities.json',{'hashes':hashes,'planned_processes':56,'preparation_processes':8,'measured_processes':48,'umask':'0022','base':'37788a7813649c994f97ce110a8638fdd165e837','variants':{'old':cli,'new':trialCli},'decision':'every new measured RSS below100000000bytes; aggregate paired elapsed95%upper at most1.05; exact output/identity gates'});
}
void pairedIdentities(){for(final e in (object('$raw/paired-identities.json')['hashes'] as Map).entries)require(sha(e.key)==e.value,'Changed paired identity');}
void pairedBatch(){
 pairedIdentities();final output=File('$raw/paired-samples.jsonl');require(!output.existsSync(),'Existing paired batch');final specs=cases();final started=<int>[];var accepted=0;
 try{for(var c=0;c<2;c++){started.add(c);agent(c,start:true);final spec=specs[c];final env=environment(c);
  for(var r=0;r<14;r++){final order=(r+c)%2==0?['old','new']:['new','old'];
   for(var position=0;position<2;position++){final variant=order[position];final executable=variant=='old'?cli:trialCli;final id='paired-c$c-r$r-p$position-$variant';final destination='${spec['root']}/output-$id';require(FileSystemEntity.typeSync(destination,followLinks:false)==FileSystemEntityType.notFound,'Output exists');budget(spec['bytes'] as int);cold(spec['archive'] as String,env,id);final timing='$raw/$id.time';
    run(['/usr/bin/time','-f','%e %U %S %M','-o',timing,'/usr/bin/taskset','-c','0-4',executable,spec['archive'] as String,'extract',spec['stored_path'] as String,destination],env,id);
    final metrics=File(timing).readAsStringSync().trim().split(RegExp(r'\s+')).map(double.parse).toList();require(metrics.length==4&&metrics.every((v)=>v.isFinite&&v>=0)&&metrics[0]>0&&metrics[3]>0,'Metrics');verifyFile(spec,destination,env,id);File(destination).deleteSync();
    output.writeAsStringSync('${jsonEncode({'case':spec['case'],'round':r,'position':position,'variant':variant,'preparation':r<2,'elapsed_seconds':metrics[0],'user_seconds':metrics[1],'system_seconds':metrics[2],'rss_bytes':metrics[3]*1024,'persisted_bytes_mode_verified':true,'cold_archive':true,'output_drained':true,'executable':executable})}\n',mode:FileMode.append,flush:true);accepted++;
   }
  }
 }}finally{cleanupAgents(started);}pairedIdentities();require(accepted==56,'Exact56');save('$raw/paired-complete.json',{'accepted':accepted,'measured':48,'preparation':8,'identities_verified_after':true,'owned_agents_stopped':true,'wall_seconds':wall.elapsedMilliseconds/1000});
}


Future<void> main(List<String> args)async{require(Platform.isLinux&&args.length==1,'Linux setup | freeze | baseline');final setMask=DynamicLibrary.process().lookupFunction<Uint32 Function(Uint32),int Function(int)>('umask');final previous=setMask(0x12);try{Directory(raw).createSync(recursive:true);counter=Directory('$raw/commands').existsSync()?Directory('$raw/commands').listSync().whereType<File>().length:0;switch(args.single){case 'setup':await setup();break;case 'freeze':freeze();break;case 'baseline':batch();break;case 'freeze-paired':freezePaired();break;case 'paired':pairedBatch();break;default:throw ArgumentError('Unknown phase');}}finally{setMask(previous);}}
