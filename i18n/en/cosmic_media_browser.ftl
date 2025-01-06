cosmic-media-browser = COSMIC Media Browser
empty-folder = Empty folder
empty-folder-hidden = Empty folder (has hidden items)
no-results = No results found
filesystem = Filesystem
home = Home
networks = Networks
notification-in-progress = File operations are in progress.
trash = Trash
recents = Recents
undo = Undo
today = Today

# List view
name = Name
modified = Modified
trashed-on = Trashed
size = Size

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
empty-trash = Empty trash
empty-trash-warning = Are you sure you want to permanently delete all the items in Trash?

## Mount Error Dialog
mount-error = Unable to access drive

## New File/Folder Dialog
create-new-file = Create new file
create-new-folder = Create new folder
file-name = File name
folder-name = Folder name
file-already-exists = A file with that name already exists.
folder-already-exists = A folder with that name already exists.
name-hidden = Names starting with "." will be hidden.
name-invalid = Name cannot be "{$filename}".
name-no-slashes = Name cannot contain slashes.
recursive-scan-directories = Scan all subdirectories for media

## Open/Save Dialog
cancel = Cancel
create = Create
open = Open
open-file = Open file
open-folder = Open folder
open-in-new-tab = Open in new tab
open-in-new-window = Open in new window
open-item-location = Open item location
open-multiple-files = Open multiple files
open-multiple-folders = Open multiple folders
save = Save
save-file = Save file

## Open With Dialog
open-with-title = How do you want to open "{$name}"?
browse-store = Browse {$store}

## Rename Dialog
rename-file = Rename file
rename-folder = Rename folder

## Replace Dialog
replace = Replace
replace-title = {$filename} already exists in this location.
replace-warning = Do you want to replace it with the one you are saving? Replacing it will overwrite its content.
replace-warning-operation = Do you want to replace it? Replacing it will overwrite its content.
original-file = Original file
replace-with = Replace with
apply-to-all = Apply to all
keep-both = Keep both
skip = Skip

## Set as Executable and Launch Dialog
set-executable-and-launch = Set as executable and launch
set-executable-and-launch-description = Do you want to set "{$name}" as executable and launch it?
set-and-launch = Set and launch

## Metadata Dialog
owner = Owner
group = Group
other = Other
read = Read
write = Write
execute = Execute

# Context Pages

## About
git-description = Git commit {$hash} on {$date}

## Add Network Drive
add-network-drive = Add network drive
connect = Connect
connect-anonymously = Connect anonymously
connecting = Connecting...
domain = Domain
enter-server-address = Enter server address
network-drive-description =
    Server addresses include a protocol prefix and address.
    Examples: ssh://192.168.0.1, ftp://[2001:db8::1]
### Make sure to keep the comma which separates the columns
network-drive-schemes =
    Available protocols,Prefix
    AppleTalk,afp://
    File Transfer Protocol,ftp:// or ftps://
    Network File System,nfs://
    Server Message Block,smb://
    SSH File Transfer Protocol,sftp:// or ssh://
    WebDav,dav:// or davs://
network-drive-error = Unable to access network drive
password = Password
remember-password = Remember password
try-again = Try again
username = Username

## Operations
cancelled = Cancelled
edit-history = Edit history
history = History
no-history = No items in history.
pending = Pending
progress = {$percent}%
progress-cancelled = {$percent}%, cancelled
progress-paused = {$percent}%, paused
failed = Failed
complete = Complete
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
setting-executable-and-launching = Setting "{$name}" as executable and launching
set-executable-and-launched = Set "{$name}" as executable and launched
moving = Moving {$items} {$items ->
        [one] item
        *[other] items
    } from {$from} to {$to}
moved = Moved {$items} {$items ->
        [one] item
        *[other] items
    } from {$from} to {$to}
renaming = Renaming {$from} to {$to}
renamed = Renamed {$from} to {$to}
restoring = Restoring {$items} {$items ->
        [one] item
        *[other] items
    } from {trash}
restored = Restored {$items} {$items ->
        [one] item
        *[other] items
    } from {trash}
unknown-folder = unknown folder

## Open with
open-with = Open with...
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
item-media-release-date = Release Date: {$text}
item-media-size = Resolution: {$width} x {$height}
item-media-runtime = Duration: {$text}
item-audio-languange = Languages: {$text}
item-subtitle-language = Subtitles: {$text}
item-media-actor = Actor: {$text}
item-media-director = Director: {$text}
item-media-artist = Artist: {$text}
item-media-albumartist = Album Artist: {$text}
item-media-composer = Composer: {$text}
item-image-lense-model = Lense Model: {$text}
item-image-focal-length = Focal length: {$text}
item-image-exposure-time = Exposure time: {$text}
item-image-fnumber = Lense crop factor: {$text}
item-image-gps-latitude = GPS Latitude: {$text}
item-image-gps-longitude = GPS Longitude: {$text}
item-image-gps-altitude = GPS Altitude: {$text}

## Search
search-context = Search the database
search-mediatypes = Search Media types
search-images = Images
search-videos = Videos
search-audios = Audio files
search-textentry = Search term
search-ranges = Search ranges
search-text-from = From / Minimum (Text)
search-text-to = To / Maximum (Text)
search-value-from = From / Minimum (Number)
search-value-to = To / Maximum (Number)
search-tooltip-date = For Example:
    Date in format YYYY-MM-DDThh:mm:ss
    Example:       2003-01-14T20:15:00
    The Time can be omitted by giving just the date 2003-01-14
search-tooltip-value = Numerical value
search-filepath = Filepath
search-title = Title
search-description = Description
search-actor = Actor/Actress
search-director = Director
search-artist = Artist name
search-album_artist = Album artist
search-duration = Duration
search-creation_date = Creation date
search-modification_date = Modification date
search-release_date = Release date
search-lense_model = Lense model
search-focal_length = Focal length
search-exposure_time = Exposure time
search-fnumber = F-number
search-gps_latitude = GPS latitude
search-gps_longitude = GPS longitude
search-gps_altitude = GPS altitude
search-commit = Start Search

## Settings
settings = Settings
settings-tab = Tab
settings-show-hidden = Show hidden files
default-view = Default view
icon-size-list = Icon size (list)
icon-size-grid = Icon size (grid)
sorting-name = Sort by
direction = Direction
ascending = Ascending
descending = Descending

### Appearance
appearance = Appearance
theme = Theme
match-desktop = Match desktop
dark = Dark
light = Light

# Context menu
add-to-sidebar = Add to sidebar
compress = Compress
extract-here = Extract
new-file = New file...
new-folder = New folder...
open-in-terminal = Open in terminal
move-to-trash = Move to trash
restore-from-trash = Restore from trash
remove-from-sidebar = Remove from sidebar
sort-by-name = Sort by name
sort-by-modified = Sort by modified
sort-by-size = Sort by size

# Menu

## File
file = File
new-tab = New tab
new-window = New window
rename = Rename...
menu-show-details = Show details...
close-tab = Close tab
quit = Quit

## Edit
edit = Edit
cut = Cut
copy = Copy
paste = Paste
select-all = Select all

## View
zoom-in = Zoom in
default-size = Default size
zoom-out = Zoom out
view = View
grid-view = Grid view
list-view = List view
show-hidden-files = Show hidden files
list-directories-first = List directories first
menu-settings = Settings...
menu-about = About COSMIC Media Browser...

## Sort
sort = Sort
sort-a-z = A-Z
sort-z-a = Z-A
sort-newest-first = Newest first
sort-oldest-first = Oldest first
sort-smallest-to-largest = Smallest to largest
sort-largest-to-smallest = Largest to smallest

# Buttons
button-back = Back
button-previous-file = Previous
button-next-file = Next
button-play = Play
button-pause = Pause
button-mute = Mute
button-loop-on = Loop active
button-loop-off = Loop off
button-subtitle = Subtitles
button-audio = Audio
button-zoom-plus = Zoom in
button-zoom-minus = Zoom out
button-zoom-fit = Fit View
button-seek = ...

descripttion-back = Back to File Browser
description-previous-element = Previous element in list
description-next-element = Next element in list
description-play = Play the file
description-pause = Pause the playback
description-mute = Mute the audio
description-loop-on = Repeat the current file
description-loop-off = Repeat the current file
description-subtitle = Select subtitle stream
description-audio = Select audio stream
description-zoom-plus = Zoom in
description-zoom-minus = Zoom out
description-zoom-fit = Fit View of image to the window size
description-seek = Pick from neighbor images

# Player
audio = Audio stream
subtitles = Subtitle stream