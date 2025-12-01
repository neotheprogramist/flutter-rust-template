set shell := ["bash", "-euo", "pipefail", "-c"]
set dotenv-load := true

default:
    @just --list

_dart-defines:
    #!/usr/bin/env zsh
    for v in ${(k)parameters[(I)CLIENT_*]}; do [ -n "${(P)v:-}" ] && echo -n " --dart-define=$v=${(P)v}"; done

_android-id:
    @flutter devices --machine 2>/dev/null | jq -r '.[] | select(.targetPlatform | startswith("android")) | select(.emulator) | .id' | head -1 || true

_ios-id:
    @flutter devices --machine 2>/dev/null | jq -r '.[] | select(.targetPlatform == "ios" and .emulator) | .id' | head -1 || true

build-apk mode="debug": frb-generate
    flutter build apk --{{ mode }} $(just _dart-defines)

build-appbundle mode="debug": frb-generate
    flutter build appbundle --{{ mode }} $(just _dart-defines)

build-ios mode="debug": frb-generate
    flutter build ios --{{ mode }} $(just _dart-defines)

build-macos mode="debug": frb-generate
    flutter build macos --{{ mode }} $(just _dart-defines)

build-web mode="debug": frb-generate
    flutter build web --{{ mode }} $(just _dart-defines)

setup-certs:
    #!/usr/bin/env zsh
    CERT="${CERT_PATH:-./certs/cert.pem}"; KEY="${KEY_PATH:-./certs/key.pem}"
    [ ! -f "$CERT" ] || [ ! -f "$KEY" ] && echo "Certs not found" >&2 && exit 1
    [ ! -f .env ] && echo ".env not found" >&2 && exit 1
    C=$(base64 < "$CERT" | tr -d '\n'); K=$(base64 < "$KEY" | tr -d '\n')
    sed -i '' "s|^SERVER_CERT_PEM_B64=.*|SERVER_CERT_PEM_B64=$C|;s|^SERVER_KEY_PEM_B64=.*|SERVER_KEY_PEM_B64=$K|;s|^CLIENT_CERT_PEM_B64=.*|CLIENT_CERT_PEM_B64=$C|" .env

frb-generate:
    flutter_rust_bridge_codegen generate

clean:
    cargo clean && flutter clean && flutter pub get

install:
    flutter pub get

upgrade:
    cargo upgrade && flutter pub upgrade

analyze: rust-clippy dart-analyze

dart-analyze:
    flutter analyze

dart-format-check:
    dart format --set-exit-if-changed .

dart-format-fix:
    dart format .

format-check: just-format-check rust-format-check dart-format-check prettier-format-check

just-format-check:
    just --fmt --unstable --check

just-format-fix:
    just --fmt --unstable

format-fix: just-format-fix rust-format-fix dart-format-fix prettier-format-fix

lint: format-fix analyze

prettier-format-check:
    npx -y prettier --check "**/*.{json,yaml,yml,md,toml}"

prettier-format-fix:
    npx -y prettier --write "**/*.{json,yaml,yml,md,toml}"

rust-clippy:
    cargo clippy --all-targets --all-features -- -D warnings

rust-clippy-fix:
    cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

rust-format-check:
    cargo +nightly fmt --all -- --check

rust-format-fix:
    cargo +nightly fmt --all

run: frb-generate
    flutter run $(just _dart-defines)

run-rust name="server":
    cargo run --bin {{ name }}

run-rust-example name="healthz":
    cargo run --example {{ name }}

run-android: frb-generate
    #!/usr/bin/env zsh
    D=$(just _android-id); [ -z "$D" ] && echo "No Android emulator" >&2 && exit 1
    flutter run -d "$D" $(just _dart-defines)

run-chrome: frb-generate
    flutter run -d chrome $(just _dart-defines)

run-ios: frb-generate
    #!/usr/bin/env zsh
    D=$(just _ios-id); [ -z "$D" ] && echo "No iOS simulator" >&2 && exit 1
    flutter run -d "$D" $(just _dart-defines)

run-macos: frb-generate
    flutter run -d macos $(just _dart-defines)

test-all: test-rust test-flutter-unit test-flutter-integration

test-flutter-unit:
    flutter test

_run-integration-test file:
    #!/usr/bin/env zsh
    D=$(just _ios-id); [ -z "$D" ] && echo "No simulator" >&2 && exit 1
    [ -z "${SERVER_CERT_PEM_B64:-}" ] || [ -z "${SERVER_KEY_PEM_B64:-}" ] && echo "Run 'just setup-certs'" >&2 && exit 1
    N=$(basename "{{ file }}" .dart); LOG="/tmp/test-$N.log"
    cargo run -p server --release > "$LOG" 2>&1 & PID=$!
    trap 'kill $PID 2>/dev/null; wait $PID 2>/dev/null' EXIT
    for i in {1..60}; do cargo run --example healthz >/dev/null 2>&1 && break; kill -0 $PID 2>/dev/null || { cat "$LOG" >&2; exit 1; }; sleep 1; done
    cargo run --example healthz >/dev/null 2>&1 || { cat "$LOG" >&2; exit 1; }
    flutter test "{{ file }}" -d "$D" $(just _dart-defines)

test-flutter-integration:
    #!/usr/bin/env zsh
    D=$(just _ios-id); [ -z "$D" ] && echo "No simulator" >&2 && exit 1
    [ -z "${SERVER_CERT_PEM_B64:-}" ] || [ -z "${SERVER_KEY_PEM_B64:-}" ] && echo "Run 'just setup-certs'" >&2 && exit 1
    P=0; F=0
    for T in integration_test/*_test.dart; do
        echo "Running $(basename $T .dart)..."
        if just _run-integration-test "$T"; then ((P++)); else ((F++)); fi
    done
    echo "Results: $P passed, $F failed"
    if [ $F -gt 0 ]; then exit 1; fi

test-flutter-integration-file name:
    just _run-integration-test "integration_test/{{ name }}_test.dart"

test-flutter: test-flutter-unit test-flutter-integration

test-rust:
    cargo test
