#! /usr/bin/env dcli
import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:dcli/dcli.dart';

const worktree='/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance';
const rust='$worktree/rust';
const evidence='$worktree/docs/evidence/security-review-2026-10-13/raw/full-runtime-api-native-recheck';
const pinsPath='$evidence/integration-pins.json';
const maxRuntime=Duration(seconds:5400),killGrace=Duration(seconds:30),samplePeriod=Duration(seconds:5);
const maxSamples=1100;
void require(bool ok,String message){if(!ok)throw StateError(message);}
void save(String p,Object v)=>File(p).writeAsStringSync('${const JsonEncoder.withIndent('  ').convert(v)}\n',flush:true);
String hashFile(String path){final r=Process.runSync('/usr/bin/sha256sum',['--',path]);require(r.exitCode==0,'sha256sum failed for $path');return (r.stdout as String).split(RegExp(r'\s+')).first;}
String currentSourceManifest(String manifestPath){final lines=File(manifestPath).readAsLinesSync();return lines.map((line){final m=RegExp(r'^([0-9a-f]{64})  (.+)$').firstMatch(line);require(m!=null,'malformed frozen runtime manifest');final path=m!.group(2)!;return '${hashFile('$rust/$path')}  $path\n';}).join();}
String runText(String exe,List<String> args,{required String cwd,String? stdoutPath,String? stderrPath}){
 final r=Process.runSync(exe,args,workingDirectory:cwd,environment:{'TMPDIR':'/home/bsutton/.cache/rv4-security-20261011','NO_COLOR':'1','TERM':'dumb'},includeParentEnvironment:true);
 if(stdoutPath!=null)File(stdoutPath).writeAsStringSync(r.stdout as String,flush:true);
 if(stderrPath!=null)File(stderrPath).writeAsStringSync(r.stderr as String,flush:true);
 require(r.exitCode==0,'preflight command failed: $exe ${args.join(' ')}');return r.stdout as String;
}
void preflight(){
 final pins=jsonDecode(File(pinsPath).readAsStringSync()) as Map<String,dynamic>;
 final executable=pins['executable'] as String;
 final expectedHead=pins['head'] as String;
 final expectedBinary=pins['executable_sha256'] as String;
 final expectedInventory=pins['inventory_count'] as int;
 final savedListPath=pins['inventory_list_path'] as String;
 final savedManifestPath=pins['source_manifest_path'] as String;
 final head=runText('/usr/bin/git',['rev-parse','HEAD'],cwd:worktree).trim();require(head==expectedHead,'HEAD mismatch: $head');
 final binary=hashFile(executable);require(binary==expectedBinary,'test executable SHA mismatch: $binary');
 final manifest=File(savedManifestPath).readAsLinesSync();var checked=0;
 for(final line in manifest){final m=RegExp(r'^([0-9a-f]{64})  (.+)$').firstMatch(line);require(m!=null,'malformed saved source manifest');final path='$rust/${m!.group(2)}';final actual=hashFile(path);require(actual==m.group(1),'runtime source changed: ${m.group(2)}');checked++;}
 final listed=runText(executable,['--list'],cwd:rust,stdoutPath:'$evidence/preflight-list.log',stderrPath:'$evidence/preflight-list.stderr');
 final count=listed.split('\n').where((line)=>line.endsWith(': test')).length;require(count==expectedInventory,'test inventory count $count != $expectedInventory');
 final previous=File(savedListPath).readAsStringSync();require(listed==previous,'test inventory differs from frozen --list output');
 save('$evidence/preflight.json',{'head':head,'binary_sha256':binary,'inventory_count':count,'inventory_matches_preserved_list':true,'source_manifest_entries_verified':checked,'source_manifest_sha256':hashFile(savedManifestPath),'test_argv':[executable,'--nocapture','--test-threads=2'],'filters':[],'ignored_tests_changed':false,'read_only':true,'max_runtime_seconds':maxRuntime.inSeconds,'kill_grace_seconds':killGrace.inSeconds});
}
int? statusKiB(String status,String key){for(final line in status.split('\n')){if(line.startsWith('$key:')){final f=line.trim().split(RegExp(r'\s+'));if(f.length>=2)return int.tryParse(f[1]);}}return null;}
Map<String,Object?> sample(int pid,int index){final now=DateTime.now().toUtc().toIso8601String();try{final s=File('/proc/$pid/status').readAsStringSync();final l=File('/proc/$pid/limits').readAsLinesSync().firstWhere((x)=>x.startsWith('Max locked memory'),orElse:()=> '');return {'sample':index,'time_utc':now,'pid':pid,'VmLck_kib':statusKiB(s,'VmLck'),'VmRSS_kib':statusKiB(s,'VmRSS'),'max_locked_memory_limit':l};}on FileSystemException{return {'sample':index,'time_utc':now,'pid':pid,'process_status':'exited'};}}
Future<void> main(List<String> args)async{
 if(args.contains('--syntax-check')){print(green('Dart runner parsed; no build or test process launched'));return;}
 final pins=jsonDecode(File(pinsPath).readAsStringSync()) as Map<String,dynamic>;
 final executable=pins['executable'] as String;
 final expectedInventory=pins['inventory_count'] as int;
 require(Platform.isLinux,'Linux /proc required');Directory(evidence).createSync(recursive:true);
 for(final name in ['preflight.json','preflight-list.log','preflight-list.stderr','stdout.raw','stderr.raw','stdout-timestamped.log','stderr-timestamped.log','resource-samples.jsonl','pid.txt','failure-trigger.json','terminal.json','source-manifest-post.sha256','postterminal-identities.json']){require(!File('$evidence/$name').existsSync(),'refusing to overwrite existing evidence: $name');}
 preflight();
 final rawOut=File('$evidence/stdout.raw').openWrite(),rawErr=File('$evidence/stderr.raw').openWrite();
 final outLog=File('$evidence/stdout-timestamped.log').openWrite(),errLog=File('$evidence/stderr-timestamped.log').openWrite(),sampleLog=File('$evidence/resource-samples.jsonl').openWrite();
 final started=DateTime.now().toUtc();
 final p=await Process.start(executable,['--nocapture','--test-threads=2'],workingDirectory:rust,environment:{'TMPDIR':'/home/bsutton/.cache/rv4-security-20261011','NO_COLOR':'1','TERM':'dumb'},includeParentEnvironment:true);
 File('$evidence/pid.txt').writeAsStringSync('${p.pid}\n',flush:true);
 print(green('full API integration started; owned test PID=${p.pid}; no filters; 5400s runtime + 30s grace'));
 final expectedNames=File(pins['inventory_list_path'] as String).readAsLinesSync().where((line)=>line.endsWith(': test')).map((line)=>line.substring(0,line.length-6)).toSet();
 final reportedNames=<String>{};var duplicateResults=false;List<int>? libtestSummary;
 final ring=<String>[];String? failedLine,captureFailure;var stopCause='';var sampleCount=0;var exited=false;var terminalResults=0,passedResults=0,failedResults=0,ignoredResults=0;Timer? graceTimer;
 void event(String stream,String line){final stamped='${DateTime.now().toUtc().toIso8601String()} [$stream] $line';ring.add(stamped);if(ring.length>300)ring.removeAt(0);(stream=='stdout'?outLog:errLog).writeln(stamped);
  final normalized=line.replaceAll(RegExp(r'\x1B\[[0-9;]*[A-Za-z]'),'').replaceAll('\r','').trimRight();
  final result=RegExp(r'^\s*test\s+(.+?)\s+\.\.\.\s+(ok|FAILED|ignored)(?:,\s*.*)?\s*$').firstMatch(normalized);
  if(result!=null){
   terminalResults++;if(!reportedNames.add(result.group(1)!))duplicateResults=true;
   switch(result.group(2)){case 'ok':passedResults++;case 'FAILED':failedResults++;case 'ignored':ignoredResults++;}
  }
  final summary=RegExp(r'^test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;').firstMatch(normalized);
  if(summary!=null)libtestSummary=[for(var i=1;i<=5;i++)int.parse(summary.group(i)!)];
  if(failedLine==null&&RegExp(r'^\s*test\s+.+\s+\.\.\.\s+FAILED\s*$').hasMatch(normalized)){
   failedLine=normalized;stopCause='individual_libtest_failure_result_line';final immediate=sample(p.pid,sampleCount++);sampleLog.writeln(jsonEncode(immediate));
   save('$evidence/failure-trigger.json',{'time_utc':DateTime.now().toUtc().toIso8601String(),'pid':p.pid,'line':normalized,'detection':'complete libtest result line only; panic text alone is not a trigger','immediate_resource_snapshot':immediate,'recent_timestamped_output':List<String>.from(ring)});
   Process.killPid(p.pid,ProcessSignal.sigterm);graceTimer=Timer(killGrace,(){if(!exited){stopCause='failure_line_sigkill_after_grace';Process.killPid(p.pid,ProcessSignal.sigkill);}});
  }
 }
 Future<void> drain(Stream<List<int>> stream,String name,IOSink raw){final done=Completer<void>();var pending='';stream.listen((b){raw.add(b);pending+=utf8.decode(b,allowMalformed:true);final lines=pending.split('\n');pending=lines.removeLast();for(final line in lines)event(name,line);},onDone:(){if(pending.isNotEmpty)event(name,pending);if(!done.isCompleted)done.complete();},onError:(Object error,StackTrace stack){captureFailure='${name}_stream_error: $error';if(!done.isCompleted)done.complete();});return done.future;}
 final drained=Future.wait([drain(p.stdout,'stdout',rawOut),drain(p.stderr,'stderr',rawErr)]);
 void takeSample(){if(sampleCount<maxSamples)sampleLog.writeln(jsonEncode(sample(p.pid,sampleCount++)));}
 takeSample();final observer=Timer.periodic(samplePeriod,(_)=>takeSample());
 final timeout=Timer(maxRuntime,(){if(!exited){stopCause='runtime_timeout';Process.killPid(p.pid,ProcessSignal.sigterm);graceTimer=Timer(killGrace,(){if(!exited){stopCause='timeout_sigkill_after_grace';Process.killPid(p.pid,ProcessSignal.sigkill);}});}});
 final code=await p.exitCode;exited=true;await drained;observer.cancel();timeout.cancel();graceTimer?.cancel();
 await Future.wait([rawOut.flush(),rawErr.flush(),outLog.flush(),errLog.flush(),sampleLog.flush()]);await Future.wait([rawOut.close(),rawErr.close(),outLog.close(),errLog.close(),sampleLog.close()]);
 final expectedHead=pins['head'] as String;final expectedBinary=pins['executable_sha256'] as String;final savedManifestPath=pins['source_manifest_path'] as String;
 final elapsed=DateTime.now().toUtc().difference(started).inMilliseconds/1000;
 final missing=expectedNames.difference(reportedNames).toList()..sort();final unexpected=reportedNames.difference(expectedNames).toList()..sort();
 final summaryMatches=libtestSummary!=null&&libtestSummary![0]==passedResults&&libtestSummary![1]==failedResults&&libtestSummary![2]==ignoredResults&&libtestSummary![3]==0&&libtestSummary![4]==0;
 final inventoryComplete=!duplicateResults&&missing.isEmpty&&unexpected.isEmpty&&summaryMatches;
 final status=failedLine!=null?'interrupted_incomplete_after_failure_line':stopCause.contains('timeout')?'timed_out':captureFailure!=null?'capture_incomplete':code==0&&terminalResults==expectedInventory&&failedResults==0&&inventoryComplete?'passed':code==0?'incomplete_result_inventory':'failed';
 save('$evidence/terminal.json',{'pid':p.pid,'exit_code':code,'status':status,'stop_cause':stopCause,'detected_failure_line':failedLine,'capture_failure':captureFailure,'elapsed_seconds':elapsed,'sample_count':sampleCount,'inventory':expectedInventory,'terminal_results':terminalResults,'passed_results':passedResults,'failed_results':failedResults,'ignored_results':ignoredResults,'complete_suite_pass':status=='passed','duplicate_results':duplicateResults,'missing_names':missing,'unexpected_names':unexpected,'libtest_summary':libtestSummary,'summary_matches_result_lines':summaryMatches});
 final postHead=runText('/usr/bin/git',['rev-parse','HEAD'],cwd:worktree).trim();final postBinary=hashFile(executable);final postManifest=currentSourceManifest(savedManifestPath);File('$evidence/source-manifest-post.sha256').writeAsStringSync(postManifest,flush:true);final sourceStable=postManifest==File(savedManifestPath).readAsStringSync();save('$evidence/postterminal-identities.json',{'head':postHead,'binary_sha256':postBinary,'source_manifest_sha256':hashFile('$evidence/source-manifest-post.sha256'),'source_manifest_unchanged':sourceStable});require(postHead==expectedHead&&postBinary==expectedBinary&&sourceStable,'postterminal identity drift');
 if(status!='passed')exitCode=1;
 print(green('full API terminal status=$status exit=$code results=$terminalResults/$expectedInventory passed=$passedResults failed=$failedResults ignored=$ignoredResults elapsed=${elapsed}s samples=$sampleCount'));
}
