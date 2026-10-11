#! /usr/bin/env dcli
import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';
import 'package:dcli/dcli.dart';

const worktree='/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance';
const prior='$worktree/docs/evidence/native-public-extraction-2026-10-11/raw';
const evidenceRoot='$worktree/docs/evidence/native-extract-cache-budget-2026-10-11/raw';
const ownedRoot='/home/bsutton/.cache/revault-native-public-20261011';
const outputRoot='$ownedRoot/cache-budget-trial-v2';
const oldBin='$worktree/rust/target/native-public-20261011/bin';
const trialBin='$worktree/rust/target/native-extract-cache-budget-20261011/bin';
const variants=['A-old','A-trial','B-old','B-trial'];
const artifactCap=24*1024*1024*1024;
const unitBytes=65536;
const commands=<String,String>{'A-old':'$oldBin/lockbox-A-default','A-trial':'$trialBin/lockbox-A-trial',
  'B-old':'$oldBin/lockbox-B-native','B-trial':'$trialBin/lockbox-B-native-trial'};

Future<void> main(List<String> args) async {
  if(args.length!=1) throw ArgumentError('preflight | freeze | batch | cleanup');
  if(args.single=='preflight') { await preflight(); return; }
  if(args.single=='cleanup') { await cleanupOwnedAgents(); return; }
  if(args.single=='freeze') { freeze(); return; }
  if(args.single=='batch') {
    try { await batch(); } catch(error) { stderr.writeln('Batch failure before cleanup: $error'); rethrow; }
    finally { await cleanupOwnedAgents(); }
    return;
  }
  throw ArgumentError('preflight | freeze | batch | cleanup');
}

List<Map<String,dynamic>> loadCases() => (jsonDecode(File('$prior/cases.json').readAsStringSync()) as List).cast<Map<String,dynamic>>();
Map<String,dynamic> readObject(String path) => jsonDecode(File(path).readAsStringSync()) as Map<String,dynamic>;
void require(bool value,String message) { if(!value) throw StateError(message); }
String archivePath(Map<String,dynamic> spec,String variant) => '${(spec['variants'] as List).singleWhere((r)=>r['variant']==variant[0])['root']}/archive.lbox';
String sourcePath(Map<String,dynamic> spec,String variant) => '${(spec['variants'] as List).singleWhere((r)=>r['variant']==variant[0])['root']}/source/file-000000.bin';

void freeze() {
  require(!File('$evidenceRoot/identities-v2.json').existsSync(),'Refusing existing identities');
  final preflightRecord=readObject('$evidenceRoot/agent-preflight-v2.json');
  require(preflightRecord['complete']==true && (preflightRecord['rows'] as List).length==16 &&
      (preflightRecord['rows'] as List).every((r)=>r['passed']==true), 'Actual agent roundtrip preflight incomplete');
  final baseline=readObject('$prior/pre-timing-identities.json');
  final built=readObject('$evidenceRoot/build/build-manifest.json');
  require(readObject('$prior/component-batch-v1/complete.json')['accepted']==480 &&
      readObject('$prior/public-cli-batch-v1/complete.json')['accepted']==64,'Baseline incomplete');
  final identities=<String,String>{};
  for(final relative in (baseline['source_hashes'] as Map).keys) identities['$worktree/$relative']=_sha('$worktree/$relative');
  for(final path in ['$prior/pre-timing-identities.json','$prior/cases.json','$prior/metrics-summary.json',
    '$evidenceRoot/build/build-manifest.json','$evidenceRoot/agent-preflight-v2.json','$evidenceRoot/protocol-before-timing-v2.md',Platform.script.toFilePath()]) identities[path]=_sha(path);
  // The executor manifest pins exactly the source files compiled into these binaries.
  final expectedSource={
    'rust/revault_lockbox_api/src/lockbox.rs':'52ecacb67160a094a2bd78d8d7ea4c2d151f4a89d62e787d8611d7cba687cea1',
    'rust/revault_lockbox_api/src/lockbox/files.rs':'cb64f5a065938318392795705fd7a3798b1a70ce72f9da0dc23d4606e744a5d9',
    'rust/revault_lockbox_api/src/storage/cache_options.rs':'39874f2eb215b25356f9712d266f5a395a29f68a09ec63614a1eb4cb7992ce16',
    'rust/revault_lockbox_api/src/storage/page_cache.rs':'79e96581dbfbc33f1c84a360b19360e9cdfad2ddb4469a0d6b2b150184be5061',
  };
  for(final e in (baseline['source_hashes'] as Map).entries) {
    if(!expectedSource.containsKey(e.key)) require(identities['$worktree/${e.key}']==e.value,'Unreviewed source change: ${e.key}');
  }
  for(final e in expectedSource.entries) require(identities['$worktree/${e.key}']==e.value,'Compiled source epoch changed');
  final expectedTrial={
    'A-trial':'14e4bc70f4fd9aba0caf4a945f7c1dca7e1990b3a3fc65fe5e0103a5493d9d92',
    'B-trial':'0eade5d185cf996c08a3ddc7a17dda242e26c055360dc0045da9c58e8965b6d4',
  };
  for(final e in commands.entries) {
    final hash=_sha(e.value);
    if(e.key.endsWith('-old')) {
      final name=e.value.substring(e.value.lastIndexOf('/')+1);
      require(hash==baseline['binaries'][name]['sha256'],'Baseline executable changed');
    } else { require(hash==expectedTrial[e.key],'Trial executable differs from release build'); }
    identities[e.value]=hash;
  }
  for(final spec in loadCases()) {
    for(final variant in ['A-old','B-old']) {
      final archive=archivePath(spec,variant);
      final source=sourcePath(spec,variant);
      final old=(baseline['archive_inventory'] as List).singleWhere((r)=>r['case']==spec['name'] && r['variant']==variant[0]);
      final sourceHash=(spec['variants'] as List).singleWhere((r)=>r['variant']==variant[0])['source_sha256'];
      require(_sha(archive)==old['sha256'] && File(archive).lengthSync()==old['bytes'],'Archive changed');
      require(_sha(source)==sourceHash && File(source).lengthSync()==spec['bytes'],'Source changed');
      identities[archive]=old['sha256']; identities[source]=sourceHash;
    }
  }
  final diff=Process.runSync('git',['diff','HEAD','--','rust'],workingDirectory:worktree);
  require(diff.exitCode==0,'Cannot preserve source diff');
  File('$evidenceRoot/tested-source-v2.diff').writeAsStringSync(diff.stdout as String,flush:true);
  identities['$evidenceRoot/tested-source-v2.diff']=_sha('$evidenceRoot/tested-source-v2.diff');
  _writeJson('$evidenceRoot/identities-v2.json',{'hashes':identities,'build_manifest':built,
    'baseline_manifest_sha256':_sha('$prior/pre-timing-identities.json'),
    'frozen_utc':DateTime.now().toUtc().toIso8601String(),'planned_processes':320,'planned_measured':256});
  print(green('Frozen trial source, old/new release binaries, protocol and unchanged fixtures.'));
}

Map<String,dynamic> verifyIdentities() {
  final manifest=readObject('$evidenceRoot/identities-v2.json');
  for(final e in (manifest['hashes'] as Map).entries) require(_sha(e.key as String)==e.value,'Identity changed: ${e.key}');
  return manifest;
}

void checkSpace(int nextOutput) {
  final owned=_apparentBytes(Directory(ownedRoot))+_apparentBytes(Directory(oldBin))+
    _apparentBytes(Directory(trialBin))+_apparentBytes(Directory(evidenceRoot))+_apparentBytes(Directory(prior));
  require(owned+nextOutput<=artifactCap && _availableBytes()>=nextOutput+2*1024*1024*1024,'Artifact cap/free-space headroom');
}

Future<void> batch() async {
  final manifest=verifyIdentities();
  final target=Directory('$evidenceRoot/batch-v2');
  require(!target.existsSync(),'Refusing existing batch');
  target.createSync(recursive:true);
  final specs=loadCases();
  final wall=Stopwatch()..start();
  var accepted=0;
  for(var caseIndex=0;caseIndex<specs.length;caseIndex++) {
    final spec=specs[caseIndex];
    for(var round=0;round<20;round++) {
      for(var position=0;position<4;position++) {
        final variant=variants[(position+round~/2+caseIndex)%4];
        final archive=archivePath(spec,variant);
        final source=File(sourcePath(spec,variant));
        final id='${spec['name']}-$variant-r${round.toString().padLeft(2,'0')}-p$position';
        final private='$outputRoot/${spec['name']}/$variant';
        Directory(private).createSync(recursive:true);
        final env=trialEnvironment(caseIndex,variant);
        final destination='$private/output-$id';
        require(FileSystemEntity.typeSync(destination,followLinks:false)==FileSystemEntityType.notFound,'Output already exists');
        checkSpace(spec['bytes'] as int);
        final expectedMode=_defaultFileMode(private);
        final gate=await _boundedColdGate(archive,wall,const Duration(minutes:60));
        require(gate['passed']==true,'Archive cold admission failed');
        final seconds=_remainingCommandSeconds(wall,const Duration(minutes:60));
        final command=['/usr/bin/time','-v','timeout','--kill-after=15s','$seconds','taskset','-c','0-4',
          commands[variant]!,archive,'extract','--to',destination];
        final capture=await _capture(command.first,command.skip(1).toList(),env,seconds+20);
        final log='${target.path}/$id';
        File('$log.stdout').writeAsStringSync(capture.stdout,flush:true);
        File('$log.stderr').writeAsStringSync(capture.stderr,flush:true);
        final actual=File('$destination/file-000000.bin');
        final regular=FileSystemEntity.typeSync(actual.path,followLinks:false)==FileSystemEntityType.file;
        final mode=regular?actual.statSync().mode&0x1ff:null;
        final verified=capture.code==0 && !capture.timedOut && !capture.incomplete &&
          _exactOutputTree(destination) && regular && actual.lengthSync()==spec['bytes'] &&
          mode==expectedMode && _filesEqual(source,actual);
        final outputHash=regular?_sha(actual.path):null;
        final elapsed=_timeElapsed(capture.stderr),rss=_timeMaximumRss(capture.stderr);
        final cpu=_timeCpu(capture.stderr);
        final metricsValid=_positive(elapsed)&&_positive(rss)&&cpu!=null;
        final drain=verified&&metricsValid?await _drainOutput(destination,wall,const Duration(minutes:60)):
          <String,dynamic>{'passed':false,'not_attempted':true};
        final passed=verified&&metricsValid&&drain['passed']==true&&outputHash==(manifest['hashes'] as Map)[source.path];
        final row={'case':spec['name'],'backend':variant,'round':round,'position':position,'round_stratum':round%2,
          'warmup':round<4,'sample_id':id,'command':command,'environment':env,'gate':gate,
          'archive':archive,'archive_sha256':(manifest['hashes'] as Map)[archive],
          'executable_sha256':(manifest['hashes'] as Map)[commands[variant]],
          'elapsed_seconds':elapsed,'cpu':cpu,'peak_rss_kib':rss,'metrics_valid':metricsValid,
          'output_sha256':outputHash,'output_bytes':regular?actual.lengthSync():null,
          'actual_mode':mode,'expected_default_mode':expectedMode,'extraction_verified':verified,
          'postprocess_drain':drain,'output_drained_and_verified':verified&&drain['passed']==true,
          'exit_code':capture.code,'timed_out':capture.timedOut,'capture_incomplete':capture.incomplete,'accepted':passed};
        _appendJsonl('${target.path}/samples.jsonl',row);
        require(passed,'Trial sample failed; output/logs retained: $id');
        accepted++;
        Directory(destination).deleteSync(recursive:true);
        _writeJson('${target.path}/progress.json',{'accepted':accepted,'case':spec['name'],'round':round,
          'position':position,'wall_seconds':wall.elapsed.inMilliseconds/1000});
      }
    }
  }
  verifyIdentities();
  require(accepted==320,'Incomplete trial matrix');
  _writeJson('${target.path}/complete.json',{'accepted':accepted,'measured':256,'preparation':64,
    'identities_verified_after':true,'wall_seconds':wall.elapsed.inMilliseconds/1000});
  print(green('Completed320 accepted public CLI cache-budget samples (256 measured,64 prep).'));
}

class Capture {
  const Capture(this.code, this.stdout, this.stderr, {this.timedOut = false, this.incomplete = false});
  final int code;
  final String stdout;
  final String stderr;
  final bool timedOut;
  final bool incomplete;
}

Future<Capture> _capture(String executable, List<String> args, Map<String, String> env, int timeoutSeconds) async {
  final process = await Process.start(executable, args, environment: env, includeParentEnvironment: false);
  final stdoutBuffer = StringBuffer();
  final stderrBuffer = StringBuffer();
  var streamFailed = false;
  final stdoutDone = Completer<void>();
  final stderrDone = Completer<void>();
  final stdoutSubscription = process.stdout.transform(utf8.decoder).listen(
    stdoutBuffer.write, onError: (Object error, StackTrace stack) {
      streamFailed = true;
      stderrBuffer.write('\n[stdout stream error: $error]');
      if (!stdoutDone.isCompleted) stdoutDone.complete();
    }, onDone: () { if (!stdoutDone.isCompleted) stdoutDone.complete(); });
  final stderrSubscription = process.stderr.transform(utf8.decoder).listen(
    stderrBuffer.write, onError: (Object error, StackTrace stack) {
      streamFailed = true;
      stderrBuffer.write('\n[stderr stream error: $error]');
      if (!stderrDone.isCompleted) stderrDone.complete();
    }, onDone: () { if (!stderrDone.isCompleted) stderrDone.complete(); });
  var code = -1;
  var timedOut = false;
  try {
    code = await process.exitCode.timeout(Duration(seconds: timeoutSeconds));
  } on TimeoutException {
    timedOut = true;
    process.kill(ProcessSignal.sigterm);
    try { code = await process.exitCode.timeout(const Duration(seconds: 10)); }
    on TimeoutException {
      process.kill(ProcessSignal.sigkill);
      try { code = await process.exitCode.timeout(const Duration(seconds: 10)); }
      on TimeoutException { code = -1; }
    }
  }
  var incomplete = false;
  try {
    await Future.wait([stdoutDone.future, stderrDone.future]).timeout(const Duration(seconds: 10));
  } on TimeoutException {
    incomplete = true;
    await stdoutSubscription.cancel();
    await stderrSubscription.cancel();
    stdoutBuffer.write('\n[stdout capture incomplete after bounded process timeout]');
    stderrBuffer.write('\n[stderr capture incomplete after bounded process timeout]');
  }
  return Capture(code, stdoutBuffer.toString(), stderrBuffer.toString(), timedOut: timedOut, incomplete: incomplete || streamFailed);
}

bool _filesEqual(File expected, File actual) {
  final left = expected.openSync();
  final right = actual.openSync();
  final a = Uint8List(unitBytes), b = Uint8List(unitBytes);
  try {
    while (true) {
      final an = left.readIntoSync(a);
      final bn = right.readIntoSync(b);
      if (an != bn) return false;
      if (an == 0) return true;
      for (var index = 0; index < an; index++) {
        if (a[index] != b[index]) return false;
      }
    }
  } finally {
    left.closeSync();
    right.closeSync();
  }
}

String _sha(String path) {
  final result = Process.runSync('sha256sum', [path]);
  if (result.exitCode != 0) throw StateError('sha256sum failed for $path: ${result.stderr}');
  return '${result.stdout}'.split(RegExp(r'\s+')).first;
}

void _writeJson(String path, Object value) {
  final file = File(path);
  file.parent.createSync(recursive: true);
  file.writeAsStringSync(const JsonEncoder.withIndent('  ').convert(value), flush: true);
}

void _appendJsonl(String path, Object value) {
  File(path).parent.createSync(recursive: true);
  File(path).writeAsStringSync('${jsonEncode(value)}\n', mode: FileMode.append, flush: true);
}

Map<String, String> _privateEnv(String root, String home) {
  final paths = <String, String>{
    'HOME': home,
    'XDG_CONFIG_HOME': '$root/private-config',
    'XDG_CACHE_HOME': '$root/private-cache',
    'XDG_STATE_HOME': '$root/private-state',
    'XDG_DATA_HOME': '$root/private-data',
    'LOCKBOX_SESSION_AGENT_DIR': '$root/private-agent',
    'LOCKBOX_VAULT_DIR': '$root/private-vault',
    'LOCKBOX_PLATFORM_SECRET_STORE': 'disabled',
    'TMPDIR': '/home/bsutton/.cache/rv4-security-20261011',
    'PATH': '/usr/bin:/bin',
  };
  for (final directory in paths.entries.where((entry) => !['PATH', 'LOCKBOX_PLATFORM_SECRET_STORE', 'TMPDIR'].contains(entry.key))) {
    Directory(directory.value).createSync(recursive: true);
  }
  Directory('$home/.revault').createSync(recursive: true);
  Directory(paths['LOCKBOX_VAULT_DIR']!).createSync(recursive: true);
  return paths;
}

Future<Capture> _boundedUtility(List<String> command, Stopwatch wall, Duration cap) async {
  final seconds = _remainingCommandSeconds(wall, cap, maximum: 30);
  return _capture('/usr/bin/timeout', ['--kill-after=15s', '$seconds', ...command],
      {'PATH': '/usr/bin:/bin', 'TMPDIR': '/home/bsutton/.cache/rv4-security-20261011'}, seconds + 20);
}

Future<Map<String, dynamic>> _boundedColdGate(String archive, Stopwatch wall, Duration cap) async {
  final attempts = <Map<String, dynamic>>[];
  for (var attempt = 0; attempt < 3; attempt++) {
    final captures = <Capture>[];
    final commands = <List<String>>[
      ['/usr/bin/sync', '-d', '--', archive],
      ['/usr/bin/fadvise', '--advice', 'dontneed', archive],
      ['/usr/bin/fincore', '--json', '--bytes', '--output', 'SIZE,RES,DIRTY,WRITEBACK,FILE', archive],
    ];
    for (final command in commands) {
      final capture = await _boundedUtility(command, wall, cap);
      captures.add(capture);
      if (capture.timedOut || capture.incomplete || capture.code != 0) break;
    }
    dynamic parsed;
    try { if (captures.length == 3) parsed = jsonDecode(captures.last.stdout); } catch (_) {}
    final rows = parsed is Map ? parsed['fincore'] : parsed;
    final raw = rows is List && rows.length == 1 && rows.single is Map ? (rows.single as Map)['res'] : null;
    final resident = raw is num ? raw.toInt() : int.tryParse('$raw');
    final passed = captures.length == 3 && captures.every((r) => r.code == 0 && !r.incomplete && !r.timedOut) && resident == 0;
    attempts.add({'attempt': attempt, 'commands': commands, 'results': [for (final r in captures)
      {'exit_code': r.code, 'stdout': r.stdout, 'stderr': r.stderr, 'timed_out': r.timedOut, 'incomplete': r.incomplete}],
      'resident_bytes_before': resident, 'passed': passed});
    if (passed) return {'passed': true, 'attempts': attempts};
  }
  _appendJsonl('$evidenceRoot/cold-admission-failures.jsonl', {'archive': archive, 'attempts': attempts});
  return {'passed': false, 'attempts': attempts};
}

Future<Map<String, dynamic>> _drainOutput(String destination, Stopwatch wall, Duration cap) async {
  final started = Stopwatch()..start();
  final commands = <List<String>>[
    ['/usr/bin/sync', '-d', '--', '$destination/file-000000.bin'],
    ['/usr/bin/sync', '-d', '--', destination, Directory(destination).parent.path],
  ];
  final results = <Map<String, dynamic>>[];
  for (final command in commands) {
    final result = await _boundedUtility(command, wall, cap);
    final passed = result.code == 0 && !result.timedOut && !result.incomplete;
    results.add({'command': command, 'exit_code': result.code, 'stdout': result.stdout, 'stderr': result.stderr,
      'timed_out': result.timedOut, 'incomplete': result.incomplete, 'passed': passed});
    if (!passed) return {'passed': false, 'commands': results, 'seconds': started.elapsed.inMicroseconds / 1e6};
  }
  return {'passed': true, 'commands': results, 'seconds': started.elapsed.inMicroseconds / 1e6, 'outside_measurement': true};
}

int _remainingCommandSeconds(Stopwatch wall, Duration cap, {int maximum = 180}) {
  // Reserve child TERM/KILL grace, outer kill fallback and stream draining.
  final available = cap.inSeconds - wall.elapsed.inSeconds - 50;
  if (available < 1) throw StateError('Batch deadline reached; partial evidence retained');
  return available < maximum ? available : maximum;
}

bool _exactOutputTree(String destination) {
  if (FileSystemEntity.typeSync(destination, followLinks: false) != FileSystemEntityType.directory) return false;
  final children = Directory(destination).listSync(followLinks: false);
  return children.length == 1 && children.single.path == '$destination/file-000000.bin' &&
      FileSystemEntity.typeSync(children.single.path, followLinks: false) == FileSystemEntityType.file;
}

int _defaultFileMode(String root) {
  final probe = File('$root/.default-mode-${DateTime.now().microsecondsSinceEpoch}');
  probe.writeAsStringSync('');
  final mode = probe.statSync().mode & 0x1ff;
  probe.deleteSync();
  return mode;
}

Map<String, double>? _timeCpu(String stderr) {
  double? field(String name) {
    final match = RegExp('$name \\(seconds\\):\\s*([0-9.]+)').firstMatch(stderr);
    return match == null ? null : double.tryParse(match.group(1)!);
  }
  final user = field('User time');
  final system = field('System time');
  return _nonnegative(user) && _nonnegative(system) ? {'user_seconds': user!, 'system_seconds': system!} : null;
}

double? _timeElapsed(String stderr) {
  final match = RegExp(r'Elapsed \(wall clock\) time \(h:mm:ss or m:ss\):\s*([^\n]+)').firstMatch(stderr);
  if (match == null) return null;
  final parts = match.group(1)!.trim().split(':').map(double.parse).toList();
  return parts.length == 3 ? parts[0] * 3600 + parts[1] * 60 + parts[2] : parts[0] * 60 + parts[1];
}

int? _timeMaximumRss(String stderr) {
  final match = RegExp(r'Maximum resident set size \(kbytes\):\s*(\d+)').firstMatch(stderr);
  return match == null ? null : int.parse(match.group(1)!);
}

bool _positive(dynamic value) => value is num && value.isFinite && value > 0;

bool _nonnegative(dynamic value) => value is num && value.isFinite && value >= 0;

int _apparentBytes(Directory directory) {
  if (!directory.existsSync()) return 0;
  var total = 0;
  for (final entity in directory.listSync(followLinks: false)) {
    final type = FileSystemEntity.typeSync(entity.path, followLinks: false);
    if (type == FileSystemEntityType.file) {
      total += File(entity.path).lengthSync();
    } else if (type == FileSystemEntityType.directory) {
      total += _apparentBytes(Directory(entity.path));
    }
  }
  return total;
}

int _availableBytes() {
  final result = Process.runSync('df', ['-Pk', '/home/bsutton/.cache']);
  if (result.exitCode != 0) throw StateError('df failed: ${result.stderr}');
  final parts = '${result.stdout}'.trim().split('\n').last.trim().split(RegExp(r'\s+'));
  return int.parse(parts[3]) * 1024;
}

Map<String,String> trialEnvironment(int caseIndex,String variant) {
  final vi=variants.indexOf(variant);
  require(vi>=0,'Unknown variant environment');
  final private='$ownedRoot/e2/c${caseIndex}v$vi';
  final env=_privateEnv(private,'$private/home');
  env['LOCKBOX_SESSION_AGENT_LOG']='$private/agent.log';
  final socket='${env['LOCKBOX_SESSION_AGENT_DIR']}/agent.sock';
  require(utf8.encode(socket).length<=100,'Agent socket path exceeds conservative Unix limit');
  return env;
}

Future<void> preflight() async {
  final path='$evidenceRoot/agent-preflight-v2.json';
  require(!File(path).existsSync(),'Refusing existing preflight evidence');
  final records=<Map<String,dynamic>>[];
  for(var ci=0;ci<4;ci++) for(final variant in variants) {
    final env=trialEnvironment(ci,variant);
    final socket='${env['LOCKBOX_SESSION_AGENT_DIR']}/agent.sock';
    final command=['/usr/bin/timeout','--kill-after=5s','20',commands[variant]!,'session','--format','json'];
    final result=await _capture(command.first,command.skip(1).toList(),env,30);
    final responses=<dynamic>[];
    try { for(final line in result.stdout.split('\n').where((v)=>v.trim().isNotEmpty)) responses.add(jsonDecode(line)); } catch(_) {}
    final running=responses.any((r)=>r is Map&&r['kind']=='agent'&&r['state']=='running'&&r['value']=='yes');
    final socketCheck=await _capture('/usr/bin/test',['-S',socket],env,5);
    final passed=result.code==0&&!result.timedOut&&!result.incomplete&&running&&socketCheck.code==0&&!socketCheck.incomplete&&!socketCheck.timedOut;
    records.add({'case_index':ci,'variant':variant,'environment':env,'command':command,'exit_code':result.code,
      'stdout':result.stdout,'stderr':result.stderr,'timed_out':result.timedOut,'capture_incomplete':result.incomplete,
      'agent_running_reported':running,'owned_socket_path':socket,'socket_path_bytes':utf8.encode(socket).length,
      'socket_type_check_command':['/usr/bin/test','-S',socket],'socket_type_check_exit':socketCheck.code,'passed':passed});
    _writeJson(path,{'complete':false,'rows':records});
    require(passed,'Actual public agent roundtrip preflight failed; evidence retained');
  }
  _writeJson(path,{'complete':true,'rows':records,'registered_activity_command':'public session (Vault activity), followed by compatible-agent status and list protocol roundtrip'});
  print(green('All16 isolated agent contexts passed public roundtrip and actual socket preflight.'));
}

Future<void> cleanupOwnedAgents() async {
  final preflightPath='$evidenceRoot/agent-preflight-v2.json';
  if(!File(preflightPath).existsSync())return;
  final results=<Map<String,dynamic>>[];
  for(final row in readObject(preflightPath)['rows'] as List) {
    final env=Map<String,String>.from(row['environment'] as Map);
    final socket=row['owned_socket_path'] as String;
    require(socket.startsWith('$ownedRoot/e2/')&&utf8.encode(socket).length<=100,'Refusing non-owned agent cleanup');
    final socketCheck=await _capture('/usr/bin/test',['-S',socket],env,5);
    if(socketCheck.code==1) { results.add({'socket':socket,'already_absent':true,'passed':true}); continue; }
    final variant=row['variant'] as String;
    final command=['/usr/bin/timeout','--kill-after=5s','20',commands[variant]!,'session','stop'];
    final result=await _capture(command.first,command.skip(1).toList(),env,30);
    var absent=false;
    for(var attempt=0;attempt<20;attempt++) {
      final check=await _capture('/usr/bin/test',['-S',socket],env,5);
      if(check.code==1) { absent=true; break; }
      await Future<void>.delayed(const Duration(milliseconds:100));
    }
    results.add({'socket':socket,'command':command,'exit_code':result.code,'stdout':result.stdout,'stderr':result.stderr,
      'socket_absent_after':absent,'passed':result.code==0&&!result.timedOut&&!result.incomplete&&absent});
  }
  final passed=results.every((r)=>r['passed']==true);
  _writeJson('$evidenceRoot/agent-cleanup-v2-${DateTime.now().microsecondsSinceEpoch}.json',{'passed':passed,'agents':results});
  require(passed,'Owned-agent cleanup incomplete; scoped evidence retained');
}
