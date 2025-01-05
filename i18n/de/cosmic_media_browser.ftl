cosmic-media-browser = COSMIC Medien Betrachter
empty-folder = Leerer Ordner
empty-folder-hidden = Leerer Ordner (mit versteckten Dateien)
no-results = No results found
filesystem = Dateisystem
home = Home
networks = Netzwerke
notification-in-progress = Dateioperation werden ausgeführt.
trash = Papierkorb
recents = Zuletzt
undo = Zurück
today = Heute

# List view
name = Name
modified = Geändert
trashed-on = Gelöscht
size = Größe

# Progress footer
details = Details
dismiss = Dismiss message
operations-running = {$running} operations running ({$percent}%)...
operations-running-finished = {$running} operations running ({$percent}%), {$finished} finished...
pause = Pause
resume = Resume

# Dialogs

## Compress Dialog
create-archive = Create archive

## Empty Trash Dialog
empty-trash = Papierkorb leeren
empty-trash-warning = Sind Sie sicher, dass Sie den alle Dateien im Papierkorb löschen wollen?

## Mount Error Dialog
mount-error = Kann auf Laufwerk nicht zugreifen

## New File/Folder Dialog
create-new-file = Neue Datei
create-new-folder = Neuen Ordner
file-name = Dateiname
folder-name = Verzeichnisname
file-already-exists = A file with that name already exists.
folder-already-exists = A folder with that name already exists.
name-hidden = Names starting with "." will be hidden.
name-invalid = Name cannot be "{$filename}".
name-no-slashes = Name cannot contain slashes.
recursive-scan-directories = Durchsuche alle Unterverzeichnisse nach Medien

## Open/Save Dialog
cancel = Abbrechen
create = Erzeugen
open = Öffnen
open-file = Öffne Datei
open-folder = Öffne Ordner
open-in-new-tab = Open in new tab
open-in-new-window = Open in new window
open-item-location = Open item location
open-multiple-files = Open multiple files
open-multiple-folders = Open multiple folders
save = Speichern
save-file = Datei speichern

## Open With Dialog
open-with-title = How do you want to open "{$name}"?
browse-store = Browse {$store}

## Rename Dialog
rename-file = Datei umbenennen
rename-folder = Ordner umbenennen

## Replace Dialog
replace = Ersetzen
replace-title = {$filename} existiert schon.
replace-warning = Wollen Sie sie ersetzen? Das wird ihren Inhalt überschreiben.
replace-warning-operation = Wollen Sie sie ersetzen? Das wird ihren Inhalt überschreiben.
original-file = Original Datei
replace-with = Ersetze durch
apply-to-all = Auf alle anwenden
keep-both = Beide behalten
skip = Überspringen

## Set as Executable and Launch Dialog
set-executable-and-launch = Set as executable and launch
set-executable-and-launch-description = Do you want to set "{$name}" as executable and launch it?
set-and-launch = Set and launch

## Metadata Dialog
owner = Besitzer
group = Gruppe
other = Andere
read = Lesen
write = Schreiben
execute = Ausführen

# Context Pages

## About
git-description = Git commit {$hash} vom {$date}

## Add Network Drive
add-network-drive = Netzlaufwerk hinzufügen
connect = Verbinde
connect-anonymously = Verbinde anonym
connecting = Verbinde...
domain = Domäne
enter-server-address = Enter server address
network-drive-description =
    Server addresses include a protocol prefix and address.
    Beispiele: ssh://192.168.0.1, ftp://[2001:db8::1]
### Make sure to keep the comma which separates the columns
network-drive-schemes =
    Available protocols,Prefix
    AppleTalk,afp://
    File Transfer Protocol,ftp:// or ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// or ssh://
    WebDav,dav:// or davs://
network-drive-error = Kann Netzlaufwerk nicht verbinden
password = Passwort
remember-password = Password speichern
try-again = Nochmal versuchen
username = Nutzername

## Operations
cancelled = Abgebrochen
edit-history = Historie bearbeiten
history = Historie
no-history = leer.
pending = Ausstehend
progress = {$percent}%
progress-cancelled = {$percent}%, abgebrochen
progress-paused = {$percent}%, pausiert
failed = Fehlgeschlagen
complete = Fertig
copy_noun = Copy
creating = Creating {$name} in {$parent}
created = Created {$name} in {$parent}
copying = Copying {$items} {$items ->
        [one] item
        *[other] items
    } from {$from} to {$to}
copied = Copied {$items} {$items ->
        [one] item
        *[other] items
    } from {$from} to {$to}
emptying-trash = Emptying {trash}
emptied-trash = Emptied {trash}
setting-executable-and-launching = "{$name}" ausführbar machen und starten
set-executable-and-launched = "{$name}" wurde ausführbar gemacht und gestartet
moving = {$items} {$items ->
        [one] verschiebe Datei
        *[other] items
    } von {$from} nach {$to}
moved = {$items} {$items ->
        [one] Dateien verschoben
        *[other] items
    } von {$from} nach {$to}
renaming = {$from} umbenennen nach {$to}
renamed = {$from} umbenannt nach {$to}
restoring = {$items} {$items ->
        [one] Datei von
        *[other] Dateien wiederhergestellt
    } aus dem {trash}
restored = {$items} {$items ->
        [one] Datei von
        *[other] Dateien wiederhergestellt
    } aus dem {trash}
unknown-folder = unbekannter Ordner

## Open with
open-with = Öffnen mit...
default-app = {$name} (default)

## Show details
show-details = Show details
type = Type: {$mime}
items = Items: {$items}
item-size = Size: {$size}
item-created = Created: {$created}
item-modified = Modified: {$modified}
item-accessed = Accessed: {$accessed}
calculating = Calculating...

## Search
search-context = Search
mediatypes = Suche Medien Typen
images = Images
videos = Videos
audios = Audio Dateien
textentry = Text der gesucht werden soll
text-from = Von / Minimum (Text)
text-to = Bis / Maximum (Text)
value-from = Von / Minimum (Zahl)
value-to = Bis / Maximum (Zahl)

## Settings
settings = Einstellungen
settings-tab = Reiter
settings-show-hidden = versteckte Dateien
default-view = Standard Ansicht
icon-size-list = Bild (Liste)
icon-size-grid = Bild (Grid)
sorting-name = Sortieren nach
direction = Richtung
ascending = Aufsteigend
descending = Absteigend

### Appearance
appearance = Aussehen
theme = Thema
match-desktop = Wie System
dark = Dunkel
light = Hell

# Context menu
add-to-sidebar = Zur Seitenleiste hinzufügen
compress = Komprimieren
extract-here = Auspacken
new-file = Neue Datei...
new-folder = Neuer Ordner...
open-in-terminal = Im Terminal öffnen
move-to-trash = Löschen
restore-from-trash = Wiederherstellen
remove-from-sidebar = Aus Seitenleiste entfernen
sort-by-name = Nach Namen sortieren
sort-by-modified = neueste zuerst
sort-by-size = kleinste zuerst

# Menu

## File
file = Datei
new-tab = Neuer Reiter
new-window = Neues Fenster
rename = Umbenennen...
menu-show-details = Details anzeigen...
close-tab = Tab schließen
quit = Beenden

## Edit
edit = Bearbeiten
cut = Ausschneiden
copy = Kopieren
paste = Einfügen
select-all = Alle Auswählen

## View
zoom-in = Zoom +
default-size = Standard
zoom-out = Zoom -
view = Ansicht
grid-view = Bilderansicht
list-view = Listenansicht
show-hidden-files = versteckte Dateien anzeigen
list-directories-first = Verzeichnisse zuerst
menu-settings = Einstellungen...
menu-about = Über COSMIC Medien Betrachter...

## Sort
sort = Sort
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Neuestes zuerst
sort-oldest-first = Ältestes zuerst
sort-smallest-to-largest = Kleinstes zu Größtes
sort-largest-to-smallest = Größtes zu Kleinstes

# Buttons
button-back = Zurück
button-previous-file = Vorherige
button-next-file = Nächste
button-play = Abspielen
button-pause = Pause
button-mute = Ton aus
button-loop-on = wiederholen
button-loop-off = nicht wiederholen
button-subtitle = Untertitel
button-audio = Tonspur
button-zoom-plus = Zoom +
button-zoom-minus = Zoom -
button-zoom-fit = Anpassen
button-seek = ...

descripttion-back = Zurück zur Dateiübersicht
description-previous-element = Vorherige Datei in der Liste
description-next-element = Nächste Datei in der Liste
description-play = Abspielen
description-pause = Pause
description-mute = Ton aus
description-loop-on = Aktuelles Medium wiederholen
description-loop-off = Aktuelles Medium nicht wiederholen
description-subtitle = Untertitel auswählen
description-audio = Tonspur auswählen
description-zoom-plus = hinein Zoomen
description-zoom-minus = heraus Zoomen
description-zoom-fit = Bild in das Fenster einpassen
description-seek = Aus anderen Dateien wählen

# Player
audio = Tonspur
subtitles = Untertitel