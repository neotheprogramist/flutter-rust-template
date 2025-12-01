import 'package:farmcare/app.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:farmcare/src/rust/frb_generated.dart';
import 'package:integration_test/integration_test.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async => await RustLib.init());

  testWidgets('Health check displays response', (WidgetTester tester) async {
    await tester.pumpWidget(const MyApp());

    expect(find.text('Health Check'), findsOneWidget);
    expect(find.text('Press the button to check health'), findsOneWidget);

    await tester.tap(find.byType(ElevatedButton));
    await tester.pump();

    expect(find.byType(CircularProgressIndicator), findsOneWidget);

    await tester.pumpAndSettle();

    expect(find.textContaining('Status:'), findsOneWidget);
    expect(find.textContaining('Version:'), findsOneWidget);
  });
}
