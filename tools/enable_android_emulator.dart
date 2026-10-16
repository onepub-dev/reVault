#! /usr/bin/env dcli

import 'dart:io';
import 'package:dcli/dcli.dart';

// Bounded CI runner setup; invoke once with sudo, never change sudo policy.
void main() {
  Shell.current.releasePrivileges();
  if (!Platform.isLinux)
    throw UnsupportedError('Android CI KVM setup requires Linux');
  try {
    Shell.current.withPrivileges(() {
      File('/etc/udev/rules.d/99-kvm4all.rules').writeAsStringSync(
        'KERNEL=="kvm", GROUP="kvm", MODE="0666", OPTIONS+="static_node=kvm"\n',
      );
      for (final arguments in [
        ['control', '--reload-rules'],
        ['trigger', '--name-match=kvm'],
      ]) {
        final result = Process.runSync('udevadm', arguments);
        if (result.exitCode != 0)
          throw StateError('udevadm failed: ${result.stderr}');
      }
    });
  } finally {
    Shell.current.releasePrivileges();
  }
}
