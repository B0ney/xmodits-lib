# XMODITS Core Library

Work in progress

## Supported Formats
| Extension | Description |
| --- | --- |
| IT | Impulse Tracker |
| XM | Extended Module | 
| S3M | Scream Tracker 3 |
| MOD | Amiga ProTracker |
| MPTM | ModPlug Tracker module (Impulse Tracker) |

## Supported Containers
| Extension | Description |
| --- | --- |
| UMX | Unreal Music Package|
| PT36 | ProTracker 3.6 project file |


## Formats samples can be exported to:
| Extension | Format |
| --- | --- |
| WAV | Microsoft Wave |
| AIFF | Audio Interchange File Format | 
| IFF (8SVX) | 8-Bit Sampled Voice |
| ITS | Impulse Tracker 2 Sample |
| S3I | Scream Tracker 3 Instrument|
| RAW | Headerless PCM |


## API
Subject to change

Extract a module from a path:

```rust
use xmodits_lib::{Ripper, AudioFormat};

let self_contained_samples = true;

Ripper::default()
    .audio_format(AudioFormat::ITS) // Export samples to the impulse tracker instrument instead of .wav
    .extract_from_path(
        "./module1.xm", 
        "~/Downloads/", 
        self_contained_samples
    );
```

Load a module from a path:
```rust
let module = xmodits_lib::load_from_path("./module1.xm").expect("valid module");
```

## License
The xmodits core library is licensed under the Mozilla Public License 2 (MPLv2)
