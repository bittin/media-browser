# Todo list

Positions ended with a ? are *stretch goals*. If it is possible AND somebody does the work, it would be good to have these.

## File browser

### Bugs

- fix bug that Movie directory is not parsed correctly

### Features

X resursive scan of the current directory - depends on image to database
X skip EXIF extraction for unsupported image formats
X Make recursive scan run in the background

- Search panel with a separate result model/view
  - (new tab per search, derived from tab? so ESC returns to the search and the search handles the previous/next)
  - filetype, actor, director, artist, albumartist, release date range, duration, chapters, ...
  - Saved search management
- disable the sidebar by default
- sort by release date / creation / modification time
- multiple file rename feature, using the sort order of the model
- fix the About and settings panel
- view files of only one type?
- adjust the sort options when just one type is displayed and more details are available?
- find similar images in background? (duplo-rs runs very long, better started on the command line!)

## Image viewer

X add a popup selection strip of the files/images in the same directory
X include images into the database

- make the popup strip disappear once the mouse is no longer hovering over it?
- adjust the size of the image viewer on zoom-in if there is space in the window?
- allow images larger than 2000x2000 pixels to be displayed without scaling it down? WGPU crashes currently.

## Video viewer

- make the player zoom a video that is smaller than the diaplay area
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
