<img src="img/modular-collection-logo.png" alt="Modular Logo" width="200" height="200"/>
<img src="img/rochen_sound_collective_logo_512.svg" alt="Rochen Sound Collective Logo" width="200" height="200"/>

# Modular::Collection

Modular::Collection is a collection of open source audio plugins in development.

## Plugins

Right now there is only [Modular::Patterns](collection/patterns/README.md). Hopefully other plugins will follow.

### Modular::Patterns
___

<img src="img/modular-patterns-logo.png" alt="Modular::Patterns Logo" width="200" height="200"/>

A MIDI arpeggiator that combines a chord channel with a pattern. It's main difference to other plugins (I know of) it 
uses midi input for both chords and patterns. This makes it possible to use the UI of your DAW to edit you MIDI clips 
and tracks. 

[More](collection/patterns/README.md)

### Modular::Chords [planned]

A plugin for creating chord sequences.

### Modular::Transmitter/Receiver [planned]

Two plugins to allow flexible MIDI routing for DAWs where it is not supported natively. 

## Building

After installing [Rust](https://rustup.rs/), you can compile modular::collection as follows:

Modular::Patterns
```shell
cargo xtask bundle modular_patterns --release
```

