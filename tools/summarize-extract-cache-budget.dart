#! /usr/bin/env dcli
import 'dart:convert';
import 'dart:io';
import 'dart:math' as math;
import 'package:dcli/dcli.dart';
import 'summarize-adaptive-workers.dart' as stats;

const root='/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance/docs/evidence/native-extract-cache-budget-2026-10-11/raw';
const prior='/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance/docs/evidence/native-public-extraction-2026-10-11/raw';
const variants=['A-old','A-trial','B-old','B-trial'];
const worktree='/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance';
const expectedExecutables={
  'A-old':'$worktree/rust/target/native-public-20261011/bin/lockbox-A-default',
  'B-old':'$worktree/rust/target/native-public-20261011/bin/lockbox-B-native',
  'A-trial':'$worktree/rust/target/native-extract-cache-budget-20261011/bin/lockbox-A-trial',
  'B-trial':'$worktree/rust/target/native-extract-cache-budget-20261011/bin/lockbox-B-native-trial',
};
void require(bool condition,String message)=>stats.require(condition,message);
double n(dynamic value,String label,{bool positive=true})=>stats.number(value,label,positive:positive);

Map<String,dynamic> paired(String label,Map<int,double> candidate,Map<int,double> baseline) {
  final rounds=candidate.keys.toList()..sort();
  require(stats.same(rounds,baseline.keys.toList()..sort()),'Unpaired observations');
  final logs=[for(final r in rounds)math.log(candidate[r]!/baseline[r]!)];
  final balanced=rounds.map((r)=>r%2).toSet().length==2;
  final blocks=<double>[];
  if(balanced) {
    require(rounds.length==16,'Expected sixteen measured rounds');
    for(var i=0;i<rounds.length;i+=2) {
      require(rounds[i].isEven&&rounds[i+1]==rounds[i]+1,'Invalid adjacent blocks');
      blocks.add((logs[i]+logs[i+1])/2);
    }
  } else { require(rounds.length==8,'Expected eight stratum rounds'); blocks.addAll(logs); }
  final seed=stats.seedFor(label);
  final rng=stats.Seeded(seed);
  final boot=List<double>.generate(10000,(_) {
    var total=0.0;
    for(var i=0;i<blocks.length;i++)total+=blocks[rng.nextInt(blocks.length)];
    return math.exp(total/blocks.length);
  })..sort();
  return {'pairs':rounds.length,'rounds':rounds,'geometric_mean_ratio':math.exp(logs.reduce((a,b)=>a+b)/logs.length),
    'low95':stats.percentile(boot,0.025),'high95':stats.percentile(boot,0.975),'bootstrap_replicates':10000,
    'seed_xorshift32':seed,'method':balanced?'paired percentile bootstrap of eight adjacent balanced two-round log-ratio blocks':
      'within-round-stratum paired percentile bootstrap of eight round log ratios','linear_percentile_interpolation':true};
}

void main() {
  final output=File('$root/metrics-summary.json');
  require(!output.existsSync(),'Refusing existing summary');
  final manifest=stats.readObject('$root/identities-v2.json');
  final complete=stats.readObject('$root/batch-v2/complete.json');
  require(complete['accepted']==320&&complete['measured']==256&&complete['preparation']==64&&complete['identities_verified_after']==true,'Incomplete terminal evidence');
  final rows=File('$root/batch-v2/samples.jsonl').readAsLinesSync().where((s)=>s.isNotEmpty)
    .map((s)=>jsonDecode(s) as Map<String,dynamic>).toList();
  require(rows.length==320,'Incorrect process count');
  final specs=(jsonDecode(File('$prior/cases.json').readAsStringSync()) as List).cast<Map<String,dynamic>>();
  final seen=<String>{};
  var allRetain=true;
  final cases=<Map<String,dynamic>>[];
  for(var ci=0;ci<specs.length;ci++) {
    final spec=specs[ci],name=spec['name'];
    final caseRows=rows.where((r)=>r['case']==name).toList();
    require(caseRows.length==80,'Incorrect case count');
    final positions=<String,int>{};
    final groups=<String,List<Map<String,dynamic>>>{for(final v in variants)v:[]};
    for(final row in caseRows) {
      final variant=row['backend'] as String,round=row['round'] as int,position=row['position'] as int;
      require(round>=0&&round<20&&position>=0&&position<4,'Invalid coordinates');
      require(variant==variants[(position+round~/2+ci)%4]&&row['round_stratum']==round%2,'Wrong frozen order');
      require(seen.add('$name/$variant/$round')&&row['warmup']==(round<4),'Duplicate or incorrect prep flag');
      require(row['accepted']==true&&row['extraction_verified']==true&&row['metrics_valid']==true&&
        row['output_drained_and_verified']==true&&row['exit_code']==0&&row['timed_out']==false&&row['capture_incomplete']==false,'Rejected sample');
      require(row['output_bytes']==spec['bytes']&&row['actual_mode']==row['expected_default_mode'],'Output bytes/permissions mismatch');
      final expected=(spec['variants'] as List).singleWhere((v)=>v['variant']==variant[0]);
      require(row['output_sha256']==expected['source_sha256'],'Output differs from corpus');
      final archive='${expected['root']}/archive.lbox';
      require(row['archive']==archive&&row['archive_sha256']==manifest['hashes'][archive],'Archive identity mismatch');
      final command=row['command'] as List;
      require(command.take(4).join('|')=='/usr/bin/time|-v|timeout|--kill-after=15s'&&command[5]=='taskset'&&command[7]=='0-4'&&
        command[9]==archive&&command[10]=='extract'&&command[11]=='--to','Wrong public command scope');
      require(command[8]==expectedExecutables[variant]&&row['executable_sha256']==manifest['hashes'][command[8]],'Variant/executable mismatch');
      require(row['environment']['LOCKBOX_PLATFORM_SECRET_STORE']=='disabled','Ambient secret-store exposure');
      final gate=row['gate'] as Map,attempts=gate['attempts'] as List;
      require(gate['passed']==true&&attempts.isNotEmpty&&attempts.length<=3&&attempts.last['resident_bytes_before']==0,'Cold admission mismatch');
      final drain=row['postprocess_drain'] as Map;
      require(drain['passed']==true&&drain['outside_measurement']==true&&(drain['commands'] as List).length==2&&
        (drain['commands'] as List).every((c)=>c['passed']==true&&c['exit_code']==0),'Drain not performed');
      n(row['elapsed_seconds'],'elapsed');n(row['peak_rss_kib'],'rss');
      n(row['cpu']['user_seconds'],'user cpu',positive:false);n(row['cpu']['system_seconds'],'system cpu',positive:false);
      if(round>=4) {
        groups[variant]!.add(row);
        final key='$variant/${round%2}/$position';positions[key]=(positions[key]??0)+1;
      }
    }
    require(positions.length==32&&positions.values.every((count)=>count==2),'Position/round-stratum imbalance');
    final metrics=<String,dynamic>{};
    for(final e in groups.entries) {
      require(e.value.length==16,'Incomplete measured group');
      final elapsed=e.value.map((r)=>n(r['elapsed_seconds'],'elapsed')).toList();
      final rss=e.value.map((r)=>n(r['peak_rss_kib'],'rss')).toList();
      metrics[e.key]={'measured_samples':16,'median_elapsed_seconds':stats.median(elapsed),
        'median_mib_per_second':stats.median(elapsed.map((t)=>(spec['bytes'] as int)/1048576/t)),
        'median_cpu_seconds':stats.median(e.value.map((r)=>n(r['cpu']['user_seconds'],'user',positive:false)+n(r['cpu']['system_seconds'],'system',positive:false))),
        'peak_rss_kib_by_round':{for(final r in e.value)'${r['round']}':r['peak_rss_kib']},
        'maximum_peak_rss_kib':rss.reduce(math.max),'maximum_peak_rss_bytes':rss.reduce(math.max)*1024,
        'median_drain_seconds':stats.median(e.value.map((r)=>n(r['postprocess_drain']['seconds'],'drain',positive:false)))};
    }
    final comparisons=<String,dynamic>{};
    for(final layout in ['A','B']) {
      Map<int,double> times(String suffix,int? stratum)=>{for(final r in groups['$layout-$suffix']!)
        if(stratum==null||r['round']%2==stratum)r['round'] as int:n(r['elapsed_seconds'],'elapsed')};
      final aggregate=paired('$name/$layout/all',times('trial',null),times('old',null));
      final strata={for(var p=0;p<2;p++)'$p':paired('$name/$layout/round-stratum$p',times('trial',p),times('old',p))};
      final under100=metrics['$layout-trial']['maximum_peak_rss_bytes']<100000000;
      final noninferior=aggregate['high95']<=1.05;
      final rejectSlow=aggregate['low95']>1.05||strata.values.any((s)=>s['low95']>1.05);
      final retain=under100&&noninferior&&!rejectSlow;
      allRetain=allRetain&&retain;
      comparisons[layout]={'elapsed_trial_over_old':aggregate,'round_strata':strata,
        'all_measured_trial_processes_below_100000000_rss_bytes':under100,'aggregate_upper95_at_most_1_05':noninferior,
        'stratum_noninferiority_not_established':[for(final e in strata.entries)if(e.value['high95']>1.05)e.key],
        'supported_slowdown_above_5_percent':rejectSlow,'retain_by_frozen_rule':retain,
        'maximum_rss_reduction_fraction':1-metrics['$layout-trial']['maximum_peak_rss_bytes']/metrics['$layout-old']['maximum_peak_rss_bytes']};
    }
    cases.add({'case':name,'payload_bytes':spec['bytes'],'metrics':metrics,'comparisons':comparisons});
  }
  output.writeAsStringSync(const JsonEncoder.withIndent('  ').convert({'accepted':320,'measured':256,'preparation':64,
    'all_fixed_matrix_identity_output_admission_checks_passed':true,'retain_policy_by_frozen_rule':allRetain,
    'rss_limit_bytes':100000000,'elapsed_ratio_upper95_limit':1.05,'cases':cases,
    'identities_sha256':stats.sha('$root/identities-v2.json'),'rows_sha256':stats.sha('$root/batch-v2/samples.jsonl'),
    'summarizer_sha256':stats.sha(Platform.script.toFilePath()),
    'statistics_helper_sha256':stats.sha('${File(Platform.script.toFilePath()).parent.path}/summarize-adaptive-workers.dart'),
    'limitations':'single-file public ExtractMany, plaintext, isolated synthetic corpora; no actual CLI reader-TID/physical-byte gate, no multiplicity correction, no many-file/password/ReadMostly qualification'}),flush:true);
  print(green('Audited320 processes; retain policy by frozen rule: $allRetain; ${output.path}'));
}
