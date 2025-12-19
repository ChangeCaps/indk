{ pkgs ? import <nixpkgs> {
  config = {
    allowUnfree = true;
    android_sdk.accept_license = true;
  };
} }:

let
  androidComposition = pkgs.androidenv.composeAndroidPackages {
    buildToolsVersions = [ "35.0.0" "34.0.0" "33.0.1" ];
    platformVersions = [ "35" "34" "33" "31" "28" ];
    abiVersions = [ "armeabi-v7a" "arm64-v8a" ];
    ndkVersions = [ "27.0.12077973" ];
    includeNDK = true;
  };
  androidSdk = androidComposition.androidsdk;
in pkgs.mkShell rec {
  buildInputs = [
    pkgs.pkg-config
    pkgs.wayland
    pkgs.libxkbcommon
    pkgs.freetype
    pkgs.fontconfig
    pkgs.vulkan-loader
    pkgs.vulkan-validation-layers

    androidSdk
    pkgs.gradle
  ];

  ANDROID_SDK_ROOT = "${androidSdk}/libexec/android-sdk";
  GRADLE_OPTS = "-Dorg.gradle.project.android.aapt2FromMavenOverride=${androidSdk}/libexec/android-sdk/build-tools/35.0.0/aapt2";

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
