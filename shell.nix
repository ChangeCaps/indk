{ pkgs ? import <nixpkgs> {
  config = {
    allowUnfree = true;
    android_sdk.accept_license = true;
  };
} }:

let
  androidComposition = pkgs.androidenv.composeAndroidPackages {
    buildToolsVersions = [ "36.0.0" "35.0.0" "34.0.0" "33.0.1" ];
    platformVersions = [ "36" "35" "34" "33" "31" "28" ];
    abiVersions = [ "armeabi-v7a" "arm64-v8a" ];
    ndkVersions = [ "27.3.13750724" ];
    includeNDK = true;
  };
  androidSdk = androidComposition.androidsdk;
in pkgs.mkShell rec {
  buildInputs = [
    pkgs.pkg-config
    pkgs.gtk4
    pkgs.librsvg
    pkgs.freetype
    pkgs.fontconfig
    pkgs.openssl

    androidSdk
    pkgs.gradle_9
    pkgs.jdk17
  ];

  ANDROID_SDK_ROOT = "${androidSdk}/libexec/android-sdk";
  GRADLE_OPTS = "-Dorg.gradle.project.android.aapt2FromMavenOverride=${androidSdk}/libexec/android-sdk/build-tools/36.0.0/aapt2";

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
