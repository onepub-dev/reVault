#! /usr/bin/env dcli
import 'dart:convert';
import 'dart:io';
import 'package:dcli/dcli.dart';
void require(bool ok,String message){if(!ok)throw StateError(message);}
Map<String,dynamic> object(String path)=>jsonDecode(File(path).readAsStringSync()) as Map<String,dynamic>;
Map<String,num> stats(List<num> values){values.sort((a,b)=>a.compareTo(b));require(values.length==3,'Three measured repetitions');return {'minimum':values.first,'median':values[1],'maximum':values.last};}
void main(List<String> args){
 require(args.length==1,'Pass fresh evidence root');final raw='${args.single}/raw';
 final completion=object('$raw/complete.json');require(completion['accepted']==8&&completion['measured']==6&&completion['preparation']==2&&completion['identities_verified_after']==true&&completion['owned_agents_stopped']==true&&completion['private_contexts_removed']==true,'Complete verified batch required');
 final cases=(jsonDecode(File('$raw/cases.json').readAsStringSync()) as List).cast<Map<String,dynamic>>();require(cases.length==2,'Two cases');
 final rows=File('$raw/samples.jsonl').readAsLinesSync().where((l)=>l.isNotEmpty).map((l)=>jsonDecode(l) as Map<String,dynamic>).toList();require(rows.length==8,'Exact8 samples');
 final output=<Map<String,dynamic>>[];
 for(var c=0;c<2;c++){
  final group=<Map<String,dynamic>>[];
  for(var round=0;round<4;round++){
   final row=rows[c*4+round];require(row['case']==cases[c]['case']&&row['round']==round&&row['preparation']==(round==0)&&row['operation']=='protected_write','Exact fixed schedule');
   require(row['persisted_bytes_mode_tree_verified']==true&&row['cold_input']==true&&row['output_mode']==0x180&&row['source_sha256']==cases[c]['source_sha256'],'Verification admission');
   for(final key in ['cli_elapsed_seconds','drain_elapsed_seconds','output_drained_elapsed_seconds','rss_bytes','archive_bytes']){final n=row[key];require(n is num&&n.isFinite&&n>=0,'Finite $key');}
   require(row['rss_bytes']>0&&row['archive_bytes']>0,'Positive sizes');
   if(round>0)group.add(row);
  }
  output.add({'case':cases[c]['case'],'measured_repetitions':3,for(final key in ['cli_elapsed_seconds','drain_elapsed_seconds','output_drained_elapsed_seconds','rss_bytes','archive_bytes'])key:stats(group.map((r)=>r[key] as num).toList())});
 }
 final result={'descriptive_only':true,'rss_scope':'maximum sequential create/open/add subprocess peak; helper agent excluded','cases':output};
 File('$raw/summary.json').writeAsStringSync('${const JsonEncoder.withIndent('  ').convert(result)}\n',flush:true);
 print(green('Eight verified writes summarized; descriptive medians/ranges only.'));
}
