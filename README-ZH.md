This script replaces Flutter's bundled Dart SDK with the macOS arm64 version

Getting Started
Set up Flutter as normal and run flutter doctor

Install Dart form homebrew:
//https://github.com/Rexios80/flutter_m1_patcher
$ brew tap dart-lang/dart
$ brew install dart
This script nukes Flutter's bundled Dart SDK, so trying to run this script with Flutter's bundled Dart SDK will fail

Use as an executable
Installation
$ dart pub global activate flutter_m1_patcher
Usage
Run flutterpatch in a terminal

Run with -p to specify the Flutter root path

Additional Information
If things go bad, delete flutter/bin/cache and run flutter doctor. This will reset the bundled Dart SDK to the one shipped with Flutter.

You will need to run flutterpatch after every flutter upgrade