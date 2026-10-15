#! /usr/bin/env dcli
import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:math';
import 'package:dcli/dcli.dart';
const worktree='/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance';
final draftRoot=File(Platform.script.toFilePath()).parent.path;
final inputs=object('$draftRoot/build-inputs.json');
final evidence=inputs['evidence_root'] as String;
const fixtureEvidence='$worktree/docs/evidence/native-public-extraction-2026-10-11/raw';
final raw='$evidence/raw';
final owned=inputs['owned_root'] as String;
String currentVariant='new';
String get cli=>inputs['variants'][currentVariant] as String;
String get expectedCli=>inputs['executable_hashes'][cli] as String;
final vaultPasswords=<int,String>{};
final wall=Stopwatch()..start();int counter=0;
void require(bool b,String m){if(!b)throw StateError(m);}
Map<String,dynamic> object(String p)=>jsonDecode(File(p).readAsStringSync()) as Map<String,dynamic>;
void save(String p,Object v){File(p).parent.createSync(recursive:true);File(p).writeAsStringSync('${const JsonEncoder.withIndent('  ').convert(v)}\n',flush:true);}
String sha(String p){final r=Process.runSync('/usr/bin/sha256sum',['--',p]);require(r.exitCode==0,'hash');return (r.stdout as String).split(' ').first;}
Map<String,String> environment(int c){final p='$owned/c$c';final e=<String,String>{'PATH':'/usr/bin:/bin','TMPDIR':'/home/bsutton/.cache/rv4-security-20261011','HOME':'$p/home','XDG_CONFIG_HOME':'$p/config','XDG_CACHE_HOME':'$p/cache','XDG_STATE_HOME':'$p/state','XDG_DATA_HOME':'$p/data','LOCKBOX_SESSION_AGENT_DIR':'$p/agent','LOCKBOX_SESSION_AGENT_LOG':'$p/agent.log','LOCKBOX_VAULT_DIR':'$p/vault','LOCKBOX_PLATFORM_SECRET_STORE':'disabled','LOCKBOX_ADD_PROGRESS':'off','LOCKBOX_VAULT_PASSWORD':vaultPasswords.putIfAbsent(c,()=>base64UrlEncode(List<int>.generate(32,(_)=>Random.secure().nextInt(256))))};for(final k in ['HOME','XDG_CONFIG_HOME','XDG_CACHE_HOME','XDG_STATE_HOME','XDG_DATA_HOME','LOCKBOX_SESSION_AGENT_DIR','LOCKBOX_VAULT_DIR'])Directory(e[k]!).createSync(recursive:true);final permissions=Process.runSync('/usr/bin/chmod',['0700',p]);require(permissions.exitCode==0,'Private owned context');require(utf8.encode('${e['LOCKBOX_SESSION_AGENT_DIR']}/agent.sock').length<=100,'socket');return e;}
ProcessResult run(List<String> argv,Map<String,String> env,String label,{int maximum=180,bool mustPass=true,bool cleanup=false}){final remaining=cleanup?maximum:1800-wall.elapsed.inSeconds-60;require(remaining>0,'30minute cap');final seconds=remaining<maximum?remaining:maximum;final command=['--kill-after=10s','$seconds',...argv];final r=Process.runSync('/usr/bin/timeout',command,environment:env,includeParentEnvironment:false);final stem='$raw/commands/${(counter++).toString().padLeft(5,'0')}-$label';File(stem).parent.createSync(recursive:true);final protectedOutput=argv.contains(cli);File('$stem.stdout').writeAsStringSync(protectedOutput?'Protected CLI output suppressed; parsed internally where required.\n':r.stdout as String);File('$stem.stderr').writeAsStringSync(protectedOutput?'Protected CLI diagnostics suppressed.\n':r.stderr as String);save('$stem.json',{'argv':['/usr/bin/timeout',...command],'exit_code':r.exitCode,'timed_out':r.exitCode==124||r.exitCode==137});if(mustPass)require(r.exitCode==0,'Command failed: $label, retainedlogs');return r;}
int diskBytes(String path){final d=Directory(path);if(!d.existsSync())return 0;var n=0;for(final f in d.listSync(recursive:true,followLinks:false)){if(f is File)n+=f.lengthSync();}return n;}
void budget(int next){final roots=<String>{owned,File(cli).parent.path,...(inputs['additional_owned_roots'] as List).cast<String>(),'/home/bsutton/.cache/rv-zipcmp-20261012','/home/bsutton/.cache/rv-pagelife-20261012','$worktree/rust/target/public-zip-comparison-20261012/bin','/home/bsutton/.cache/rv-pagebody-20261012','$worktree/rust/target/page-body-borrow-20261012/bin','/home/bsutton/.cache/rv-pageown-20261012','$worktree/rust/target/page-flush-ownership-20261012/bin','/home/bsutton/.cache/rv-addcache8-20261012','$worktree/rust/target/single-file-add-cache8-20261012/bin','/home/bsutton/.cache/rv-rawown-20261012','$worktree/rust/target/raw-frame-ownership-20261012/bin','/home/bsutton/.cache/rv-addcache-20261012','$worktree/rust/target/single-file-add-cache-20261012/bin','/home/bsutton/.cache/rv-win-20261012','$worktree/rust/target/import-window-20261012/bin','/home/bsutton/.cache/rv-enc-20261012','$worktree/rust/target/page-body-lifetime-20261012/bin','/home/bsutton/.cache/rv-add1-20261012','$worktree/rust/target/parallel-encoder-reuse-202612/bin','/home/bsutton/.cache/rv-add-20261012','/home/bsutton/.cache/rv-single-20261012','$worktree/rust/target/explicit-file-extraction-20261012/bin','/home/bsutton/.cache/rv-many-20261012','/home/bsutton/.cache/rv-many-zstd-20261012','/home/bsutton/.cache/revault-native-public-20261011','/home/bsutton/.cache/rv-credential-20261012','$worktree/rust/target/credential-open-20261012/bin','$worktree/rust/target/parallel-extraction-202612/bin','$worktree/rust/target/native-public-20261011/bin','$worktree/rust/target/native-extract-cache-budget-20261011/bin',evidence};final total=roots.map(diskBytes).fold<int>(0,(a,b)=>a+b);require(total+next<=24*1024*1024*1024,'24GiB ownedcap');final r=Process.runSync('/usr/bin/df',['-Pk',owned]);require(r.exitCode==0,'df');final fields=(r.stdout as String).trim().split('\n').last.split(RegExp(r'\s+'));require(int.parse(fields[3])*1024>=next+2*1024*1024*1024,'headroom');}
void agent(int c,{required bool start}){final e=environment(c);final socket='${e['LOCKBOX_SESSION_AGENT_DIR']}/agent.sock';if(start){final r=run([cli,'session','--format','json'],e,'session$c',maximum:20);final rows=(r.stdout as String).split('\n').where((v)=>v.trim().isNotEmpty).map(jsonDecode);require(rows.any((v)=>v is Map&&v['kind']=='agent'&&v['state']=='running'&&v['value']=='yes'),'agent roundtrip');run(['/usr/bin/test','-S',socket],e,'socket$c',maximum:5);}else{final present=run(['/usr/bin/test','-S',socket],e,'cleanup-present$c',maximum:5,mustPass:false,cleanup:true);if(present.exitCode==1)return;require(present.exitCode==0,'Socket inspection failed');run([cli,'session','stop'],e,'stop$c',maximum:20,cleanup:true);run(['/usr/bin/test','!','-S',socket],e,'stopped$c',maximum:5,cleanup:true);}}
void cleanupAgents(List<int> cases){final results=<Map<String,dynamic>>[];for(final c in cases){try{agent(c,start:false);final context=Directory('$owned/c$c');if(context.existsSync())context.deleteSync(recursive:true);vaultPasswords.remove(c);results.add({'case':c,'stopped':true,'private_context_removed':true});}catch(error){results.add({'case':c,'stopped':false,'error':'$error'});}}save('$raw/cleanup-${DateTime.now().microsecondsSinceEpoch}.json',{'rows':results,'passed':results.every((r)=>r['stopped']==true)});require(results.every((r)=>r['stopped']==true),'Owned cleanup incomplete');}
void initVault(int c){
 final env=environment(c);require(Directory(env['LOCKBOX_VAULT_DIR']!).listSync().isEmpty,'Fresh synthetic Vault only');
 run([cli,'vault','init'],env,'vault-init$c',maximum:180);
 save('$raw/vault-setup-$c.json',{'public_init_exit':0,'synthetic_default_profile':true,'raw_output_discarded':true,'password_transport':'in-memory child environment','outside_timing':true});
}
List<Map<String,dynamic>> cases()=>(jsonDecode(File('$raw/cases.json').readAsStringSync()) as List).cast<Map<String,dynamic>>();
void identities(){verifyCurrentBuild();for(final e in (object('$raw/paired-identities.json')['hashes'] as Map).entries)require(sha(e.key)==e.value,'Changed identity');for(final s in cases())require(FileSystemEntity.typeSync(s['source'],followLinks:false)==FileSystemEntityType.file&&(File(s['source']).statSync().mode&0x1ff)==s['source_mode'],'Source mode identity');}
void cold(String archive,Map<String,String> env,String id){for(var a=0;a<3;a++){run(['/usr/bin/sync','-d','--',archive],env,'$id-sync$a',maximum:30);run(['/usr/bin/fadvise','--advice','dontneed',archive],env,'$id-evict$a',maximum:30);final r=run(['/usr/bin/fincore','--json','--bytes','--output','RES,FILE',archive],env,'$id-res$a',maximum:30);final rows=jsonDecode(r.stdout)['fincore'] as List;if(int.parse('${rows.single['res']}')==0)return;}throw StateError('Cold gate failed');}
void verifyOutput(Map<String,dynamic> spec,String output,Map<String,String> env,String id){
 require(FileSystemEntity.typeSync(output,followLinks:false)==FileSystemEntityType.file,'Regular output');final f=File(output);require(f.lengthSync()==spec['bytes']&&(f.statSync().mode&0x1ff)==spec['expected_mode'],'Output size/declared mode');run(['/usr/bin/cmp','-s','--',spec['source'] as String,output],env,'$id-compare');run(['/usr/bin/sync','-d','--',output,f.parent.path],env,'$id-output-drain',maximum:60);
}
List<double> timed(List<String> argv,Map<String,String> env,String id){
 final path='$raw/$id.time';
 run(['/usr/bin/time','-f','%e %U %S %M','-o',path,'/usr/bin/taskset','-c','0-4',...argv],env,id,maximum:600);
 final m=File(path).readAsStringSync().trim().split(RegExp(r'\s+')).map(double.parse).toList();
 require(m.length==4&&m.every((v)=>v.isFinite&&v>=0)&&m[3]>0,'Valid measured resources');return m;
}
List<double> sumCommands(List<List<double>> components)=>[
 for(var i=0;i<3;i++)components.fold<double>(0,(s,c)=>s+c[i]),
 components.map((c)=>c[3]).reduce((a,b)=>a>b?a:b)
];
List<String> createArgs(String archive)=>[cli,archive,'create','--encryption','chacha20-poly1305','--signing','owner','--compression','zstd','--compression-level','3'];
String unusedInspectArchive(Map<String,dynamic> s,String archive,Map<String,String> env,String id){
 final doctor=run([cli,archive,'doctor'],env,'$id-doctor',maximum:600).stdout as String;
 for(final value in ['encryption: chacha20-poly1305','signing: owner','compression: zstd (level 3)','size padding: default'])require(doctor.contains(value),'Native options');
 final listing=run([cli,archive,'list','--recursive','--format','json'],env,'$id-list').stdout as String;
 final entries=listing.split('\n').where((s)=>s.trim().isNotEmpty).map(jsonDecode).toList();
 require(entries.length==1&&entries.single['kind']=='file'&&entries.single['path']=='/${s['basename']}'&&int.parse('${entries.single['len']}')==s['bytes'],'Exact native entry');
 return 'ChaCha20-Poly1305 + owner signing; Zstd3 configured; per-frame raw fallback permitted';
}
void verifyCurrentBuild(){
 require(inputs['execution_authorized']==true,'Preparation draft: execution requires completed integration and current build provenance');
 require(inputs['worktree']==worktree,'Unexpected worktree');
 require(evidence.startsWith('$worktree/docs/evidence/extraction-last-use-diagnostic-')&&owned.startsWith('/home/bsutton/.cache/rv-lud-'),'Fresh owned namespaces required');
 final head=Process.runSync('/usr/bin/git',['-C',worktree,'rev-parse','HEAD']);
 require(head.exitCode==0&&(head.stdout as String).trim()==inputs['head'],'Current HEAD differs');
 require(sha(cli)==expectedCli,'Current executable hash');
 final sourceHashes=(inputs['runtime_source_hashes'] as Map).cast<String,String>();
 require(sourceHashes.length>700,'Full current runtime manifest required');
 for(final entry in sourceHashes.entries){require(entry.key.startsWith('$worktree/rust/')&&sha(entry.key)==entry.value,'Current runtime identity changed');}
 require(inputs['build_source_manifest_verified']==true,'Retained runtime correspondence must be documented');
 for(final entry in (inputs['build_provenance_hashes'] as Map).entries){require(sha(entry.key)==entry.value,'Build provenance changed');}
}
Map<String,int> verifyMany(Map<String,dynamic> spec,String output,Map<String,String> env,String id,{bool observe=false}){
 final files=(spec['files'] as List).cast<Map<String,dynamic>>();
 require(Directory(output).listSync(followLinks:false).length==1,'Exact root');
 require(FileSystemEntity.typeSync('$output/data',followLinks:false)==FileSystemEntityType.directory,'Stored directory');
 final actual=Directory('$output/data').listSync(recursive:true,followLinks:false);require(actual.length==files.length,'Exact count');
 final expected={for(final f in files)'$output/data/${f['name']}':f};final modes=<String,int>{'.':Directory(output).statSync().mode&0x1ff,'data':Directory('$output/data').statSync().mode&0x1ff};
 for(final f in actual){final entry=expected.remove(f.path);require(entry!=null&&f is File,'Exact names/types');require(File(f.path).lengthSync()==entry!['bytes'],'Length');modes['data/${entry['name']}']=f.statSync().mode&0x1ff;}
 require(expected.isEmpty,'No missing paths');
 if(!observe){final pinned=(spec['output_modes'] as Map).cast<String,int>();require(modes.length==pinned.length&&modes.entries.every((e)=>pinned[e.key]==e.value),'Persisted permission expectations');}
 run(['/usr/bin/diff','--brief','--recursive','--',spec['source_directory'] as String,'$output/data'],env,'$id-compare',maximum:180);
 run(['/usr/bin/sync','-d','--',...actual.map((f)=>f.path),'$output/data',output],env,'$id-drain',maximum:60);
 return modes;
}
void setup(){
 verifyCurrentBuild();require(!File('$raw/cases.json').existsSync(),'Existing setup');Directory(owned).createSync(recursive:true);
 const prior='$worktree/docs/evidence/many-file-current-2026-10-12/raw';final complete=object('$prior/complete.json');require(complete['accepted']==16&&complete['identities_verified_after']==true,'Verified current baseline');
 final spec=File('$prior/cases.json');require(sha(spec.path)==inputs['verified_cases_sha256'],'Pinned setup cases');spec.copySync('$raw/cases.json');
 require(cases().length==4,'Four cohorts');print(green('Reused independently verified immutable fixtures/modes; no setup extraction repeats.'));
}
void freeze(){
 verifyCurrentBuild();require(!File('$raw/paired-identities.json').existsSync(),'Existing freeze');final hashes=<String,String>{};
 for(final p in [...(inputs['variants'] as Map).values.cast<String>(),'$worktree/tools/summarize-adaptive-workers.dart','/usr/bin/strace','/usr/bin/time','/usr/bin/taskset','/usr/bin/sync','/usr/bin/timeout','/usr/bin/fadvise','/usr/bin/fincore','/usr/bin/cmp','/usr/bin/diff',Platform.script.toFilePath(),'$draftRoot/PROTOCOL.txt','$draftRoot/build-inputs.json','$raw/cases.json'])hashes[p]=sha(p);
 for(final e in (inputs['runtime_source_hashes'] as Map).entries)hashes[e.key]=e.value;
 for(final e in (inputs['build_provenance_hashes'] as Map).entries)hashes[e.key]=e.value;
 for(final s in cases()){
  hashes[s['archive']]=s['archive_sha256'];hashes[s['source']]=s['source_sha256'];hashes[s['prior_manifest']]=s['prior_manifest_sha256'];
  for(final f in s['files'])hashes['${s['source_directory']}/${f['name']}']=f['sha256'];
 }
 for(final e in hashes.entries)require(sha(e.key)==e.value,'Freeze identity');
 save('$raw/paired-identities.json',{'hashes':hashes,'variants':inputs['variants'],'planned':8,'scope':'four untraced resource observations plus four strace summaries; no timing inference','affinity':'0-4','umask':'0022','protection':'existing plaintext unsigned archives','read_profile':'ExtractMany; >=256 files eligible for up to4 worker handles; observed count not instrumented','permission_scope':'current default compared to independent explicit restoration setup','rss_scope':'Untraced GNUtime covers CLI; traced GNUtime includes tracer and descendant accounting. Agent snapshots separate, no aggregate bound computed. All agent task children checked at boundaries, not transient census.'});
}
void batch(){
 identities();final log=File('$raw/diagnostic-samples.jsonl');require(!log.existsSync()&&!File('$raw/batch-started.json').existsSync(),'Existing batch');save('$raw/batch-started.json',{'planned':8,'started_utc':DateTime.now().toUtc().toIso8601String()});final started=<int>[];var accepted=0;
 try{final specs=cases();for(final c in [1,3]){final spec=specs[c];for(var mode=0;mode<2;mode++){for(var position=0;position<2;position++){
 currentVariant=(mode+position).isEven?'old':'new';final context=accepted;started.add(context);agent(context,start:true);final env=environment(context);final owner=findAgent(context);agentMemory(owner,'setup');
 final id='c$c-m$mode-p$position-$currentVariant';final output='$owned/output-$id';require(!Directory(output).existsSync(),'Fresh output');budget((spec['bytes'] as int)+256*1024*1024);cold(spec['archive'],env,id);
 final command=<String>['/usr/bin/time','-f','%e %U %S %M %R %F %w %c %I %O','-o','$raw/$id.time','/usr/bin/taskset','-c','0-4'];
 if(mode==1)command.addAll(['/usr/bin/strace','-f','-c','-e','trace=mmap,munmap,mremap,madvise,brk,read,pread64,write,pwrite64,futex','-o','$raw/$id.strace-summary']);
 command.addAll([cli,spec['archive'],'extract','--to',output]);run(command,env,id,maximum:180);
 final values=File('$raw/$id.time').readAsStringSync().trim().split(RegExp(r'\s+')).map(double.parse).toList();require(values.length==10&&values.every((v)=>v.isFinite&&v>=0)&&values[3]>0,'Resource counters');
 if(mode==1)require(File('$raw/$id.strace-summary').readAsStringSync().contains('total'),'Trace summary');
 final after=agentMemory(owner,'extract');verifyMany(spec,output,env,id);Directory(output).deleteSync(recursive:true);
 log.writeAsStringSync('${jsonEncode({'case':spec['case'],'variant':currentVariant,'mode':mode==0?'untraced-counters':'strace-summary','values_order':['elapsed_seconds','user_seconds','system_seconds','max_rss_kib','minor_faults','major_faults','voluntary_context_switches','involuntary_context_switches','filesystem_inputs','filesystem_outputs'],'values':values,'agent_after':after,'exact_tree_bytes_modes_verified':true,'cold_archive':true,'output_drained':true,'executable':cli})}\n',mode:FileMode.append,flush:true);
 accepted++;cleanupAgents([context]);started.remove(context);
 }}}}finally{cleanupAgents(started);}
 identities();require(accepted==8,'Exact8');save('$raw/diagnostic-complete.json',{'accepted':accepted,'identities_verified_after':true,'owned_agents_stopped':true,'private_contexts_removed':true,'wall_seconds':wall.elapsedMilliseconds/1000});print(green('Eight diagnostic extractions verified; no retention decision.'));
}

Map<String,dynamic> processIdentity(int pid){
 final stat=File('/proc/$pid/stat').readAsStringSync();final fields=stat.substring(stat.lastIndexOf(')')+2).split(' ');
 return {'pid':pid,'start_ticks':int.parse(fields[19])};
}
Map<String,dynamic> findAgent(int context){
 final socket='${environment(context)['LOCKBOX_SESSION_AGENT_DIR']}/agent.sock';
 final entries=File('/proc/net/unix').readAsLinesSync().skip(1).map((l)=>l.trim().split(RegExp(r'\s+'))).where((f)=>f.length==8&&f[7]==socket).toList();
 require(entries.length==1,'Exactly one owned socket');final inode=entries.single[6];final owners=<int>[];
 for(final p in Directory('/proc').listSync(followLinks:false)){
  final pid=int.tryParse(p.path.split('/').last);if(pid==null)continue;
  try{for(final fd in Directory('${p.path}/fd').listSync(followLinks:false)){if(fd is Link&&fd.targetSync()=='socket:[$inode]'){owners.add(pid);break;}}}on FileSystemException{continue;}
 }
 require(owners.length==1,'Exactly one owned agent process');final pid=owners.single;
 require(Link('/proc/$pid/exe').resolveSymbolicLinksSync()==File(cli).resolveSymbolicLinksSync(),'Agent executable path');
 require(sha('/proc/$pid/exe')==expectedCli,'Agent executable bytes');
 return processIdentity(pid);
}
Map<String,dynamic> agentMemory(Map<String,dynamic> owner,String phase){
 final pid=owner['pid'] as int;require(processIdentity(pid)['start_ticks']==owner['start_ticks'],'Agent PID generation');
 final lines=File('/proc/$pid/status').readAsLinesSync();
 int value(String key){final text=lines.singleWhere((l)=>l.startsWith('$key:')).split(':').last.trim().split(RegExp(r'\s+'));require(text.length==2&&text[1]=='kB','Numeric status unit');return int.parse(text[0])*1024;}
 final children=<int>{};for(final task in Directory('/proc/$pid/task').listSync(followLinks:false)){try{children.addAll(File('${task.path}/children').readAsStringSync().trim().split(RegExp(r'\s+')).where((s)=>s.isNotEmpty).map(int.parse));}on FileSystemException{require(!Directory(task.path).existsSync(),'Task children unreadable');}}
 require(children.isEmpty,'Unexpected persistent agent child');
 final rss=value('VmRSS'),hwm=value('VmHWM');require(hwm>=rss&&rss>0,'Agent positive resources');
 require(processIdentity(pid)['start_ticks']==owner['start_ticks'],'Stable agent snapshot');
 return {...owner,'phase':phase,'observed_utc':DateTime.now().toUtc().toIso8601String(),'rss_bytes':rss,'hwm_bytes':hwm,'children':(children.toList()..sort())};
}

void main(List<String> args){require(Platform.isLinux&&args.length==1,'Linux setup|freeze|batch');final mask=DynamicLibrary.process().lookupFunction<Uint32 Function(Uint32),int Function(int)>('umask');final previous=mask(0x12);try{verifyCurrentBuild();Directory(raw).createSync(recursive:true);counter=Directory('$raw/commands').existsSync()?Directory('$raw/commands').listSync().whereType<File>().length:0;switch(args.single){case 'setup':setup();break;case 'freeze':freeze();break;case 'batch':batch();break;default:throw ArgumentError('phase');}}finally{mask(previous);}}
