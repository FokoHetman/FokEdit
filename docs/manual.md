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


# Konfiguracja:
Po pierwszym uruchomieniu programu stworzy się ścieżka `~/.config/FokEdit/` wraz z presetami oraz domyślnym plikiem konfiguracyjnym.
* `~/.config/FokEdit/presets.fok` - kilka przykładowych stylów edytora, do zaimportowania i użytku.
* `~/.config/FokEdit/configuration.fok` - główny plik konfiguracji.

Schemat głównego pliku konfiguracji wygląda mniej więcej tak:
```fok
{
  theme = {};
  elements = {};
  ops = {};
  foklang = {};
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
* foklang - ustawienia języka embed'owanego (Foklang-FokEdit)
* * persistence - zachowywanie zmiennych
* * rc - RC programu
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

Domyślna konfiguracja ma zdefiniowane następujące keybindy:

shift_[up/left/down/right] - wejdź w tryb zaznaczenia.
control_[left/right] - zmień buffer (1 w lewo/prawo)



można ją również zobaczyć tutaj:

```
presets = load_file "/home/foko/.config/FokEdit/presets.fok";
{
  theme = presets.minimal;
  ops = {
    tab_size = 4;
    line_numbers = {
      enable = false;
    };
  };
  elements = {
    empty_line = {
      text = "~";
    };
    debug = {
      cursor = true;
    };
  };
  foklang = {
    persistence = true;
    rc = 
"
cy = program.cursor.y; #! basically, a shortcut
"; #! `rc` is code that is ran right at the start, meaning it defines some values etc. Technically speaking this whole config is `rc`.
  };
  keybinds = [ 
    {
      key = "ctrl_left";                        #! ctrl_left --> ctrl + left_arrow combination #! due to budget you are not able to do stuff like ctrl_shift_left etc.
      action = "mb (0-1)";                      #! foklang command, look at fokedit+foklang documentation for reference
      override = true;                          #! override default left_arrow action
      states = [states.control states.select];  #! states in which the keybind is valid (ex. don't move buffers with this keybind in `input` and `command` state)
    }
    {
      key = "ctrl_right";
      action = "mb 1";
      override = true;
      states = [states.control states.select];
    }
    {key="shift_right"; action="select";override=false;states=states.all;}
    {key="shift_left"; action="select";override=false;states=states.all;}
    {key="shift_up"; action="select";override=false;states=states.all;}
    {key="shift_down"; action="select";override=false;states=states.all;}
  ];

}
```






Lista wbudowanych komend FokLang-FokEdit:

* `[q/quit/exit]` - wyjście z programu                                              #! ex. `q`
* `[w/write] [[opcjonalnie] nazwa pliku: String]` - zapisz plik                     #! ex. `w`, `w test.txt`
* `[mb/movebuf] [ilość: i32]` - przemieszczanie się między bufferami                #! ex. `mb 1`, `mb 0-1`
* `[b/setbuf] [buffer: i32]` - zmień aktywny buffer na argument                     #! ex. `b 3`, `b 1`, `b 0`
* `[o/open] [nazwa pliku: String]` - otwórz plik w nowym bufferze                   #! ex. `o "configuration.fok"`, `o "/home/foko/Projects/test.txt"`
* `[load_fokedit] [konfiguracja: {}]` - załaduj konfigurację z argumentu            #! ex. `load_fokedit {theme = presets.gruvbox;}`
* `[program]` - zbiór kilku zmiennych (aktualnie jedynie cursor)                    #! ex. `program.cursor.y`


Foklang pozwala na ciekawe kombinacje operacji:
```
c = program.cursor.y
```
następnie można przemieszczać kursor relatywnie do aktualnej pozycji, tzn. `c+5`, `c-5`, et cetera.


