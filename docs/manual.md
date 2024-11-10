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
* `~/.config/FokEdit/themes.fok` - kilka przykładowych stylów edytora, do zaimportowania i użytku.
* `~/.config/FokEdit/configuration.fok` - główny plik konfiguracji.




DO DOKOŃCZENIA.
