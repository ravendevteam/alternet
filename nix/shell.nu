$env.PATH = ($env.PATH | prepend ($env.PWD | path join ".local" "bin"))
$env.PATH = ($env.PATH | prepend ($env.HOME | path join ".cargo" "bin"))

try {
    rustup target add wasm32-unknown-unknown
}

# --- Onboarding Banner ---
print -e $"
(ansi green_bold)Welcome to the dev environment!(ansi reset)

(ansi yellow_bold)🛠️ Task Runner:(ansi reset)
  The (ansi cyan_bold)task(ansi reset) binary is available in your environment.
  Run (ansi cyan_bold)task(ansi reset) to list all available helper commands for this repository.

(ansi yellow_bold)❄️ Nix Usage Tip:(ansi reset)
  • This dev shell isolates dependencies without modifying your global system.
  • Run (ansi cyan_bold)nix flake check(ansi reset) to run all project checks.
  • Run (ansi cyan_bold)nix build .#<package>(ansi reset) to build a specific target.
  • If inputs change, update them with (ansi cyan_bold)nix flake update(ansi reset).
"
