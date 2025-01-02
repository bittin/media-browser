# Todo list

Positions ended with a ? are *stretch goals*. If it is possible AND somebody does the work, it would be good to have these.

## File browser

### Bugs

- fix bug that Movie directory is not parsed correctly

### Features

DONE resursive scan of the current directory - depends on image to database
DONE skip EXIF extraction for unsupported image formats
DONE Make recursive scan run in the background
DONE filetype, actor, director, artist, albumartist, release date range, duration, chapters, ...
DONE Saved search management
DONE multiple file rename feature, using the sort order of the model
DONE fix the About and settings panel

- Search panel with a separate result model/view
  - (new tab per search result, derived from tab? so ESC returns to the search and the search handles the previous/next)
- disable the sidebar by default
- sort by release date / creation / modification time
- display detail information of entry?
- view files of only one type?
- adjust the sort options when just one type is displayed and more details are available?
- find similar images in background? (duplo-rs runs very long, better started on the command line!)

## Image viewer

DONE add a popup selection strip of the files/images in the same directory
DONE include images into the database

- make the popup strip disappear once the mouse is no longer hovering over it?
- adjust the size and shape away from the original aspect ratio of the image viewer on zoom-in if there is room to grow in the window?
- allow images larger than 2000x2000 pixels to be displayed without scaling it down?
  (cosmic::iced_WGPU crashes currently if files over 2048 are attempted to load into the buffer).

## Video viewer

- make the player zoom a video that is smaller than the display area
- add a browse button to have the same navigation strip as in image view
- add chapter navigation?
- add playback speed control?
- add seek buttons?
- add seek on mouse scroll?
  
## Audio Player

- update the playback positon in the number and slider
- add a browse button to have the same navigation strip as in image view
- add chapter navigation?
- add playback speed control?
- add seek buttons?
- add seek on mouse scroll?
