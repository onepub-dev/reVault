#! /usr/bin/env dcli
import 'dart:convert';
import 'dart:io';
import 'package:dcli/dcli.dart';
const w='/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance';
const root='$w/docs/evidence/public-zip-comparison-2026-10-12/raw';
void require(bool value,String message){if(!value)throw StateError(message);}
String sha(String path){final p=Process.runSync('/usr/bin/sha256sum',['--',path]);require(p.exitCode==0,'sha');return(p.stdout as String).split(' ').first;}
Map<String,dynamic> object(String path)=>jsonDecode(File(path).readAsStringSync()) as Map<String,dynamic>;
Map<String,num> stats(Iterable<num> values){final s=values.toList()..sort();require(s.length==3,'Three measured values');return{'median':s[1],'minimum':s.first,'maximum':s.last};}
void main(){
 final complete=object('$root/complete.json');require(complete['accepted']==32&&complete['measured']==24&&complete['preparation']==8&&complete['identities_verified_after']==true&&complete['owned_agents_stopped']==true,'Complete accepted batch');
 final ids=object('$root/baseline-identities.json');for(final e in(ids['hashes'] as Map).entries)require(sha(e.key)==e.value,'Frozen identity changed');
 final cases=(jsonDecode(File('$root/cases.json').readAsStringSync()) as List).cast<Map<String,dynamic>>();
 final rows=File('$root/samples.jsonl').readAsLinesSync().where((s)=>s.isNotEmpty).map((s)=>jsonDecode(s) as Map<String,dynamic>).toList();require(rows.length==32,'Exact rows');
 final seen=<String>{};
 for(final row in rows){
  final c=cases.indexWhere((c)=>c['case']==row['case']);final r=row['round'];final v=row['variant'];final action=row['operation'];
  require(c>=0&&r is int&&r>=0&&r<4&&['native','zip'].contains(v)&&['write','read'].contains(action),'Expected coordinates');
  final order=(r+c).isEven?['native','zip']:['zip','native'];require(row['position']==order.indexOf(v)&&row['preparation']==(r==0),'Order/preparation');
  require(seen.add('$c/$r/$v/$action'),'No duplicate');require(row['source_sha256']==cases[c]['source_sha256']&&row['persisted_bytes_mode_tree_verified']==true&&row['cold_input']==true,'Correctness admission');
  for(final k in ['cli_elapsed_seconds','drain_elapsed_seconds','output_drained_elapsed_seconds','user_seconds','system_seconds','rss_bytes','drain_rss_bytes','archive_bytes']){final x=row[k];require(x is num&&x.isFinite&&x>=0,'Metric $k');}
  require(row['output_mode']==(v=='native'?0x180:0x1b4),'Actual permission-policy gate');
  require(row['rss_bytes']>0&&row['archive_bytes']>0&&row['cli_elapsed_seconds']>0,'Positive resources');
  require((row['output_drained_elapsed_seconds']-row['cli_elapsed_seconds']-row['drain_elapsed_seconds']).abs()<0.000001,'Time sum');
  final executable=v=='native'?'$w/rust/target/public-zip-comparison-20261012/bin/lockbox-current':(action=='write'?'/usr/bin/zip':'/usr/bin/unzip');require(row['executable']==executable&&(ids['hashes'] as Map).containsKey(executable),'Executable binding');
 }
 final results=<Map<String,dynamic>>[];
 for(final c in cases){for(final action in ['write','read']){
  final variants=<String,dynamic>{};
  for(final v in ['native','zip']){
   final samples=rows.where((r)=>r['case']==c['case']&&r['operation']==action&&r['variant']==v&&r['preparation']==false).toList();require(samples.length==3,'Exact measured group');
   variants[v]={for(final metric in ['cli_elapsed_seconds','drain_elapsed_seconds','output_drained_elapsed_seconds','rss_bytes','archive_bytes'])metric:stats(samples.map((r)=>r[metric] as num)),'cpu_seconds':stats(samples.map((r)=>(r['user_seconds'] as num)+(r['system_seconds'] as num))),'observed_codecs':samples.map((r)=>r['codec']).toSet().toList(),'output_modes':samples.map((r)=>r['output_mode']).toSet().toList()};
  }
  results.add({'case':c['case'],'operation':action,'variants':variants,'native_over_zip_cli_median_ratio':variants['native']['cli_elapsed_seconds']['median']/variants['zip']['cli_elapsed_seconds']['median'],'native_over_zip_drained_median_ratio':variants['native']['output_drained_elapsed_seconds']['median']/variants['zip']['output_drained_elapsed_seconds']['median']});
 }}
 final summary={'accepted':32,'measured':24,'preparation':8,'method':'three measured repetitions; medians and full ranges only; no inferential qualification','results':results,'samples_sha256':sha('$root/samples.jsonl'),'identities_sha256':sha('$root/baseline-identities.json'),'analysis_sha256':sha(Platform.script.toFilePath()),'scope':'Current default-layout native public CLI restoring stored0600 versus Info-ZIP restoring source0664; plaintext unsigned Zstd3/default padding versus Deflate6/observed fallback and CRC; unequal protection/recovery; single1GiB files only; process peak excludes helper agent.'};
 File('$root/metrics-summary.json').writeAsStringSync('${const JsonEncoder.withIndent('  ').convert(summary)}\n',flush:true);
 print(green('Public ZIP comparison audited:32 operations,24 measured; descriptive summary saved.'));
}
