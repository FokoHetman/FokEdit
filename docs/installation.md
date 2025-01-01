### FokEdit działa **tylko** na systemach typu GNU/Linux (ze względu na libc) oraz na WSL

# Metody instalacji:



* ## Pozostałe systemy Linux:
### Instalacja Cargo Rust:
> kompilowane za pomocą cargo `cargo 1.82.0 (8f40fc59f 2024-08-21)`
- Powołując się na oficjalną stronę [rust-lang](<https://www.rust-lang.org/learn/get-started>)
```curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh```
> Można również zainstalować za pomocą lokalnego menedżera pakietów, instalując pakiet `cargo`
### Instalacja FokEdit:

```
git clone https://github.com/fokohetman/FokEdit
cd FokEdit
cargo build             # lub `cargo run`, pomijając następną komendę
./target/debug/FokEdit
```


* ## Nix/NixOS:
> metoda zalecana, ze względu na 100% kompatybilność z ustawieniami autora.
* * Nix:
```nix
nix shell github:fokohetman/FokEdit
```
następnie, w nowostworzonym tymczasowym ENV:
```
FokEdit [*opcjonalnie: pliki do edycji]
```
ex. `FokEdit`, `FokEdit ~/.config/FokEdit/configuration.fok`, `FokEdit ~/Projects/`.
* * NixOS:
- jak każdy inny flake, tzn:
- - flake:
```nix
#flake.nix
inputs.fokedit.url = github:fokohetman/FokEdit;
```
```
#configuration.nix
environment.systemPackages = with pkgs; [
  inputs.fokedit.packages.${system}.default
];
```
