#! /usr/bin/env dcli
import 'dart:convert';
import 'dart:io';
import 'dart:math' as math;
import 'package:dcli/dcli.dart';
import 'summarize-adaptive-workers.dart' as stats;
const worktree='/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance';
const root='$worktree/docs/evidence/credential-open-2026-10-12/raw';
void require(bool v,String m)=>stats.require(v,m);
Map<String,dynamic> paired(String label,List<Map<String,dynamic>> rows,{int? stratum}) {
 final rounds=[for(var r=2;r<14;r++)if(stratum==null||r%2==stratum)r];
 final logs=[for(final r in rounds)math.log(stats.number(rows.singleWhere((v)=>v['round']==r&&v['variant']=='new')['elapsed_seconds'],'new')/stats.number(rows.singleWhere((v)=>v['round']==r&&v['variant']=='old')['elapsed_seconds'],'old'))];
 final blocks=stratum==null?[for(var i=0;i<logs.length;i+=2)(logs[i]+logs[i+1])/2]:logs;
 require(blocks.length==6,'Six bootstrap blocks');final seed=stats.seedFor(label);final random=stats.Seeded(seed);
 final draws=List<double>.generate(10000,(_){var total=0.0;for(var n=0;n<6;n++)total+=blocks[random.nextInt(6)];return math.exp(total/6);})..sort();
 return {'geometric_mean_ratio':math.exp(logs.reduce((a,b)=>a+b)/logs.length),'low95':stats.percentile(draws,.025),'high95':stats.percentile(draws,.975),'pairs':rounds.length,'blocks':6,'replicates':10000,'seed':seed,'method':stratum==null?'six adjacent two-round paired log-ratio blocks':'six within-round-stratum paired log ratios'};
}
void main() {
 final output=File('$root/metrics-summary.json');require(!output.existsSync(),'Existing summary');
 final complete=stats.readObject('$root/paired-complete.json');require(complete['accepted']==56&&complete['identities_verified_after']==true,'Incomplete run');
 final rows=File('$root/paired-samples.jsonl').readAsLinesSync().map((v)=>jsonDecode(v) as Map<String,dynamic>).toList();require(rows.length==56,'Exact56');
 final cases=(jsonDecode(File('$root/cases.json').readAsStringSync()) as List).cast<Map<String,dynamic>>();require(cases.length==2,'Two cases');
 final results=<Map<String,dynamic>>[];var retain=true;
 for(var c=0;c<2;c++) {
  final name=cases[c]['case'];final selected=rows.where((r)=>r['case']==name).toList();require(selected.length==28,'Exact28 percase');
  for(var r=0;r<14;r++) for(var p=0;p<2;p++) {
   final variant=(p+r+c)%2==0?'old':'new';final row=selected.singleWhere((v)=>v['round']==r&&v['position']==p);
   require(row['variant']==variant&&row['preparation']==(r<2)&&row['executable']=='$worktree/rust/target/credential-open-20261012/bin/evaluator-$variant','Fixed schedule/executable');
   require(row['proof_verified']==true&&row['cold_archive']==true&&row['output_drained']==true&&row['endpoint']['verified']==true&&row['endpoint']['whole_payload_verified_in_timed_operation']==false,'Output/measurement gates');
   for(final key in ['elapsed_seconds','rss_bytes'])stats.number(row[key],key);
  }
  final measured=selected.where((v)=>v['preparation']==false).toList();final elapsed=paired('$name/aggregate',measured);final views=[for(var p=0;p<2;p++)paired('$name/stratum$p',measured,stratum:p)];
  final variants=<String,dynamic>{};for(final variant in ['old','new']) {
   final group=measured.where((v)=>v['variant']==variant).toList();final times=group.map((v)=>stats.number(v['elapsed_seconds'],'elapsed')).toList()..sort();final rss=group.map((v)=>stats.number(v['rss_bytes'],'rss')).toList()..sort();
   variants[variant]={'median_seconds':stats.percentile(times,.5),'maximum_rss_bytes':rss.last,'every_rss_bytes':rss};
  }
  final passed=elapsed['high95']<=1.05&&variants['new']['maximum_rss_bytes']<100000000;retain=retain&&passed;
  results.add({'case':name,'aggregate':elapsed,'round_strata':views,'variants':variants,'retention_gate':passed});
 }
 output.writeAsStringSync('${const JsonEncoder.withIndent('  ').convert({'accepted':56,'measured':48,'preparation':8,'retain':retain,'results':results,'samples_sha256':stats.sha('$root/paired-samples.jsonl'),'identities_sha256':stats.sha('$root/paired-identities.json'),'analysis_script_sha256':stats.sha(Platform.script.toFilePath()),'statistics_helper_sha256':stats.sha('$worktree/tools/summarize-adaptive-workers.dart'),'scope':'Direct public password API unlock + persisted tiny proof; no contact or CLI-unlock performance claim'})}\n');
 print(green('Credential-open retention: $retain;56 audited.'));
 for(final r in results) print(jsonEncode(r));
}
