#! /usr/bin/env dcli
import 'dart:convert';
import 'dart:io';
import 'dart:math' as math;
import 'package:dcli/dcli.dart';
import '../summarize-adaptive-workers.dart' as stats;
const wt='/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance';
const root='$wt/docs/evidence/extraction-last-use-2026-10-12/raw';
void require(bool v,String m)=>stats.require(v,m);
Map<String,dynamic> paired(String label,List<Map<String,dynamic>> rows,{int? stratum}){
 final rounds=[for(var r=2;r<14;r++)if(stratum==null||r%2==stratum)r];
 final logs=[for(final r in rounds)math.log(stats.number(rows.singleWhere((v)=>v['round']==r&&v['variant']=='new')['elapsed_seconds'],'new')/stats.number(rows.singleWhere((v)=>v['round']==r&&v['variant']=='old')['elapsed_seconds'],'old'))];
 final blocks=stratum==null?[for(var i=0;i<logs.length;i+=2)(logs[i]+logs[i+1])/2]:logs;
 require(blocks.length==6,'Six blocks');final seed=stats.seedFor(label),random=stats.Seeded(stats.seedFor(label));
 final draws=List<double>.generate(10000,(_){var total=0.0;for(var n=0;n<6;n++)total+=blocks[random.nextInt(6)];return math.exp(total/6);})..sort();
 return {'geometric_mean_ratio':math.exp(logs.reduce((a,b)=>a+b)/logs.length),'low95':stats.percentile(draws,.025),'high95':stats.percentile(draws,.975),'pairs':rounds.length,'blocks':6,'replicates':10000,'seed':seed};
}
void main(){
 final output=File('$root/metrics-summary.json');require(!output.existsSync(),'Existing summary');
 final complete=stats.readObject('$root/paired-complete.json');require(complete['accepted']==112&&complete['measured']==96&&complete['preparation']==16&&complete['identities_verified_after']==true&&complete['owned_agents_stopped']==true&&complete['private_contexts_removed']==true,'Incomplete');
 final identities=stats.readObject('$root/paired-identities.json'),executables=stats.readObject('${File(Platform.script.toFilePath()).parent.path}/build-inputs.json')['variants'] as Map;
 require(jsonEncode(identities['variants'])==jsonEncode(executables),'Variant identity');
 final rows=File('$root/paired-samples.jsonl').readAsLinesSync().map((v)=>jsonDecode(v) as Map<String,dynamic>).toList();require(rows.length==112,'Exact112');
 final cases=(jsonDecode(File('$root/cases.json').readAsStringSync()) as List).cast<Map<String,dynamic>>();require(cases.length==4,'Four cases');
 final results=<Map<String,dynamic>>[];var guards=true,useful=true,usefulCases=0;
 for(var c=0;c<4;c++){
  final name=cases[c]['case'] as String,selected=rows.sublist(c*28,(c+1)*28);
  final sizes=<double>{};
  for(var r=0;r<14;r++)for(var p=0;p<2;p++){
   final row=selected[r*2+p],variant=(r+p+c).isEven?'old':'new';
   require(row['case']==name&&row['round']==r&&row['position']==p&&row['variant']==variant&&row['preparation']==(r<2)&&row['executable']==executables[variant],'Exact schedule/executable');
   require(row['exact_tree_bytes_modes_verified']==true&&row['cold_archive']==true&&row['output_drained']==true&&row['source_sha256']==cases[c]['source_sha256'],'Verification admission');
   for(final key in ['elapsed_seconds','rss_bytes','helper_inclusive_peak_upper_bound_bytes'])stats.number(row[key],key);
   for(final key in ['user_seconds','system_seconds'])stats.number(row[key],key,positive:false);
   require(row['archive_sha256']==cases[c]['archive_sha256'],'Immutable archive');sizes.add(stats.number(cases[c]['archive_bytes'],'archive'));final snaps=(row['agent_snapshots'] as List).cast<Map<String,dynamic>>();require(snaps.length==2,'Two agent snapshots');for(final snap in snaps)require(snap['pid']==snaps.first['pid']&&snap['start_ticks']==snaps.first['start_ticks']&&(snap['children'] as List).isEmpty&&snap['rss_bytes']>0&&snap['hwm_bytes']>=snap['rss_bytes'],'Stable owned agent');require(snaps.last['hwm_bytes']>=snaps.first['hwm_bytes']&&row['helper_inclusive_peak_upper_bound_bytes']==row['rss_bytes']+snaps.last['hwm_bytes'],'Bound arithmetic');
  }
  require(sizes.length==1,'Equal archive size across variants');
  final measured=selected.where((r)=>r['preparation']==false).toList(),aggregate=paired('$name/aggregate',selected);
  final variants=<String,dynamic>{};
  for(final variant in ['old','new']){
   final group=measured.where((r)=>r['variant']==variant).toList();
   final time=group.map((r)=>stats.number(r['elapsed_seconds'],'elapsed')).toList()..sort();
   final rss=group.map((r)=>stats.number(r['rss_bytes'],'rss')).toList()..sort();
   final allRss=selected.where((r)=>r['variant']==variant).map((r)=>stats.number(r['rss_bytes'],'rss')).toList()..sort();
   variants[variant]={'median_seconds':stats.percentile(time,.5),'minimum_seconds':time.first,'maximum_seconds':time.last,'median_rss_bytes':stats.percentile(rss,.5),'maximum_rss_bytes':rss.last,'maximum_including_preparation_rss_bytes':allRss.last,'every_rss_bytes':rss};
  }
  final maxRatio=variants['new']['maximum_rss_bytes']/variants['old']['maximum_rss_bytes'],medianRatio=variants['new']['median_rss_bytes']/variants['old']['median_rss_bytes'];
  final guard=aggregate['high95']<=1.05&&maxRatio<=1.05;guards=guards&&guard;
  if(name=='1024-files-1gib'||name=='1024-source-files-1gib'){usefulCases++;useful=useful&&medianRatio<=.80;}
  results.add({'case':name,'aggregate':aggregate,'round_strata':[for(var p=0;p<2;p++)paired('$name/stratum$p',selected,stratum:p)],'variants':variants,'archive_bytes':sizes.single,'maximum_rss_ratio':maxRatio,'median_rss_ratio':medianRatio,'nonregression_gate':guard});
 }
 require(usefulCases==2,'Both prespecified large cohorts');
 output.writeAsStringSync('${const JsonEncoder.withIndent('  ').convert({'accepted':112,'measured':96,'preparation':16,'retain':guards&&useful,'all_nonregression_gates':guards,'both_large_median_memory_gates':useful,'results':results,'samples_sha256':stats.sha('$root/paired-samples.jsonl'),'identities_sha256':stats.sha('$root/paired-identities.json'),'analysis_script_sha256':stats.sha(Platform.script.toFilePath()),'statistics_helper_sha256':stats.sha('$wt/tools/summarize-adaptive-workers.dart'),'scope':'Plaintext many-file extraction, CLI process RSS retention gate; agent-inclusive bounds separately recorded; preparation reported separately; no universal100MB claim.'})}\n',flush:true);
 print(green('Last-use extraction frozen retention decision: ${guards&&useful}'));
}
