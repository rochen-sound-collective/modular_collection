<img src="../../img/rochen_sound_collective_logo_512.svg" alt="Rochen Sound Collective Logo" width="200" height="200"/>

# Modular::Euclidian

Modular::Euclidian
## Installation

There is nothing special about installing Modular::Euclidian. It is done like with any other plugin.
Please download the latest version from the [releases](https://github.com/rochen-sound-collective/modular_collection/releases) page on GitHub and copy it to the Plugins folder. The correct 
location of that folder depends on your DAW. 

> Typical folders are:
> - VST-3 Windows:
>   - C:\Program Files\Common Files\VST3
>   - C:\Program Files\VstPlugins
>   - C:\Program Files (x86\VstPlugins
>   - C:\Program Files (x86)\Common Files\VST3
> - CLAP Windows:
>   - C:\Program Files\Common Files\CLAP
> - VST-3 Linux:
>   - ~/.vst3
> - CLAP Linux:
>   - ~/.clap

## DAW Compatibility
The plugin is tested with Bitwig Studio.

Unfortunately I could not convince Ableton Live to work with it, because it is not as flexible as Bitwig, when it comes 
to midi routing and defining an explicit channel for your clips. Sadly, the support for MIDI-only plug-ins in Ableton 
Live is very poor. I'm trying to find a solution to this and I will also test the plugin with other DAWs in the near future.

## OS Compatibility
The plugin is compatible with Windows and Linux. I do not own a Mac, but I do not see a reason why it should not be 
compatible with OSX, so if you are on OSX, have a look at the [building](#Building) section. Building the plugin is not 
terribly difficult. Please let me know if you are having trouble or if you were successful.

## Building

After installing [Rust](https://rustup.rs/), you can compile modular::collection as follows:

```shell
cargo xtask bundle modular_euclidian --release
```
> :warning: **Bitwig Flatpak on Linux** Please note that the flatpak version of Bitwig on Linux might not be able to 
> load the plugin with an error similar to this:
> - /usr/lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.34' not found
> 
> This is because a binary built with GLIBC of a newer version is not compatible to run with an older version of GLIBC.
> This is however contained in the Flatpak. You should update to the newest Bitwig version in this case and if there is
> no newer version you need to build with a lower version of GLIBC. The official releases are always compatible with
> the latest Flatpak.
