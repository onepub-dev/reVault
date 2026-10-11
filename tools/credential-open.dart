#! /usr/bin/env dcli
import 'dart:convert';
import 'dart:io';
import 'package:dcli/dcli.dart';

const worktree='/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance';
const evidence='$worktree/docs/evidence/credential-open-2026-10-12';
const raw='$evidence/raw';
const owned='/home/bsutton/.cache/rv-credential-20261012';
const oldOwned='/home/bsutton/.cache/revault-native-public-20261011';
const bin='$worktree/rust/target/credential-open-20261012/bin';
const cli='$bin/lockbox-old';
final synthetic=base64Encode(utf8.encode('public benchmark fixture credential 20261012'));
final wall=Stopwatch()..start();
int counter=0;
Map<String,dynamic> obj(String p)=>jsonDecode(File(p).readAsStringSync()) as Map<String,dynamic>;
void require(bool b,String m) {if(!b) throw StateError(m);}
void jsonFile(String p,Object v) {File(p).parent.createSync(recursive:true);File(p).writeAsStringSync('${const JsonEncoder.withIndent('  ').convert(v)}\n',flush:true);}
String sha(String p) {final r=Process.runSync('/usr/bin/sha256sum',['--',p]);require(r.exitCode==0,'hash failed');return (r.stdout as String).split(' ').first;}
Map<String,String> environment(int c) {
 final p='$owned/c$c'; final env=<String,String>{'PATH':'/usr/bin:/bin','TMPDIR':'/home/bsutton/.cache/rv4-security-20261011','HOME':'$p/home','XDG_CONFIG_HOME':'$p/config','XDG_CACHE_HOME':'$p/cache','XDG_STATE_HOME':'$p/state','XDG_DATA_HOME':'$p/data','LOCKBOX_SESSION_AGENT_DIR':'$p/agent','LOCKBOX_SESSION_AGENT_LOG':'$p/agent.log','LOCKBOX_VAULT_DIR':'$p/vault','LOCKBOX_PLATFORM_SECRET_STORE':'disabled','LOCKBOX_ADD_PROGRESS':'off','LOCKBOX_PASSWORD':synthetic};
 for(final k in ['HOME','XDG_CONFIG_HOME','XDG_CACHE_HOME','XDG_STATE_HOME','XDG_DATA_HOME','LOCKBOX_SESSION_AGENT_DIR','LOCKBOX_VAULT_DIR']) Directory(env[k]!).createSync(recursive:true);
 require(utf8.encode('${env['LOCKBOX_SESSION_AGENT_DIR']}/agent.sock').length<=100,'socket path');return env;
}
ProcessResult run(List<String> args,Map<String,String> env,String label,{int maxSeconds=180,bool mustPass=true}) {
 final remaining=1800-wall.elapsed.inSeconds-15; require(remaining>0,'30 minute batch cap');
 final seconds=remaining<maxSeconds?remaining:maxSeconds;
 final command=['--kill-after=10s','$seconds',...args];
 final r=Process.runSync('/usr/bin/timeout',command,environment:env,includeParentEnvironment:false);
 final safeOut=(r.stdout as String).replaceAll(synthetic,'[synthetic credential redacted]');
 final safeErr=(r.stderr as String).replaceAll(synthetic,'[synthetic credential redacted]');
 final stem='$raw/commands/${(counter++).toString().padLeft(4,'0')}-$label';
 File(stem).parent.createSync(recursive:true);File('$stem.stdout').writeAsStringSync(safeOut);File('$stem.stderr').writeAsStringSync(safeErr);
 jsonFile('$stem.json',{'argv':['/usr/bin/timeout',...command],'exit_code':r.exitCode,'timed_out':r.exitCode==124||r.exitCode==137,'stdout':'$stem.stdout','stderr':'$stem.stderr'});
 if(mustPass) require(r.exitCode==0,'Command failed: $label (${r.exitCode}), retained logs');return r;
}
int bytes(Directory d) {if(!d.existsSync())return 0;var n=0;for(final e in d.listSync(recursive:true,followLinks:false)) {if(e is File)n+=e.lengthSync();}return n;}
void budget(int next) {
 final total=bytes(Directory(owned))+bytes(Directory(oldOwned))+bytes(Directory(bin))+bytes(Directory('$worktree/rust/target/native-public-20261011/bin'))+bytes(Directory('$worktree/rust/target/native-extract-cache-budget-20261011/bin'))+bytes(Directory(evidence));
 require(total+next<=24*1024*1024*1024,'24GiB owned artifact cap');
 final r=Process.runSync('/usr/bin/df',['-Pk',owned]);require(r.exitCode==0,'df');final parts=(r.stdout as String).trim().split('\n').last.trim().split(RegExp(r'\s+'));require(int.parse(parts[3])*1024>=next+2*1024*1024*1024,'headroom');
}
List<Map<String,dynamic>> cases() => (jsonDecode(File('$raw/cases.json').readAsStringSync()) as List).cast<Map<String,dynamic>>();
void setup() {
 require(!File('$raw/cases.json').existsSync(),'Existing fixtures');Directory(owned).createSync(recursive:true);
 final previous=(jsonDecode(File('$worktree/docs/evidence/native-public-extraction-2026-10-11/raw/cases.json').readAsStringSync()) as List).where((v)=>(v['name'] as String).startsWith('random-raw')).toList();require(previous.length==2,'Pinned inputs');
 final records=<Map<String,dynamic>>[];
 try {for(var c=0;c<2;c++) {
  final spec=previous[c]; final input=(spec['variants'] as List).singleWhere((v)=>v['variant']=='A'); final source=input['source_path'] as String;
  require(sha(source)==input['source_sha256'],'Pinned source mismatch');budget((spec['bytes'] as int)*3);
  final root='$owned/fixture$c';require(!Directory(root).existsSync(),'Existing fixture root');Directory(root).createSync();
  final proof=File('$root/proof-source.bin')..writeAsStringSync('persisted credential-open proof $c\n');final archive='$root/archive.lbox';final env=environment(c);
  final session=run([cli,'session','--format','json'],env,'session$c');final rows=(session.stdout as String).split('\n').where((v)=>v.trim().isNotEmpty).map(jsonDecode).toList();require(rows.any((r)=>r is Map&&r['kind']=='agent'&&r['state']=='running'&&r['value']=='yes'),'Agent roundtrip');run(['/usr/bin/test','-S','${env['LOCKBOX_SESSION_AGENT_DIR']}/agent.sock'],env,'socket$c');
  run([cli,archive,'create','--password','--signing','none','--compression','none'],env,'create$c');
  run([cli,archive,'add',source,'--to','/payload.bin'],env,'add-payload$c');
  run([cli,archive,'add',proof.path,'--to','/proof.bin'],env,'add-proof$c');
  final output='$root/full-verify';run([cli,archive,'extract','--to',output],env,'full-extract$c');
  require(Directory(output).listSync().length==2,'Exact setup output tree');run(['/usr/bin/cmp','-s',source,'$output/payload.bin'],env,'full-compare$c');run(['/usr/bin/cmp','-s',proof.path,'$output/proof.bin'],env,'proof-compare$c');run(['/usr/bin/sync','-d','--','$output/payload.bin','$output/proof.bin',output],env,'drain$c');Directory(output).deleteSync(recursive:true);
  records.add({'case':spec['name'],'root':root,'bytes':spec['bytes'],'source':source,'source_sha256':input['source_sha256'],'archive_sha256':sha(archive),'archive_bytes':File(archive).lengthSync(),'proof_sha256':sha(proof.path),'full_payload_verified_by_separate_cli':true});
 }} finally {
 final cleanup=<Map<String,dynamic>>[];
 for(var c=0;c<2;c++) {
  final env=environment(c);final result=run([cli,'session','stop'],env,'stop$c',maxSeconds:20,mustPass:false);
  final absent=run(['/usr/bin/test','!','-S','${env['LOCKBOX_SESSION_AGENT_DIR']}/agent.sock'],env,'stopped-socket$c',maxSeconds:5,mustPass:false);
  cleanup.add({'case':c,'stop_exit_code':result.exitCode,'socket_absent':absent.exitCode==0,'passed':result.exitCode==0&&absent.exitCode==0});
 }
 jsonFile('$raw/setup-cleanup.json',{'rows':cleanup,'passed':cleanup.every((v)=>v['passed']==true)});
 require(cleanup.every((v)=>v['passed']==true),'Owned agent cleanup incomplete');
 }
 jsonFile('$raw/cases.json',records);print(green('Two public CLI password fixtures fully verified.'));
}
void freeze(String phase) {
 require(sha(cli)=='5a067b807308308d51e7bd7352fa97584039c49c3277860ea55c2fbcf1d1994f' && sha('$bin/evaluator-old')=='c69fd47638451df74f3b4fee8fddb52fdf2967bf7a8d26229bd82ce99d34cbd9','Old executable differs from completed release build');
 require(obj('$raw/setup-cleanup.json')['passed']==true,'Setup cleanup incomplete');
 if(phase=='paired') {
  require(sha('$bin/evaluator-new')=='6934b691d767a4836681853a18ea87830712c6d7d407bcb95bbbc94203329bb4' && sha('$bin/lockbox-new')=='b0fb323753105217dd780e6bc3cdfb8e29af4b5f9044ef6a9835c63d913d994b','New executable differs from release build');
  final built=File('$raw/build-new/manifest.txt').readAsStringSync();
  for(final relative in ['revault_lockbox_api/src/lockbox/key_management.rs','revault_lockbox_api/benches/evaluation/runner.rs']) require(built.contains('${sha('$worktree/rust/$relative')}  $relative'),'New source differs from release build');
 }

 final path='$raw/$phase-identities.json';require(!File(path).existsSync(),'Existing frozen epoch');final hashes=<String,String>{};
 final sourceList=Process.runSync('git',['ls-files','rust/revault_lockbox_api/src','rust/revault_lockbox_api/Cargo.toml','rust/Cargo.lock','rust-toolchain.toml'],workingDirectory:worktree);require(sourceList.exitCode==0,'source inventory');
 for(final relative in (sourceList.stdout as String).split('\n').where((v)=>v.isNotEmpty)) hashes['$worktree/$relative']=sha('$worktree/$relative');
 for(final p in [Platform.script.toFilePath(),if(phase=='paired')'$raw/build-new/manifest.txt','$evidence/README.md','$raw/cases.json','$worktree/rust/revault_lockbox_api/benches/evaluation/runner.rs',cli,'$bin/evaluator-old',if(phase=='paired')'$bin/evaluator-new']) hashes[p]=sha(p);
 for(final c in cases()) {for(final p in ['${c['root']}/archive.lbox','${c['root']}/proof-source.bin',c['source'] as String])hashes[p]=sha(p);require(hashes['${c['root']}/archive.lbox']==c['archive_sha256']&&hashes[c['source']]==c['source_sha256'],'Fixture identity');}
 jsonFile(path,{'phase':phase,'hashes':hashes,'planned_processes':phase=='baseline'?8:56,'utc':DateTime.now().toUtc().toIso8601String()});
}
void verify(String phase) {for(final e in (obj('$raw/$phase-identities.json')['hashes'] as Map).entries) require(sha(e.key)==e.value,'Changed identity: ${e.key}');}
void cold(String archive,Map<String,String> env,String id) {
 for(var a=0;a<3;a++) {run(['/usr/bin/sync','-d','--',archive],env,'$id-sync$a',maxSeconds:30);run(['/usr/bin/fadvise','--advice','dontneed',archive],env,'$id-evict$a',maxSeconds:30);final r=run(['/usr/bin/fincore','--json','--bytes','--output','SIZE,RES,FILE',archive],env,'$id-res$a',maxSeconds:30);final rows=jsonDecode(r.stdout)['fincore'] as List;if(int.parse('${rows.single['res']}')==0)return;}
 throw StateError('Cold admission failed; preserved attempts');
}
void batch(String phase) {
 verify(phase);final rowsFile=File('$raw/$phase-samples.jsonl');require(!rowsFile.existsSync(),'Existing batch');var accepted=0;final prep=phase=='baseline'?1:2;final rounds=phase=='baseline'?4:14;
 final specs=cases();for(var c=0;c<specs.length;c++) {final spec=specs[c];final root=spec['root'] as String;final env=environment(c);
 for(var r=0;r<rounds;r++) {final order=phase=='baseline'?['old']:((r+c)%2==0?['old','new']:['new','old']);for(var p=0;p<order.length;p++) {
  final v=order[p];final id='$phase-c$c-r$r-p$p-$v';final output='$root/$id.proof';require(!File(output).existsSync(),'Output exists');budget(1024);cold('$root/archive.lbox',env,id);
  final time='$raw/$id.time';final executable='$bin/evaluator-$v';final command=['/usr/bin/time','-f','%e %U %S %M','-o',time,'/usr/bin/taskset','-c','0-4',executable,'open-password',root,output];
  final result=run(command,env,id);final record=jsonDecode((result.stdout as String).trim()) as Map;require(record['verified']==true&&record['operation']=='public-password-open-proof'&&record['whole_payload_verified_in_timed_operation']==false,'Endpoint gate');
  final metrics=File(time).readAsStringSync().trim().split(RegExp(r'\s+')).map(double.parse).toList();require(metrics.length==4&&metrics.every((v)=>v.isFinite&&v>=0)&&metrics[0]>0&&metrics[3]>0,'Metrics gate');
  run(['/usr/bin/cmp','-s','$root/proof-source.bin',output],env,'$id-independent-proof');require(sha(output)==spec['proof_sha256'],'Output hash');run(['/usr/bin/sync','-d','--',output,root],env,'$id-drain');File(output).deleteSync();
  rowsFile.writeAsStringSync('${jsonEncode({'case':spec['case'],'variant':v,'round':r,'position':p,'preparation':r<prep,'elapsed_seconds':metrics[0],'user_seconds':metrics[1],'system_seconds':metrics[2],'rss_bytes':metrics[3]*1024,'endpoint':record,'executable':executable,'proof_verified':true,'cold_archive':true,'output_drained':true})}\n',mode:FileMode.append,flush:true);accepted++;
 }}}
 verify(phase);require(accepted==(phase=='baseline'?8:56),'Exact matrix');jsonFile('$raw/$phase-complete.json',{'accepted':accepted,'identities_verified_after':true,'wall_seconds':wall.elapsedMilliseconds/1000});print(green('$phase completed: $accepted accepted.'));
}
void main(List<String> args) {
 require(args.length==1,'setup | freeze-baseline | baseline | freeze-paired | paired');Directory(raw).createSync(recursive:true);
 // Command labels are globally unique across separate invocations, preserving every raw result.
 counter=Directory('$raw/commands').existsSync()?Directory('$raw/commands').listSync().whereType<File>().length:0;
 switch(args.single) {case 'setup':setup();break;case 'freeze-baseline':freeze('baseline');break;case 'freeze-paired':freeze('paired');break;case 'baseline':batch('baseline');break;case 'paired':batch('paired');break;default:throw ArgumentError('unknown phase');}
}
