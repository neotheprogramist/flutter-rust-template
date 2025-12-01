import 'package:flutter/material.dart';

import 'package:farmcare/app.dart';
import 'package:farmcare/src/rust/frb_generated.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(const MyApp());
}
