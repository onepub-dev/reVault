#! /usr/bin/env dcli
import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'package:dcli/dcli.dart';
const worktree='/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance';
const evidence='$worktree/docs/evidence/import-memory-attribution-2026-10-12';
const fixtureEvidence='$worktree/docs/evidence/native-public-extraction-2026-10-11/raw';
const raw='$evidence/raw';
const owned='/home/bsutton/.cache/rv-memdiag-20261012';
const cli='$worktree/rust/target/public-zip-comparison-20261012/bin/lockbox-current';
const expectedCli='08bf819f90c3b77a7e69f0639c7bd5f73dde275b6e13f0d443a3285fe98d00f2';
const instrumented='$worktree/rust/target/import-memory-attribution-20261012/bin/lockbox-instrumented';
const expectedInstrumented='84bf1bf67210a9f62d2b2b4d0c2a6591b3330ee236da9b3e898cfe286584227b';
final wall=Stopwatch()..start();int counter=0;
void require(bool b,String m){if(!b)throw StateError(m);}
Map<String,dynamic> object(String p)=>jsonDecode(File(p).readAsStringSync()) as Map<String,dynamic>;
void save(String p,Object v){File(p).parent.createSync(recursive:true);File(p).writeAsStringSync('${const JsonEncoder.withIndent('  ').convert(v)}\n',flush:true);}
String sha(String p){final r=Process.runSync('/usr/bin/sha256sum',['--',p]);require(r.exitCode==0,'hash');return (r.stdout as String).split(' ').first;}
Map<String,String> environment(int c){final p='$owned/c$c';final e=<String,String>{'PATH':'/usr/bin:/bin','TMPDIR':'/home/bsutton/.cache/rv4-security-20261011','HOME':'$p/home','XDG_CONFIG_HOME':'$p/config','XDG_CACHE_HOME':'$p/cache','XDG_STATE_HOME':'$p/state','XDG_DATA_HOME':'$p/data','LOCKBOX_SESSION_AGENT_DIR':'$p/agent','LOCKBOX_SESSION_AGENT_LOG':'$p/agent.log','LOCKBOX_VAULT_DIR':'$p/vault','LOCKBOX_PLATFORM_SECRET_STORE':'disabled','LOCKBOX_ADD_PROGRESS':'off'};for(final k in ['HOME','XDG_CONFIG_HOME','XDG_CACHE_HOME','XDG_STATE_HOME','XDG_DATA_HOME','LOCKBOX_SESSION_AGENT_DIR','LOCKBOX_VAULT_DIR'])Directory(e[k]!).createSync(recursive:true);require(utf8.encode('${e['LOCKBOX_SESSION_AGENT_DIR']}/agent.sock').length<=100,'socket');return e;}
ProcessResult run(List<String> argv,Map<String,String> env,String label,{int maximum=180,bool mustPass=true,bool cleanup=false}){final remaining=cleanup?maximum:1800-wall.elapsed.inSeconds-60;require(remaining>0,'30minute cap');final seconds=remaining<maximum?remaining:maximum;final command=['--kill-after=10s','$seconds',...argv];final r=Process.runSync('/usr/bin/timeout',command,environment:env,includeParentEnvironment:false);final stem='$raw/commands/${(counter++).toString().padLeft(5,'0')}-$label';File(stem).parent.createSync(recursive:true);File('$stem.stdout').writeAsStringSync(r.stdout as String);File('$stem.stderr').writeAsStringSync(r.stderr as String);save('$stem.json',{'argv':['/usr/bin/timeout',...command],'exit_code':r.exitCode,'timed_out':r.exitCode==124||r.exitCode==137});if(mustPass)require(r.exitCode==0,'Command failed: $label, retainedlogs');return r;}
int diskBytes(String path){final d=Directory(path);if(!d.existsSync())return 0;var n=0;for(final f in d.listSync(recursive:true,followLinks:false)){if(f is File)n+=f.lengthSync();}return n;}
void budget(int next){final roots=[owned,'$worktree/rust/target/import-memory-attribution-20261012/bin','/home/bsutton/.cache/rv-zipcmp-20261012','/home/bsutton/.cache/rv-protzip-20261012','/home/bsutton/.cache/rv-pagelife-20261012','$worktree/rust/target/public-zip-comparison-20261012/bin','/home/bsutton/.cache/rv-pagebody-20261012','$worktree/rust/target/page-body-borrow-20261012/bin','/home/bsutton/.cache/rv-pageown-20261012','$worktree/rust/target/page-flush-ownership-20261012/bin','/home/bsutton/.cache/rv-addcache8-20261012','$worktree/rust/target/single-file-add-cache8-20261012/bin','/home/bsutton/.cache/rv-rawown-20261012','$worktree/rust/target/raw-frame-ownership-20261012/bin','/home/bsutton/.cache/rv-addcache-20261012','$worktree/rust/target/single-file-add-cache-20261012/bin','/home/bsutton/.cache/rv-win-20261012','$worktree/rust/target/import-window-20261012/bin','/home/bsutton/.cache/rv-enc-20261012','$worktree/rust/target/page-body-lifetime-20261012/bin','/home/bsutton/.cache/rv-add1-20261012','$worktree/rust/target/parallel-encoder-reuse-202612/bin','/home/bsutton/.cache/rv-add-20261012','/home/bsutton/.cache/rv-single-20261012','$worktree/rust/target/explicit-file-extraction-20261012/bin','/home/bsutton/.cache/rv-many-20261012','/home/bsutton/.cache/rv-many-zstd-20261012','/home/bsutton/.cache/revault-native-public-20261011','/home/bsutton/.cache/rv-credential-20261012','$worktree/rust/target/credential-open-20261012/bin','$worktree/rust/target/parallel-extraction-202612/bin','$worktree/rust/target/native-public-20261011/bin','$worktree/rust/target/native-extract-cache-budget-20261011/bin',evidence];final total=roots.map(diskBytes).fold<int>(0,(a,b)=>a+b);require(total+next<=24*1024*1024*1024,'24GiB ownedcap');final r=Process.runSync('/usr/bin/df',['-Pk',owned]);require(r.exitCode==0,'df');final fields=(r.stdout as String).trim().split('\n').last.split(RegExp(r'\s+'));require(int.parse(fields[3])*1024>=next+2*1024*1024*1024,'headroom');}
void agent(int c,{required bool start}){final e=environment(c);final socket='${e['LOCKBOX_SESSION_AGENT_DIR']}/agent.sock';if(start){final r=run([cli,'session','--format','json'],e,'session$c',maximum:20);final rows=(r.stdout as String).split('\n').where((v)=>v.trim().isNotEmpty).map(jsonDecode);require(rows.any((v)=>v is Map&&v['kind']=='agent'&&v['state']=='running'&&v['value']=='yes'),'agent roundtrip');run(['/usr/bin/test','-S',socket],e,'socket$c',maximum:5);}else{final present=run(['/usr/bin/test','-S',socket],e,'cleanup-present$c',maximum:5,mustPass:false,cleanup:true);if(present.exitCode==1)return;require(present.exitCode==0,'Socket inspection failed');run([cli,'session','stop'],e,'stop$c',maximum:20,cleanup:true);run(['/usr/bin/test','!','-S',socket],e,'stopped$c',maximum:5,cleanup:true);}}
void cleanupAgents(List<int> cases){final results=<Map<String,dynamic>>[];for(final c in cases){try{agent(c,start:false);results.add({'case':c,'stopped':true});}catch(error){results.add({'case':c,'stopped':false,'error':'$error'});}}save('$raw/cleanup-${DateTime.now().microsecondsSinceEpoch}.json',{'rows':results,'passed':results.every((r)=>r['stopped']==true)});require(results.every((r)=>r['stopped']==true),'Owned cleanup incomplete');}
List<Map<String,dynamic>> cases()=>(jsonDecode(File('$raw/cases.json').readAsStringSync()) as List).cast<Map<String,dynamic>>();
void identities(){for(final e in (object('$raw/baseline-identities.json')['hashes'] as Map).entries)require(sha(e.key)==e.value,'Changed identity');}
void cold(String archive,Map<String,String> env,String id){for(var a=0;a<3;a++){run(['/usr/bin/sync','-d','--',archive],env,'$id-sync$a',maximum:30);run(['/usr/bin/fadvise','--advice','dontneed',archive],env,'$id-evict$a',maximum:30);final r=run(['/usr/bin/fincore','--json','--bytes','--output','RES,FILE',archive],env,'$id-res$a',maximum:30);final rows=jsonDecode(r.stdout)['fincore'] as List;if(int.parse('${rows.single['res']}')==0)return;}throw StateError('Cold gate failed');}
void verifyOutput(Map<String,dynamic> spec,String output,Map<String,String> env,String id){
 require(FileSystemEntity.typeSync(output,followLinks:false)==FileSystemEntityType.file,'Regular output');final f=File(output);require(f.lengthSync()==spec['bytes']&&(f.statSync().mode&0x1ff)==0x180,'Output size/stored mode0600');run(['/usr/bin/cmp','-s','--',spec['source'] as String,output],env,'$id-compare');run(['/usr/bin/sync','-d','--',output,f.parent.path],env,'$id-output-drain',maximum:60);
}
List<String> addArgs(Map<String,dynamic> spec,String archive,int workflow){final source=spec['source'] as String;return workflow==0?[instrumented,archive,'add',source,'--to','/payload.bin']:[instrumented,archive,'add','--recursive',File(source).parent.path,'--include',source.split('/').last,'--to','/data'];}
void create(String archive,Map<String,String> env,String id){require(!File(archive).existsSync(),'Fresh archive');run([cli,archive,'create','--encryption','none','--signing','none','--compression','zstd','--compression-level','3'],env,'$id-create');}
String verifyArchive(Map<String,dynamic> spec,String archive,int workflow,Map<String,String> env,String id){
 final inspect=run([cli,archive,'doctor'],env,'$id-doctor',maximum:600).stdout as String;
 for(final field in ['encryption: none','signing: none','compression: zstd (level 3)','size padding: default'])require(inspect.contains(field),'Public format mismatch');
 final listing=run([cli,archive,'list','--recursive','--format','json'],env,'$id-list').stdout as String;
 final rows=listing.split('\n').where((v)=>v.trim().isNotEmpty).map(jsonDecode).toList();final files=rows.where((v)=>v['kind']=='file').toList();require(files.length==1&&int.parse('${files.single['len']}')==spec['bytes'],'Sole payload');require(rows.every((v)=>v['kind']=='file'||v['kind']=='directory'),'Unexpected entry kind');final path=files.single['path'] as String;final expected=workflow==0?'/payload.bin':'/data/${(spec['source'] as String).split('/').last}';require(path==expected,'Exact selected public path');return path;
}
void setup(){
 require(!File('$raw/cases.json').existsSync(),'Existing setup');Directory(owned).createSync(recursive:true);
 require(sha(cli)==expectedCli&&sha(instrumented)==expectedInstrumented,'Binary identities');
 final original=(jsonDecode(File('$fixtureEvidence/cases.json').readAsStringSync()) as List).where((v)=>v['bytes']==1073741824).toList();require(original.length==2,'Two pinned sources');
 final specs=<Map<String,dynamic>>[];
 for(var c=0;c<2;c++){
  final v=(original[c]['variants'] as List).singleWhere((v)=>v['variant']=='A');final source=v['source_path'] as String;
  require(sha(source)==v['source_sha256']&&File(source).lengthSync()==1073741824,'Source identity');require(sha('${v['root']}/case.json')==v['case_json_sha256'],'Original case identity');
  final siblings=File(source).parent.listSync(followLinks:false).toList();require(siblings.length==1&&siblings.single.path==source&&siblings.single is File,'Exact one-file source directory');
  final root='$owned/fixture$c';Directory(root).createSync();specs.add({'case':original[c]['name'],'source':source,'source_sha256':v['source_sha256'],'bytes':1073741824,'root':root});
 }
 save('$raw/cases.json',specs);print(green('Diagnostic sources pinned; no setup imports.'));
}
void freeze(){
 require(!File('$raw/baseline-identities.json').existsSync(),'Already frozen');require(sha(cli)==expectedCli&&sha(instrumented)==expectedInstrumented,'Binary identities');
 final hashes=<String,String>{};
 for(final p in [cli,instrumented,Platform.script.toFilePath(),'$evidence/README.md','$raw/cases.json','/usr/bin/time','/usr/bin/taskset','/usr/bin/sync','/usr/bin/fadvise','/usr/bin/fincore','/usr/bin/cmp','/usr/bin/timeout'])hashes[p]=sha(p);
 for(final p in ['lib.rs','import_memory_trace.rs','compression/encode_workspace.rs','file_format/page.rs','lockbox/files.rs','lockbox/file_import_pipeline.rs','lockbox/commit.rs']){final path='$worktree/rust/revault_lockbox_api/src/$p';hashes[path]=sha(path);}
 for(final f in Directory('$raw/build-instrumented').listSync(recursive:true).whereType<File>())hashes[f.path]=sha(f.path);
 for(final spec in cases())hashes[spec['source']]=spec['source_sha256'];
 save('$raw/baseline-identities.json',{'hashes':hashes,'planned_adds':4,'timing_claim':'descriptive instrumented runs only, no comparison','trace_categories':['queued','input','prepared','ordered','writing','scratch','encoder','pooled','object_stream','body_temp','body','stored_body','encode_page'],'maximum_trace_rows':272,'normal_verifier_sha256':expectedCli,'instrumented_sha256':expectedInstrumented});
}
Map<String,dynamic> auditTrace(String stderr,String id){
 final rows=stderr.split('\n').where((v)=>v.startsWith('RV_IMPORT_MEMORY ')).map((v)=>jsonDecode(v.substring('RV_IMPORT_MEMORY '.length)) as Map<String,dynamic>).toList();
 require(rows.isNotEmpty&&rows.length<=272,'Bounded diagnostic rows');
 for(final p in rows){
  require(p['balanced']==true&&p['rss'] is num&&p['rss']>0&&p['hwm'] is num&&p['hwm']>0,'Valid ledger/RSS');
  for(final key in ['bytes','counts','added','removed']){final values=p[key] as List;require(values.length==13&&values.every((v)=>v is int&&v>=0),'Counter vector');}
  final bytes=p['bytes'] as List,counts=p['counts'] as List,added=p['added'] as List,removed=p['removed'] as List;
  require(p['live']==bytes.fold<int>(0,(a,b)=>a+(b as int)),'Live ledger sum');
  for(var i=0;i<13;i++)require(added[i]-removed[i]==bytes[i]&&(counts[i]>0||bytes[i]==0),'Per-category conservation');
 }
 final joined=rows.where((p)=>p['record']=='boundary'&&p['event']=='parallel_joined').toList();require(joined.length==1,'Exactly one joined import');
 require((joined.single['bytes'] as List).every((v)=>v==0)&&(joined.single['counts'] as List).every((v)=>v==0),'Post-join zero bytes and counts');
 require(rows.any((p)=>p['record']=='boundary'&&p['event']=='commit_done'),'Successful commit boundary');
 final peaks=<String,dynamic>{};
 for(final name in ['peak_rss','peak_hwm','peak_live']){final found=rows.where((p)=>p['record']==name).toList();require(found.isNotEmpty,'Required full peak');peaks[name]=found.last;}
 save('$raw/$id-trace.json',rows);
 return {'rows':rows.length,'all_balanced':true,'joined':joined.single,'peaks':peaks,'suppressed_selected_snapshots':rows.map((v)=>v['suppressed'] as int).reduce((a,b)=>a>b?a:b)};
}
void batch(){
 identities();final output=File('$raw/diagnostic-samples.jsonl');require(!output.existsSync(),'Existing batch');final started=<int>[];var accepted=0;
 try{final specs=cases();for(var c=0;c<2;c++){
  started.add(c);agent(c,start:true);final spec=specs[c];final env=environment(c);
  for(final workflow in [0,1]){
   final id='c$c-w$workflow';final archive='${spec['root']}/$id.lbox',destination='${spec['root']}/$id.out';require(!File(destination).existsSync(),'Fresh output');budget(3*(spec['bytes'] as int)+256*1024*1024);
   create(archive,env,id);cold(spec['source'] as String,env,id);final timing='$raw/$id.time';
   final result=run(['/usr/bin/time','-f','%e %U %S %M','-o',timing,'/usr/bin/taskset','-c','0-4',...addArgs(spec,archive,workflow)],{...env,'REVAULT_IMPORT_MEMORY_TRACE':'1'},id,maximum:600);
   final metrics=File(timing).readAsStringSync().trim().split(RegExp(r'\s+')).map(double.parse).toList();require(metrics.length==4&&metrics.every((v)=>v.isFinite&&v>=0)&&metrics[0]>0&&metrics[3]>0,'Resource metadata');
   final trace=auditTrace(result.stderr as String,id);
   run(['/usr/bin/sync','-d','--',archive,File(archive).parent.path],env,'$id-drain',maximum:60);budget(spec['bytes'] as int);
   final path=verifyArchive(spec,archive,workflow,env,id);run([cli,archive,'extract',path,destination],env,'$id-extract',maximum:600);verifyOutput(spec,destination,env,id);
   final row={'case':spec['case'],'workflow':workflow==0?'single-file-interactive':'one-file-directory-bulkimport','instrumented_elapsed_seconds':metrics[0],'whole_process_peak_rss_bytes':metrics[3]*1024,'archive_bytes':File(archive).lengthSync(),'source_sha256':spec['source_sha256'],'archive_sha256':sha(archive),'persisted_bytes_mode_verified':true,'cold_source':true,'trace':trace};
   output.writeAsStringSync('${jsonEncode(row)}\n',mode:FileMode.append,flush:true);accepted++;File(destination).deleteSync();File(archive).deleteSync();
  }
 }}finally{cleanupAgents(started);}
 identities();require(accepted==4,'Exactly four diagnostic imports');save('$raw/complete.json',{'accepted':4,'identities_verified_after':true,'owned_agents_stopped':true,'all_ledger_gates_passed':true,'wall_seconds':wall.elapsedMilliseconds/1000});print(green('Four diagnostic imports independently verified; no performance qualification claim.'));
}
void main(List<String> args){require(Platform.isLinux&&args.length==1,'Linux setup|freeze|batch');final mask=DynamicLibrary.process().lookupFunction<Uint32 Function(Uint32),int Function(int)>('umask');final previous=mask(0x12);try{Directory(raw).createSync(recursive:true);counter=Directory('$raw/commands').existsSync()?Directory('$raw/commands').listSync().whereType<File>().length:0;switch(args.single){case 'setup':setup();break;case 'freeze':freeze();break;case 'batch':batch();break;default:throw ArgumentError('phase');}}finally{mask(previous);}}
