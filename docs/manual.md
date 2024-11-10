# Edytor:
edytor tekstu - jak wcześniej stwierdzono - inspirowany jest vim'em.
Stąd i system "stanu" edytora. W domyślnej konfiguracji (o tym więcej później) - w prawym dolnym rogu, najbliższa rogu cyfru reprezentuje stan edytora.
Są 4 stany:
* Control - kursor znajduje się na tekście, nie można edytować tekstu.
* Input - kursor znajduje się na tekście, można edytować tekst.
* Command - kursor znajduje się na linii IO w dolnym rogu okna terminalu.
* Select - kursor jest w stanie zaznaczenia tekstu.

Aby przejść w stan:
* Control - należy nacisnąć 2x `esc` w dowolnym innym stanie.
* Input - [zależne od konfiguracji] w stanie Control należy nacisnąć `a` lub `i`.
* Command - w stanie Control należy nacisnąć `:`.
* Select - [zależne od konfiguracji] w dowolnym stanie należy nacisnąć kombinację `shift+strzałka`.


# konfiguracja:
Po pierwszym uruchomieniu programu stworzy się ścieżka `~/.config/FokEdit/` wraz z presetami oraz domyślnym plikiem konfiguracyjnym.
* `~/.config/FokEdit/presets.fok` - kilka przykładowych stylów edytora, do zaimportowania i użytku.
* `~/.config/FokEdit/configuration.fok` - główny plik konfiguracji.

Schemat głównego pliku konfiguracji wygląda mniej więcej tak:
```fok
{
  theme = {};
  elements = {};
  ops = {};
  keybinds = [];
}
```
Omawiając po kolei:
* theme - odpowiada za kolory edytora, może być ustawiony jako jeden z presetów - `presets.*`, domyślna wartość to `presets.minimal`.
* elements - szczegóły edytora
* * debug - wartości debugowania
* * * cursor - pokazywanie pozycji kursora w prawym dolnym rogu, domyślna wartość to `true`.
* * empty_line
* * * text - tekst w pustych linijkach, domyślna wartość to `"~"`.
* ops - opcje edytora
* * line_numbers - numerowanie linijek
* * * enable - czy numerowanie jest włączone, domyślna wartość to `false`.
* keybinds - lista keybindów edytora.


## Keybindy
Reprezentuje je taka struktura:
```fok
{
  key = "ctrl_left";                       #! kombinacja/keybind
  action = "mb (0 - 1)";                   #! akcja (w języku foklang) która wydarzy się po naciśnięciu kombinacji
  override = true;                         #! czy keybind powinien zatrzymać normalne działanie klawiszy (w tym przypadku lewej strzałki)
  states = [states.control states.select]; #!  lista stanów w którym keybind obowiązuje
}
```

DO DOKOŃCZENIA.
