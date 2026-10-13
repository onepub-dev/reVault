#! /usr/bin/env dcli
import 'dart:io';
import 'package:dcli/dcli.dart';

const root='/home/bsutton/git/.codex.workspaces/revault-issue-310-zip-read-performance';
const ev='$root/docs/evidence/security-review-2026-10-13/raw/full-runtime-api-recheck';
void check(bool value,String message){if(!value)throw StateError(message);}
void main(){
 final expected=File('$ev/api-default.list').readAsLinesSync().where((line)=>line.endsWith(': test')).map((line)=>line.substring(0,line.length-6)).toSet();
 final raw=File('$ev/stdout.raw').readAsLinesSync();
 final results=<String,String>{};
 final re=RegExp(r'^\s*test\s+(.+?)\s+\.\.\.\s+(ok|FAILED|ignored)(?:,\s*.*)?\s*$');
 final summaryRe=RegExp(r'^test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;');
 List<int>? summary;
 for(final line in raw){
  final normalized=line.replaceAll(RegExp(r'\x1B\[[0-9;]*[A-Za-z]'),'').replaceAll('\r','').trimRight();
  final match=re.firstMatch(normalized);
  if(match!=null){check(!results.containsKey(match.group(1)!),'duplicate result ${match.group(1)}');results[match.group(1)!]=match.group(2)!;}
  final sm=summaryRe.firstMatch(normalized);
  if(sm!=null)summary=[for(var i=1;i<=5;i++)int.parse(sm.group(i)!)];
 }
 final missing=expected.difference(results.keys.toSet());
 final unexpected=results.keys.toSet().difference(expected);
 final passed=results.values.where((v)=>v=='ok').length;
 final failed=results.values.where((v)=>v=='FAILED').length;
 final ignored=results.values.where((v)=>v=='ignored').length;
 check(missing.isEmpty&&unexpected.isEmpty,'name mismatch: missing=${missing.length} unexpected=${unexpected.length}');
 check(summary!=null&&summary![0]==passed&&summary![1]==failed&&summary![2]==ignored&&summary![3]==0&&summary![4]==0,'libtest summary mismatch');
 print(green('parser replay PASS tests=${results.length}/${expected.length} passed=$passed failed=$failed ignored=$ignored summary=$summary'));
}
