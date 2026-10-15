#! /usr/bin/env dcli
import 'dart:io';
import 'dart:convert';
import 'package:dcli/dcli.dart';
void require(bool b,String m){if(!b)throw StateError(m);}
Map<String,dynamic> object(String p)=>jsonDecode(File(p).readAsStringSync()) as Map<String,dynamic>;
Map<String,num> stats(List<num> v){v.sort((a,b)=>a.compareTo(b));require(v.length==3,'Three measured');return {'minimum':v.first,'median':v[1],'maximum':v.last};}
void main(List<String> args){
 require(args.length==1,'Evidence root');final raw='${args.single}/raw';final done=object('$raw/complete.json');require(done['accepted']==16&&done['measured']==12&&done['preparation']==4&&done['identities_verified_after']==true&&done['owned_agents_stopped']==true&&done['private_contexts_removed']==true,'Terminal success');
 final specs=(jsonDecode(File('$raw/cases.json').readAsStringSync()) as List).cast<Map<String,dynamic>>();require(specs.length==4,'Four cohorts');
 final rows=File('$raw/samples.jsonl').readAsLinesSync().map((l)=>jsonDecode(l) as Map<String,dynamic>).toList();require(rows.length==16,'Fixed16');final results=<Map<String,dynamic>>[];
 for(var c=0;c<4;c++){final group=<Map<String,dynamic>>[];for(var r=0;r<4;r++){final row=rows[c*4+r];require(row['case']==specs[c]['case']&&row['round']==r&&row['preparation']==(r==0),'Exact schedule');require(row['exact_tree_bytes_modes_verified']==true&&row['cold_archive']==true&&row['output_drained']==true&&row['archive_sha256']==specs[c]['archive_sha256']&&row['source_sha256']==specs[c]['source_sha256'],'Admission');
  for(final key in ['elapsed_seconds','user_seconds','system_seconds','rss_bytes','helper_inclusive_peak_upper_bound_bytes'])require(row[key] is num&&(row[key] as num).isFinite&&row[key]>=0,'Finite metrics');
  final snapshots=(row['agent_snapshots'] as List).cast<Map<String,dynamic>>();require(snapshots.length==2,'Two snapshots');for(final s in snapshots)require(s['pid']==snapshots.first['pid']&&s['start_ticks']==snapshots.first['start_ticks']&&(s['children'] as List).isEmpty&&s['rss_bytes']>0&&s['hwm_bytes']>=s['rss_bytes'],'Owned stable agent');require(snapshots.last['hwm_bytes']>=snapshots.first['hwm_bytes'],'Monotonic HWM');require(row['helper_inclusive_peak_upper_bound_bytes']==row['rss_bytes']+snapshots.last['hwm_bytes'],'Bound arithmetic');if(r>0)group.add(row);
 }results.add({'case':specs[c]['case'],'archive_bytes':specs[c]['archive_bytes'],'files':(specs[c]['files'] as List).length,'logical_bytes':specs[c]['bytes'],'compressed':specs[c]['compressed'],for(final key in ['elapsed_seconds','rss_bytes','helper_inclusive_peak_upper_bound_bytes'])key:stats(group.map((r)=>r[key] as num).toList()),'preparation':rows[c*4]});}
 File('$raw/summary.json').writeAsStringSync(const JsonEncoder.withIndent('  ').convert({'descriptive_only':true,'scope':'Current default plaintext many-file extraction, CLI peak plus owned agent HWM; no paired historical inference','cases':results})+'\n');print(green('Sixteen verified extractions summarized.'));
}
