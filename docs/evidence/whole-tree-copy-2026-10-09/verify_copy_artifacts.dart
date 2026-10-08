#! /usr/bin/env dcli
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';
import 'package:crypto/crypto.dart';
import 'package:dcli/dcli.dart';

Stream<List<int>> lengthBound(File file, int length) async* {
  final bytes=ByteData(8)..setUint64(0,length,Endian.little);
  yield bytes.buffer.asUint8List();
  yield* file.openRead();
}
Future<void> main(List<String> args) async {
  if(args.length!=1) throw ArgumentError('completed evidence directory required');
  final directory=Directory(args.single);
  final logs=directory.listSync().whereType<File>().where((f)=>f.path.endsWith('.log'));
  String? transcript;
  for(final log in logs){final text=log.readAsStringSync();if(text.contains('WHOLE_TREE_COPY_FUNCTIONAL {')&&text.contains('test result: ok. 1 passed')){if(transcript!=null) throw StateError('multiple aggregate logs');transcript=text;}}
  if(transcript==null) throw StateError('no completed successful aggregate transcript');
  final rows=<Map<String,dynamic>>[];
  for(final line in const LineSplitter().convert(transcript)){
    const marker='WHOLE_TREE_COPY_FUNCTIONAL ';
    final index=line.indexOf(marker);if(index<0) continue;
    rows.add(jsonDecode(line.substring(index+marker.length)) as Map<String,dynamic>);
  }
  final modes=rows.map((r)=>r['mode'] as int).toSet();
  if(rows.length!=16||modes.length!=16||!List.generate(16,(i)=>i).every(modes.contains)) throw StateError('incomplete or duplicate modes');
  final artifacts=<Map<String,dynamic>>[];
  for(final row in rows){
    for(final role in ['source','destination']){
      final file=File(row[role] as String);
      if(!file.path.startsWith('/tmp/revault-whole-tree-copy-functional-')) throw StateError('unexpected artifact path');
      final length=await file.length();if(length!=row['${role}_bytes']) throw StateError('artifact size changed');
      final bound=await sha256.bind(lengthBound(file,length)).first;
      if(bound.bytes.join(',')!=(row['${role}_length_bound_sha256'] as List).join(',')) throw StateError('artifact length-bound digest changed');
      final raw=await sha256.bind(file.openRead()).first;
      artifacts.add({'mode':row['mode'],'role':role,'path':file.path,'bytes':length,'sha256':raw.toString(),'length_bound_sha256':bound.toString()});
    }
  }
  final output={'verified_modes':16,'large_logical_bytes_each':10485760,'scope':'Independent size and digest verification of retained synthetic FileStore images; no timing or physical-disk durability claim.','artifacts':artifacts};
  File('${directory.path}/verified-artifacts.json').writeAsStringSync('${const JsonEncoder.withIndent('  ').convert(output)}\n');
  print(green('Verified all16 unique modes and32 retained images against test-reported sizes and length-bound digests.'));
}
