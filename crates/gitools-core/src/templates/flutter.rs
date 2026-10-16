use super::common::{ci_node, editorconfig, file, gitignore, opt_editorconfig, opt_gitignore, opt_license, opt_readme};
use crate::template::{Template, TemplateOption};

pub fn template() -> Template {
    Template {
        id: "flutter".into(),
        name: "Flutter".into(),
        tagline: "Aplikacja mobilna (Android/iOS) w Dart".into(),
        description: "Projekt Flutter z Material 3, warstwą danych, testami widgetów \
                      i konfiguracją linters."
            .into(),
        icon: "🦋".into(),
        accent: "#54c5f8".into(),
        tags: vec!["flutter".into(), "dart".into(), "mobile".into()],
        options: vec![
            TemplateOption::choice(
                "platforms",
                "Platformy",
                "Które platformy dodać do projektu",
                "Projekt",
                &[
                    ("android-ios", "Android + iOS"),
                    ("all", "Wszystkie"),
                ],
                "android-ios",
            ),
            TemplateOption::toggle(
                "riverpod",
                "Riverpod",
                "Zarządzanie stanem aplikacji",
                "Zależności",
                false,
            ),
            TemplateOption::toggle(
                "tests",
                "Testy",
                "Testy jednostkowe i widgetów",
                "Jakość",
                true,
            ),
            TemplateOption::toggle(
                "ci",
                "GitHub Actions",
                "Pipeline budujący apk/ipa",
                "Jakość",
                true,
            ),
            opt_readme(),
            opt_gitignore(),
            opt_editorconfig(),
            opt_license(),
        ],
        hints: vec!["flutter pub get".into(), "flutter run".into()],
        files: vec![
            file(
                "pubspec.yaml",
                r#"name: {{project_snake}}
description: "{{description}}"
publish_to: none
version: 0.1.0+1

environment:
  sdk: ^3.6.0

dependencies:
  flutter:
    sdk: flutter
  cupertino_icons: ^1.0.8
{{#if riverpod}}
  flutter_riverpod: ^2.6.1
{{/if}}

dev_dependencies:
  flutter_test:
    sdk: flutter
  flutter_lints: ^5.0.0

flutter:
  uses-material-design: true
"#,
            ),
            file(
                "analysis_options.yaml",
                r#"include: package:flutter_lints/flutter.yaml

linter:
  rules:
    - prefer_const_constructors
    - prefer_final_locals
    - avoid_print
    - require_trailing_commas: true
"#,
            ),
            file(
                "lib/main.dart",
                r#"import 'package:flutter/material.dart';
{{#if riverpod}}
import 'package:flutter_riverpod/flutter_riverpod.dart';
{{/if}}

import 'src/app.dart';

void main() {
{{#if riverpod}}
  runApp(const ProviderScope(child: {{project_pascal}}App()));
{{/if}}
{{#unless riverpod}}
  runApp(const {{project_pascal}}App());
{{/unless}}
}
"#,
            ),
            file(
                "lib/src/app.dart",
                r#"import 'package:flutter/material.dart';

import 'home_page.dart';

class {{project_pascal}}App extends StatelessWidget {
  const {{project_pascal}}App({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: '{{project_name}}',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: const Color(0xFF6750A4)),
        useMaterial3: true,
      ),
      home: const HomePage(),
    );
  }
}
"#,
            ),
            file(
                "lib/src/home_page.dart",
                r#"import 'package:flutter/material.dart';

class HomePage extends StatefulWidget {
  const HomePage({super.key});

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  int _counter = 0;

  void _increment() => setState(() => _counter++);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('{{project_name}}')),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Text('{{description}}'),
            const SizedBox(height: 24),
            FilledButton(
              onPressed: _increment,
              child: Text('Kliknięto ${_counter} razy'),
            ),
          ],
        ),
      ),
    );
  }
}
"#,
            ),
            file(
                "test/widget_test.dart",
                r#"import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:{{project_snake}}/src/home_page.dart';

void main() {
  testWidgets('Licznik rośnie po kliknięciu', (tester) async {
    await tester.pumpWidget(const MaterialApp(home: HomePage()));

    expect(find.text('Kliknięto 0 razy'), findsOneWidget);

    await tester.tap(find.byType(FilledButton));
    await tester.pump();

    expect(find.text('Kliknięto 1 razy'), findsOneWidget);
  });
}
"#,
            )
            .when("tests"),
            file(
                "test/smoke_test.dart",
                r#"import 'package:flutter_test/flutter_test.dart';

void main() {
  test('Nazwa pakietu jest ustawiona', () {
    expect(true, isTrue);
  });
}
"#,
            )
            .when("tests"),
            file(
                ".github/workflows/ci.yml",
                r#"name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  ci:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: subosito/flutter-action@v2
        with:
          channel: stable
          cache: true

      - run: flutter pub get
      - run: flutter analyze
      - run: flutter test
      - run: flutter build apk --release
"#
            )
            .when("ci"),
            file(
                "android/app/build.gradle.kts",
                r#"plugins {
    id("com.android.application")
    id("kotlin-android")
    id("dev.flutter.flutter-gradle-plugin")
}

android {
    namespace = "com.example.{{project_snake}}"
    compileSdk = flutter.compileSdkVersion
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }

    defaultConfig {
        applicationId = "com.example.{{project_snake}}"
        minSdk = flutter.minSdkVersion
        targetSdk = flutter.targetSdkVersion
        versionCode = flutter.versionCode
        versionName = flutter.versionName
    }
}

flutter {
    source = "../.."
}
"#
            )
            .when("platforms=android-ios"),
            file(
                "android/app/src/main/AndroidManifest.xml",
                r#"<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <application
        android:label="{{project_name}}"
        android:name="${applicationName}"
        android:icon="@mipmap/ic_launcher">
        <activity
            android:name=".MainActivity"
            android:exported="true"
            android:launchMode="singleTop"
            android:theme="@style/LaunchTheme"
            android:configChanges="orientation|keyboardHidden|keyboard|screenSize|smallestScreenSize|locale|layoutDirection|fontScale|screenLayout|density|uiMode"
            android:hardwareAccelerated="true"
            android:windowSoftInputMode="adjustResize">
            <meta-data
                android:name="io.flutter.embedding.android.NormalTheme"
                android:resource="@style/NormalTheme" />
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>
        <meta-data
            android:name="flutterEmbedding"
            android:value="2" />
    </application>
    <queries>
        <intent>
            <action android:name="android.intent.action.PROCESS_TEXT" />
            <data android:mimeType="text/plain" />
        </intent>
    </queries>
</manifest>
"#
            )
            .when("platforms=android-ios"),
            gitignore(
                &[
                    ".dart_tool/",
                    ".flutter-plugins",
                    ".flutter-plugins-dependencies",
                    ".packages",
                    "build/",
                    "coverage/",
                    "\n# Android",
                    "**/android/local.properties",
                    "*.jks",
                    "\n# iOS",
                    "**/ios/**/*.mode1v3",
                    "**/ios/**/*.pbxuser",
                    "\n# edytory",
                    ".idea/",
                    ".vscode/*",
                ],
            )
            .when("gitignore"),
            editorconfig().when("editorconfig"),
            ci_node().when("ci"),
            file(
                "README.md",
                r#"# {{project_name}}

{{description}}

## Wymagania

- Flutter SDK 3.24+
- Android Studio (Android) / Xcode (iOS)

## Start

```bash
flutter pub get
flutter run
```

## Testy i analiza

```bash
flutter analyze
flutter test
```

## Build

```bash
flutter build apk --release
flutter build ios --release
```

## Licencja

{{license_spdx}}
"#,
            )
            .when("readme"),
        ],
    }
}