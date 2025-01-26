# media-browser

Media browser with database backend for the COSMIC desktop environment. It manages media, makes it searchable by internal or external metadata and displays / plays them.

EXIF metadata for images, NFO files for videos (XBMC/Kodi style) and ID3/MP4tag metadata for audio files are stored in a database and can be searched to produce a list of files matching, regardless where they are stored.

Search results and existing directories are navigatable with keyboard or picking files in preview.

The GUI of the file manager part is a clone of COSMIC files with a few modifications.

The GUI of the video / audio player is a clone of COSMIC player. The player itself is a modification of iced-video-player.

The Image viewer GUI is inspired by gthumb.

The backend is a genuine creation.

This project is developed and tested on Linux using Wayland and Pipewire. Gstreamer supports any audio and video pipeline. But the GUI is libcosmic, which is a Wayland only framework. It should be possible to run this on any Linux/Wayland desktop. And also WSL2 on Windows 11. I just will not test it.

> [!NOTE]
> The current Status is feature complete for the 1.0 release. We are in beta mode for the release.

## Required dependencies

Video and audio playback requires GStreamer.

Gstreamer is modular. Depending on what formats you want to playback (mp3, m4a, aiff, flac, mp4, mkv, wmv, av1, vp9, h264, hevc, ...) and what backend you want to use (pipewire, pulseaudio, alsa, jack, ...) specific gst-plugin-`format` have to be installed.

[Installing Gstreamer on Linux](https://gstreamer.freedesktop.org/documentation/installing/on-linux.html?gi-language=c)

Creation of video thumbnails and metadata extraction from video and audio require an installation of ffmpeg in available to execute from the command line. Most linux distributions install that or at least have a copy available in the repositories.

You will need a rust environment to compile the project.

[Installing Rust](https://www.rust-lang.org/tools/install)

## Build the project from source

```sh
# Clone the project using `git`
git clone https://github.com/fangornsrealm/media-browser
# Change to the directory that was created by `git`
cd media-browser
# Build an optimized version using `cargo`, this may take a while
cargo build --release
# Run the optimized version using `cargo`
cargo run --release
```

## License

This project is licensed under [GPLv3](LICENSE)
Parts coming from external projects are specially marked
Usually they are licensed [MIT](http://opensource.org/licenses/MIT)
