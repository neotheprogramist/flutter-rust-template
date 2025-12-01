class Config {
  static final String serverAddr = _env(
    const String.fromEnvironment('CLIENT_SERVER_ADDR'),
    'CLIENT_SERVER_ADDR',
  );
  static final String serverName = _env(
    const String.fromEnvironment('CLIENT_SERVER_NAME'),
    'CLIENT_SERVER_NAME',
  );
  static final String cert = _env(
    const String.fromEnvironment('CLIENT_CERT_PEM_B64'),
    'CLIENT_CERT_PEM_B64',
  );

  static String _env(String value, String name) {
    if (value.isEmpty) throw StateError('$name not set');
    return value;
  }
}
