CHIRP-8
=======

This is my attempt at a CHIP-8 interpreter (following just the original spec, no extentions) in Rust. Because it's rusty, I decided to name it `chirp8`.

Installation
------------

If you're interested in installing `chirp8`, you'll need a working installation of Rust. Just clone this repository and build:

```sh
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
$ git clone https://github.com/ryanq/chirp8
$ cargo install --path chirp8
```

Usage
-----

`chirp8` can take a CHIP-8 binary and run with it, but there are also options for configuring the keymap and the rendering size, and the log level for debugging:

```
chirp8 0.1.0
Ryan Quattlebaum <ryan.quattlebaum@icloud.com>
A CHIP-8 interpreter implementing the original spec

USAGE:
    chirp8 [FLAGS] [OPTIONS] <program>

ARGS:
    <program>    Path to a Chip-8 binary

FLAGS:
    -h, --help       Prints help information
    -v, --verbose    Sets logging level
    -V, --version    Prints version information

OPTIONS:
    -k, --keymap <keymap>    Sets the key mapping to use [env: CHIRP_KEYMAP=]  [default: qwerty]  [possible values:
                             colemak, qwerty]
    -s, --size <size>        Sets the rendering size [default: normal]  [possible values: small, normal, large]
```

Key Mapping
-----------

The keyboard for the CHIP-8 looked something like this:

|     |     |     |     |
|-----|-----|-----|-----|
|**1**|**2**|**3**|  C  |
|**4**|**5**|**6**|  D  |
|**7**|**8**|**9**|  E  |
|  A  |**0**|  B  |  F  |

...but that doesn't match the keyboards we have today. So `chirp8` comes with two key layouts available that map to the layout above:

### QWERTY

|     |     |     |     |
|-----|-----|-----|-----|
|**1**|**2**|**3**|  4  |
|**Q**|**W**|**E**|  R  |
|**A**|**S**|**D**|  F  |
|  Z  |**X**|  C  |  V  |

### Colemak

|     |     |     |     |
|-----|-----|-----|-----|
|**1**|**2**|**3**|  4  |
|**Q**|**W**|**F**|  P  |
|**A**|**R**|**S**|  T  |
|  Z  |**X**|  C  |  V  |
