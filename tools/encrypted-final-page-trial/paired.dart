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
String currentVariant='old';
String get cli=>(inputs['variants'] as Map)[currentVariant] as String;
String get expectedCli=>(inputs['executable_hashes'] as Map)[cli] as String;
final vaultPasswords=<int,String>{};
final wall=Stopwatch()..start();int counter=0;
void require(bool b,String m){if(!b)throw StateError(m);}
Map<String,dynamic> object(String p)=>jsonDecode(File(p).readAsStringSync()) as Map<String,dynamic>;
void save(String p,Object v){File(p).parent.createSync(recursive:true);File(p).writeAsStringSync('${const JsonEncoder.withIndent('  ').convert(v)}\n',flush:true);}
String sha(String p){final r=Process.runSync('/usr/bin/sha256sum',['--',p]);require(r.exitCode==0,'hash');return (r.stdout as String).split(' ').first;}
Map<String,String> environment(int c){final p='$owned/c$c';final e=<String,String>{'PATH':'/usr/bin:/bin','TMPDIR':'/home/bsutton/.cache/rv4-security-20261011','HOME':'$p/home','XDG_CONFIG_HOME':'$p/config','XDG_CACHE_HOME':'$p/cache','XDG_STATE_HOME':'$p/state','XDG_DATA_HOME':'$p/data','LOCKBOX_SESSION_AGENT_DIR':'$p/agent','LOCKBOX_SESSION_AGENT_LOG':'$p/agent.log','LOCKBOX_VAULT_DIR':'$p/vault','LOCKBOX_PLATFORM_SECRET_STORE':'disabled','LOCKBOX_ADD_PROGRESS':'off','LOCKBOX_VAULT_PASSWORD':vaultPasswords.putIfAbsent(c,()=>base64UrlEncode(List<int>.generate(32,(_)=>Random.secure().nextInt(256))))};for(final k in ['HOME','XDG_CONFIG_HOME','XDG_CACHE_HOME','XDG_STATE_HOME','XDG_DATA_HOME','LOCKBOX_SESSION_AGENT_DIR','LOCKBOX_VAULT_DIR'])Directory(e[k]!).createSync(recursive:true);final permissions=Process.runSync('/usr/bin/chmod',['0700',p]);require(permissions.exitCode==0,'Private owned context');require(utf8.encode('${e['LOCKBOX_SESSION_AGENT_DIR']}/agent.sock').length<=100,'socket');return e;}
ProcessResult run(List<String> argv,Map<String,String> env,String label,{int maximum=180,bool mustPass=true,bool cleanup=false}){final remaining=cleanup?maximum:5400-wall.elapsed.inSeconds-60;require(remaining>0,'90minute cap');final seconds=remaining<maximum?remaining:maximum;final command=['--kill-after=10s','$seconds',...argv];final r=Process.runSync('/usr/bin/timeout',command,environment:env,includeParentEnvironment:false);final stem='$raw/commands/${(counter++).toString().padLeft(5,'0')}-$label';File(stem).parent.createSync(recursive:true);final protectedOutput=argv.contains(cli);File('$stem.stdout').writeAsStringSync(protectedOutput?'Protected CLI output suppressed; parsed internally where required.\n':r.stdout as String);File('$stem.stderr').writeAsStringSync(protectedOutput?'Protected CLI diagnostics suppressed.\n':r.stderr as String);save('$stem.json',{'argv':['/usr/bin/timeout',...command],'exit_code':r.exitCode,'timed_out':r.exitCode==124||r.exitCode==137});if(mustPass)require(r.exitCode==0,'Command failed: $label, retainedlogs');return r;}
int diskBytes(String path){final d=Directory(path);if(!d.existsSync())return 0;var n=0;for(final f in d.listSync(recursive:true,followLinks:false)){if(f is File)n+=f.lengthSync();}return n;}
void budget(int next){final roots=[owned,...(inputs['variants'] as Map).values.cast<String>().map((p)=>File(p).parent.path),...(inputs['additional_owned_roots'] as List).cast<String>(),'/home/bsutton/.cache/rv-zipcmp-20261012','/home/bsutton/.cache/rv-pagelife-20261012','$worktree/rust/target/public-zip-comparison-20261012/bin','/home/bsutton/.cache/rv-pagebody-20261012','$worktree/rust/target/page-body-borrow-20261012/bin','/home/bsutton/.cache/rv-pageown-20261012','$worktree/rust/target/page-flush-ownership-20261012/bin','/home/bsutton/.cache/rv-addcache8-20261012','$worktree/rust/target/single-file-add-cache8-20261012/bin','/home/bsutton/.cache/rv-rawown-20261012','$worktree/rust/target/raw-frame-ownership-20261012/bin','/home/bsutton/.cache/rv-addcache-20261012','$worktree/rust/target/single-file-add-cache-20261012/bin','/home/bsutton/.cache/rv-win-20261012','$worktree/rust/target/import-window-20261012/bin','/home/bsutton/.cache/rv-enc-20261012','$worktree/rust/target/page-body-lifetime-20261012/bin','/home/bsutton/.cache/rv-add1-20261012','$worktree/rust/target/parallel-encoder-reuse-202612/bin','/home/bsutton/.cache/rv-add-20261012','/home/bsutton/.cache/rv-single-20261012','$worktree/rust/target/explicit-file-extraction-20261012/bin','/home/bsutton/.cache/rv-many-20261012','/home/bsutton/.cache/rv-many-zstd-20261012','/home/bsutton/.cache/revault-native-public-20261011','/home/bsutton/.cache/rv-credential-20261012','$worktree/rust/target/credential-open-20261012/bin','$worktree/rust/target/parallel-extraction-202612/bin','$worktree/rust/target/native-public-20261011/bin','$worktree/rust/target/native-extract-cache-budget-20261011/bin',evidence];final total=roots.toSet().map(diskBytes).fold<int>(0,(a,b)=>a+b);require(total+next<=24*1024*1024*1024,'24GiB ownedcap');final r=Process.runSync('/usr/bin/df',['-Pk',owned]);require(r.exitCode==0,'df');final fields=(r.stdout as String).trim().split('\n').last.split(RegExp(r'\s+'));require(int.parse(fields[3])*1024>=next+2*1024*1024*1024,'headroom');}
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
String inspectArchive(Map<String,dynamic> s,String archive,Map<String,String> env,String id){
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
 require(evidence.startsWith('$worktree/docs/evidence/encrypted-final-page-')&&owned.startsWith('/home/bsutton/.cache/rv-ep-'),'Fresh owned namespaces required');
 final head=Process.runSync('/usr/bin/git',['-C',worktree,'rev-parse','HEAD']);
 require(head.exitCode==0&&(head.stdout as String).trim()==inputs['head'],'Current HEAD differs');
 require(sha(cli)==expectedCli,'Current executable hash');
 for(final entry in (inputs['executable_hashes'] as Map).entries){require(sha(entry.key)==entry.value,'Preserved executable changed');}
 final sourceHashes=(inputs['runtime_source_hashes'] as Map).cast<String,String>();
 require(sourceHashes.length>700,'Full current runtime manifest required');
 for(final entry in sourceHashes.entries){require(entry.key.startsWith('$worktree/rust/')&&sha(entry.key)==entry.value,'Current runtime identity changed');}
 require(inputs['build_source_manifest_verified']==true,'Build must attest same current runtime manifest');
 require((inputs['build_provenance_hashes'] as Map).isNotEmpty,'Nonempty build provenance required');
 for(final entry in (inputs['build_provenance_hashes'] as Map).entries){require(sha(entry.key)==entry.value,'Build provenance changed');}
}
void verifyTree(Map<String,dynamic> s,String destination,Map<String,String> env,String id){
 require(FileSystemEntity.typeSync(destination,followLinks:false)==FileSystemEntityType.directory,'Output directory');
 final entries=Directory(destination).listSync(recursive:true,followLinks:false).toList();
 final file='$destination/${s['basename']}';
 require(entries.length==1&&entries.single.path==file&&entries.single is File,'Exact output tree');
 verifyOutput(s,file,env,id);
}
void setup(){
 require(!File('$raw/cases.json').existsSync(),'Existing setup');Directory(owned).createSync(recursive:true);final privateRoot=Process.runSync('/usr/bin/chmod',['0700',owned]);require(privateRoot.exitCode==0,'Private owned root');
 verifyCurrentBuild();
 final previous=(jsonDecode(File('$fixtureEvidence/cases.json').readAsStringSync()) as List).where((s)=>s['bytes']==1073741824).toList();require(previous.length==2,'Two sources');
 final rows=<Map<String,dynamic>>[];
 for(var c=0;c<2;c++){
  final original=previous[c];final v=(original['variants'] as List).singleWhere((v)=>v['variant']=='A');final path=v['source_path'] as String;
  require(sha(path)==v['source_sha256']&&File(path).lengthSync()==original['bytes'],'Source identity');
  require(FileSystemEntity.typeSync(path,followLinks:false)==FileSystemEntityType.file&&(File(path).statSync().mode&0x1ff)==0x1b4,'Source regular0664');
  require(sha('${v['root']}/case.json')==v['case_json_sha256'],'Original case identity');
  final basename=path.split('/').last;require(!basename.startsWith('-')&&!basename.contains('\n'),'Safe filename');
  final root='$owned/case$c';Directory(root).createSync();final privateCase=Process.runSync('/usr/bin/chmod',['0700',root]);require(privateCase.exitCode==0,'Private case output root');
  rows.add({'case':original['name'],'source':path,'source_sha256':v['source_sha256'],'bytes':original['bytes'],'basename':basename,'source_mode':0x1b4,'root':root});
 }
 save('$raw/cases.json',rows);print(green('Pinned two sources; no setup imports.'));
}
void freeze(){
 verifyCurrentBuild();require(!File('$raw/paired-identities.json').existsSync(),'Already frozen');
 final hashes=<String,String>{};
 for(final path in [...(inputs['variants'] as Map).values.cast<String>(),'$worktree/tools/summarize-adaptive-workers.dart','/usr/bin/time','/usr/bin/taskset','/usr/bin/sync','/usr/bin/timeout','/usr/bin/fadvise','/usr/bin/fincore','/usr/bin/cmp',Platform.script.toFilePath(),'$draftRoot/analyze.dart','$draftRoot/PROTOCOL.txt','$draftRoot/build-inputs.json','$raw/cases.json','$fixtureEvidence/cases.json'])hashes[path]=sha(path);
 for(final entry in (inputs['runtime_source_hashes'] as Map).entries){hashes[entry.key]=entry.value;}
 for(final entry in (inputs['build_provenance_hashes'] as Map).entries){hashes[entry.key]=entry.value;}
 for(final s in cases()){require(sha(s['source'])==s['source_sha256'],'Pinned source');hashes[s['source']]=s['source_sha256'];}
 save('$raw/paired-identities.json',{'hashes':hashes,'variants':inputs['variants'],'planned_writes':56,'preparation':8,'measured':48,'umask':'0022','affinity':'0-4','write_workers':'Auto','credential_scope':'synthetic Vault initialization and agent startup excluded; create+explicit open+add include per-command Vault/KDF costs; agent remains running; extraction verification excluded','rss_scope':'maximum GNU time peak across sequential create/open/add CLI subprocesses; helper agent excluded; not whole process tree','permissions':'host sources remain0664; native stores/restores0600','drain_scope':'sync -d separately reported, not equivalent crash durability','available_processors':Platform.numberOfProcessors});
}
void batch(){
 identities();final log=File('$raw/paired-samples.jsonl');require(!log.existsSync()&&!File('$raw/batch-started.json').existsSync(),'Existing batch; no retries');
 save('$raw/batch-started.json',{'started_utc':DateTime.now().toUtc().toIso8601String(),'planned':56});
 final started=<int>[];var accepted=0;
 try{final specs=cases();require(specs.length==2,'Two cases');
  for(var c=0;c<2;c++){
   for(var v=0;v<2;v++){currentVariant=v==0?'old':'new';final context=c*2+v;started.add(context);agent(context,start:true);initVault(context);}
   final s=specs[c];
   for(var r=0;r<14;r++)for(var position=0;position<2;position++){
    currentVariant=(r+c+position).isEven?'old':'new';final context=c*2+(currentVariant=='old'?0:1);final env=environment(context);
    final id='c$c-r$r-$currentVariant';final archive='${s['root']}/$id.lbox';final destination='${s['root']}/$id-output';
    require(!File(archive).existsSync()&&!Directory(destination).existsSync(),'Fresh outputs');budget(3*(s['bytes'] as int)+256*1024*1024);
    final components=<List<double>>[];
    components.add(timed(createArgs(archive),env,'$id-create'));
    components.add(timed([cli,archive,'open'],env,'$id-open'));
    cold(s['source'] as String,env,'$id-source');
    components.add(timed([cli,archive,'add',s['source'] as String,'--to','/${s['basename']}'],env,'$id-add'));
    final write=sumCommands(components);
    final drain=timed(['/usr/bin/sync','-d','--',archive,File(archive).parent.path],env,'$id-write-drain');
    budget(s['bytes'] as int);final codec=inspectArchive(s,archive,env,id);final archiveHash=sha(archive),archiveBytes=File(archive).lengthSync();require(archiveBytes>0,'Nonempty archive');
    run([cli,archive,'extract','--to',destination],env,'$id-verify-extract',maximum:600);
    verifyTree({...s,'expected_mode':0x180},destination,env,id);
    log.writeAsStringSync('${jsonEncode({'case':s['case'],'round':r,'position':position,'variant':currentVariant,'preparation':r<2,'operation':'protected_write','components_create_open_add':components,'elapsed_seconds':write[0],'drain_elapsed_seconds':drain[0],'output_drained_elapsed_seconds':write[0]+drain[0],'user_seconds':write[1],'system_seconds':write[2],'rss_bytes':write[3]*1024,'archive_bytes':archiveBytes,'archive_sha256':archiveHash,'source_sha256':s['source_sha256'],'codec':codec,'output_mode':0x180,'persisted_bytes_mode_tree_verified':true,'cold_input':true,'executable':cli})}\n',mode:FileMode.append,flush:true);accepted++;
    Directory(destination).deleteSync(recursive:true);File(archive).deleteSync();
   }
  }
 }finally{currentVariant='old';cleanupAgents(started);}
 identities();require(accepted==56,'Exactly56 writes');save('$raw/paired-complete.json',{'accepted':56,'measured':48,'preparation':8,'identities_verified_after':true,'owned_agents_stopped':true,'private_contexts_removed':true,'wall_seconds':wall.elapsedMilliseconds/1000});print(green('56 protected write lifecycles independently verified.'));
}
void main(List<String> args){require(Platform.isLinux&&args.length==1,'Linux setup|freeze|batch');final mask=DynamicLibrary.process().lookupFunction<Uint32 Function(Uint32),int Function(int)>('umask');final previous=mask(0x12);try{verifyCurrentBuild();Directory(raw).createSync(recursive:true);counter=Directory('$raw/commands').existsSync()?Directory('$raw/commands').listSync().whereType<File>().length:0;switch(args.single){case 'setup':setup();break;case 'freeze':freeze();break;case 'batch':batch();break;default:throw ArgumentError('phase');}}finally{mask(previous);}}
