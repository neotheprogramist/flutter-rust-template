import 'package:farmcare/app.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('HealthCheckPage displays initial state', (
    WidgetTester tester,
  ) async {
    await tester.pumpWidget(const MaterialApp(home: HealthCheckPage()));

    expect(find.text('Health Check'), findsOneWidget);
    expect(find.text('Press the button to check health'), findsOneWidget);
    expect(find.byType(ElevatedButton), findsOneWidget);
    expect(find.text('Check Health'), findsOneWidget);
  });

  testWidgets('HealthCheckPage shows error when config is missing', (
    WidgetTester tester,
  ) async {
    await tester.pumpWidget(const MaterialApp(home: HealthCheckPage()));

    await tester.tap(find.byType(ElevatedButton));
    await tester.pumpAndSettle();

    expect(find.textContaining('Error:'), findsOneWidget);
  });
}
