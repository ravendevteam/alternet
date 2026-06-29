{ pkgs, system, ... }:
let
	pname = "stellar-cli";
	version = "26.0.0";
	architecture.x86_64-linux.target = "x86_64-unknown-linux-gnu";
	architecture.x86_64-linux.sha256 = "sha256-Mcg9s0LRGEsx9lkec5lQ60V1a+RtzXu1fk846R6jLoQ=";
	architecture.x86_64-darwin.target = "x86_64-apple-darwin";
	architecture.x86_64-darwin.sha256 = "sha256-5v6qppaasR8T84XwiTfXhVs2OZuNqtPGd3knmt6hbsg=";
	architecture.aarch64-linux.target = "aarch64-unknown-linux-gnu";
	architecture.aarch64-linux.sha256 = "sha256-q/Wu5hii+ocgX871MrV/MhDzSB0S/j0pDFZnexio79Q=";
	architecture.aarch64-darwin.target = "x86_64-apple-darwin";
	architecture.aarch64-darwin.sha256 = "sha256-OO7oOWuxlCfenDbwfsOVtZzE2P6lupUaC51GPszzq6g=";
	compatible_architecture = architecture.${system} or (throw "(unsupported_system=${system})");
	src =
	let
		url = "https://github.com/stellar/stellar-cli/releases/download/v${version}/stellar-cli-${version}-${compatible_architecture.target}.tar.gz";
	in pkgs.fetchurl {
		inherit url;
		inherit (compatible_architecture) sha256;
	};
	nativeBuildInputs = [
		pkgs.nushell
	] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
		pkgs.autoPatchelfHook
	];
	buildInputs = [
		pkgs.stdenv.cc.cc.lib
	] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
		pkgs.dbus
		pkgs.systemd
		pkgs.openssl
	];
	dontUnpack = true;
	dontBuild = true;
	installPhase = ''
		nu -c '
			mkdir ($env.out | path join "bin")
			tar -xzf $env.src
			cp stellar ($env.out | path join "bin" "stellar")
			chmod +x ($env.out | path join "bin" "stellar")
		'
	'';
in pkgs.stdenv.mkDerivation {
	inherit pname;
	inherit version;
	inherit src;
	inherit nativeBuildInputs;
	inherit buildInputs;
	inherit dontUnpack;
	inherit dontBuild;
	inherit installPhase;
}