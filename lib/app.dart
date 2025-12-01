import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:farmcare/config.dart';
import 'package:farmcare/src/rust/api/healthz.dart';

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(home: const HealthCheckPage());
  }
}

class HealthCheckPage extends StatefulWidget {
  const HealthCheckPage({super.key});

  @override
  State<HealthCheckPage> createState() => _HealthCheckPageState();
}

class _HealthCheckPageState extends State<HealthCheckPage> {
  Client? _client;
  HealthResponse? _response;
  String? _error;
  bool _loading = false;

  @override
  void dispose() {
    _client?.close();
    super.dispose();
  }

  Future<void> _connect() async {
    setState(() {
      _loading = true;
      _error = null;
    });

    try {
      final certPem = utf8.decode(base64.decode(Config.cert));
      _client = await Client.connect(
        serverAddr: Config.serverAddr,
        serverName: Config.serverName,
        certPem: certPem,
      );
      setState(() => _loading = false);
    } catch (e) {
      setState(() {
        _error = e.toString();
        _loading = false;
      });
    }
  }

  Future<void> _healthz() async {
    if (_client == null) {
      await _connect();
      if (_client == null) return;
    }

    setState(() {
      _loading = true;
      _error = null;
    });

    try {
      final response = await _client!.healthz();
      setState(() {
        _response = response;
        _loading = false;
      });
    } catch (e) {
      setState(() {
        _error = e.toString();
        _loading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Health Check')),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            if (_loading)
              const Center(child: CircularProgressIndicator())
            else if (_error != null)
              Card(
                color: Colors.red.shade100,
                child: Padding(
                  padding: const EdgeInsets.all(16.0),
                  child: Text(
                    'Error: $_error',
                    style: const TextStyle(color: Colors.red),
                  ),
                ),
              )
            else if (_response != null)
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16.0),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'Status: ${_response!.status}',
                        style: const TextStyle(fontSize: 18),
                      ),
                      const SizedBox(height: 8),
                      Text(
                        'Version: ${_response!.version}',
                        style: const TextStyle(fontSize: 18),
                      ),
                    ],
                  ),
                ),
              )
            else
              const Center(child: Text('Press the button to check health')),
            const SizedBox(height: 24),
            ElevatedButton(
              onPressed: _loading ? null : _healthz,
              child: const Text('Check Health'),
            ),
          ],
        ),
      ),
    );
  }
}
