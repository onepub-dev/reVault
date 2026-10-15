#! /usr/bin/env dcli
import 'dart:convert';
import 'dart:io';
import 'dart:math' as math;
import 'package:dcli/dcli.dart';
import 'summarize-adaptive-workers.dart' as stats;

const root = '/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance/docs/evidence/native-public-extraction-2026-10-11/raw';
const variants = ['A', 'B', 'C1', 'Cauto', 'ZIP'];
const pairs = [['A','ZIP'],['B','ZIP'],['C1','ZIP'],['Cauto','ZIP'],['B','A'],['Cauto','C1']];
void require(bool value, String message) => stats.require(value, message);
List<Map<String,dynamic>> rows(String path) => File(path).readAsLinesSync().where((v)=>v.trim().isNotEmpty)
    .map((v)=>jsonDecode(v) as Map<String,dynamic>).toList();
double number(dynamic value, String label, {bool positive=true}) => stats.number(value,label,positive:positive);
Map<String,dynamic> object(dynamic value,String label) => stats.object(value,label);

stats.Observation observation(Map<String,dynamic> row) {
  final sample=object(row['sample'],'sample');
  final io=object(sample['storage_io'],'io');
  final cpu=object(sample['resources'],'resources')['cpu'] as Map;
  final payload=sample['logical_bytes_written'] as int;
  final elapsed=number(sample['total_seconds'],'elapsed');
  final deviceReads=<String,int>{for(final device in sample['block_device_io']['devices'] as List)
    device['device'] as String:device['delta']['read_sectors'] as int};
  return stats.Observation(row['round'] as int,elapsed,
      number(cpu['user_seconds'],'user',positive:false)+number(cpu['system_seconds'],'system',positive:false),
      payload/1048576/elapsed,number(sample['resources']['peak_rss_kib'],'rss'),
      number(row['whole_process_peak_rss_kib'],'process rss'),io['read_bytes'] as int,
      io['write_bytes'] as int,io['cancelled_write_bytes'] as int,payload,
      number(row['postprocess_drain']['seconds'],'drain',positive:false),deviceReads);
}

void main() {
  final output=File('$root/metrics-summary.json');
  require(!output.existsSync(),'Refusing existing summary');
  final component=rows('$root/component-batch-v1/samples.jsonl');
  final cli=rows('$root/public-cli-batch-v1/samples.jsonl');
  final identities=stats.readObject('$root/pre-timing-identities.json');
  final specs=(jsonDecode(File('$root/cases.json').readAsStringSync()) as List).cast<Map<String,dynamic>>();
  require(component.length==480 && cli.length==64,'Incomplete planned sample counts');
  require(stats.readObject('$root/component-batch-v1/complete.json')['accepted']==480 &&
      stats.readObject('$root/public-cli-batch-v1/complete.json')['accepted']==64,'Missing successful batch terminal evidence');
  final seen=<String>{};
  final results=<Map<String,dynamic>>[];
  for(var caseIndex=0;caseIndex<specs.length;caseIndex++) {
    final spec=specs[caseIndex];
    final name=spec['name'] as String;
    final selected=component.where((r)=>r['case']==name).toList();
    require(selected.length==120,'Wrong case size $name');
    final byVariant=<String,Map<int,stats.Observation>>{for(final v in variants)v:{}};
    final positions=<String,int>{};
    for(final row in selected) {
      final variant=row['backend'] as String;
      final round=row['round'] as int;
      final position=row['position'] as int;
      require(round>=0 && round<24 && position>=0 && position<5,'Invalid coordinates');
      require(variants[(position+round+caseIndex)%5]==variant,'Wrong fixed execution order');
      require(seen.add('$name/$variant/$round'),'Duplicate sample');
      require(row['warmup']==(round<4) && row['parity']==round%2,'Invalid preparation/parity');
      require(row['accepted']==true && row['exit_code']==0 && row['capture_incomplete']==false &&
          row['outer_runner_timeout']==false && row['metrics_valid']==true && row['extraction_verified']==true &&
          row['output_drained_and_verified']==true && row['physical_read_gate_passed']==true,'Failed sample admitted');
      final drain=object(row['postprocess_drain'],'drain');
      require(drain['passed']==true && drain['outside_measurement']==true && (drain['commands'] as List).length==2 &&
          (drain['commands'] as List).every((r)=>r['passed']==true && r['exit_code']==0),'Missing actual drain');
      final sample=object(row['sample'],'endpoint');
      final reader=object(sample['reader_identity'],'reader');
      final tid=reader['reader_tid'] as int;
      require(tid>0 && tid%2==round%2 && reader['reader_tid_parity']==round%2 &&
          stats.same(reader['namespace_tids'],[tid]) && reader['pid_namespace']=='pid:[4026531836]' &&
          sample['requested_reader_tid_parity']==round%2,'Reader identity mismatch');
      require(sample['verified']==true && sample['files']==1 && sample['logical_bytes_written']==spec['bytes'] &&
          row['output_bytes']==spec['bytes'] && sample['file_syncs']==0 && sample['directory_syncs']==0,'Endpoint scope mismatch');
      final archive=(identities['archive_inventory'] as List).singleWhere((r)=>r['case']==name && r['variant']==(variant.startsWith('C')?'C':variant));
      require(row['archive_path']==archive['path'] && row['archive_sha256']==archive['sha256'] && row['archive_bytes']==archive['bytes'],'Archive identity mismatch');
      final expectedHash=(spec['variants'] as List).first['source_sha256'];
      require(row['output_sha256']==expectedHash,'Output differs from pinned corpus');
      require(row['read_bytes']>=row['required_read_bytes'] && row['required_read_bytes']>0,'Missing physical read admission');
      final attempts=row['admission_attempts'] as List;
      require(attempts.isNotEmpty && attempts.length<=16 && attempts.last['accepted']==true && attempts.last['gate']['passed']==true,'Invalid admission trace');
      final command=row['command'] as List;
      require(command.take(4).join('|')=='/usr/bin/time|-v|timeout|--kill-after=15s' && command[5]=='taskset' && command[7]=='0-4','Wrong process controls');
      final binary=(identities['binaries'] as Map).values.cast<Map>().singleWhere((b)=>b['path']==command[8]);
      require(binary['sha256'] is String,'Unknown executable');
      if(round>=4) {
        byVariant[variant]![round]=observation(row);
        final key='$variant/${round%2}/$position'; positions[key]=(positions[key]??0)+1;
      }
    }
    require(positions.length==50 && positions.values.every((n)=>n==2),'Position/parity balance failed');
    final comparisons=<String,dynamic>{};
    for(final pair in pairs) {
      final label='${pair[0]}/${pair[1]}';
      final aggregate=stats.paired('$name/$label/all',byVariant[pair[0]]!,byVariant[pair[1]]!,(r)=>r.elapsed);
      aggregate['method']='paired percentile bootstrap of ten balanced adjacent two-round blocks; linear percentile interpolation';
      final parity=<String,dynamic>{for(var p=0;p<2;p++) '$p':stats.paired('$name/$label/parity$p',
        Map.fromEntries(byVariant[pair[0]]!.entries.where((e)=>e.key%2==p)),
        Map.fromEntries(byVariant[pair[1]]!.entries.where((e)=>e.key%2==p)),(r)=>r.elapsed)};
      comparisons[label]={'aggregate':aggregate,'parity':parity,
        'useful_by_frozen_rule':aggregate['high95']<1 && parity.values.every((v)=>v['high95']<1)};
    }
    final public=<String,dynamic>{};
    for(final variant in ['A','B']) {
      final selectedCli=cli.where((r)=>r['case']==name && r['backend']==variant).toList();
      require(selectedCli.length==8,'Public CLI count mismatch');
      final rounds=<int>{};
      for(final row in selectedCli) {
        final round=row['round'] as int;
        require(rounds.add(round) && round>=0 && round<8 && row['warmup']==(round<2),'CLI round mismatch');
        require(stats.same(row['order'],round.isEven?['A','B']:['B','A']),'CLI order mismatch');
        require(row['accepted']==true && row['metrics_valid']==true && row['extraction_verified']==true &&
          row['output_drained_and_verified']==true && row['output_bytes']==spec['bytes'] &&
          row['actual_mode']==row['expected_default_mode'] && row['gate']['passed']==true &&
          row['capture_incomplete']==false && row['outer_runner_timeout']==false && row['exit_code']==0,'CLI admission failure');
        require(row['output_sha256']==(spec['variants'] as List).first['source_sha256'],'CLI output hash mismatch');
      }
      final measured=selectedCli.where((r)=>r['warmup']==false).toList();
      public[variant]={'measured':6,'median_elapsed_seconds':stats.median(measured.map((r)=>number(r['elapsed_seconds'],'CLI elapsed'))),
        'max_peak_rss_kib':measured.map((r)=>number(r['peak_rss_kib'],'CLI rss')).reduce(math.max),
        'median_cpu_seconds':stats.median(measured.map((r)=>number(r['cpu']['user_seconds'],'user',positive:false)+number(r['cpu']['system_seconds'],'system',positive:false))),
        'scope':'descriptive whole CLI process; ExtractMany policy; not component parity/physical-I/O qualification'};
    }
    results.add({'case':name,'bytes':spec['bytes'],'component_metrics':{for(final v in variants)v:stats.metrics(byVariant[v]!.values.toList())},
      'paired_elapsed_comparisons':comparisons,'public_cli':public});
  }
  output.writeAsStringSync(const JsonEncoder.withIndent('  ').convert({'component_accepted':480,'component_measured':400,
    'public_cli_accepted':64,'public_cli_measured':48,'pre_timing_manifest_sha256':stats.sha('$root/pre-timing-identities.json'),
    'component_rows_sha256':stats.sha('$root/component-batch-v1/samples.jsonl'),'cli_rows_sha256':stats.sha('$root/public-cli-batch-v1/samples.jsonl'),
    'summarizer_sha256':stats.sha(Platform.script.toFilePath()),'statistical_helper_sha256':stats.sha('${File(Platform.script.toFilePath()).parent.path}/summarize-adaptive-workers.dart'),
    'bootstrap_replicates':10000,'no_multiplicity_correction':true,'all_exact_counts_order_parity_and_admission_verified':true,'cases':results}),flush:true);
  print(green('Verified fixed matrices and wrote ${output.path}'));
}
