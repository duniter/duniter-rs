# Installing DURS on your computer

## Basic install

In any case, you will have to:

1. choose between server version or desktop version
2. which version number you want
3. select the release corresponding to your operating system and processor architecture

### `dunitrust-server` or `durs-desktop`

`durs-desktop` comes with a graphic user interface and is made for people who want to install it in their desktop computer

`dunitrust-server` is much lighter but only has a terminal user interface. It's recommended for:

* Installing on a remote server
* Installing on a low performance computer
* For users who are confortable with command line interfaces

Note: it's possible to remotely control `dunitrust-server` via a graphic user interface (see [durs remote admin]).

## Choose the durs version to install

<s>You can get the last stable version on [the official Dunitrust website](dunitrust.org)</s>

If you have any question about which version to install, ask it on the [duniter forum](https://forum.duniter.org/).

You will find all available version on [this gitlab page](https://git.duniter.org/nodes/rust/duniter-rs/tags)

The 4 types of version are:

* **alpha**: the most advanced test version, likely to have unstable behavior, suitable for alpha-testers
* **beta** : more usable test version, open to all testers
* **RC**: release candidate, a version that can be used in production by advanced users provided they check for updates in case of security issue
* **stable** : the more stable version, intended for all users

### Choose the release correspondig to your operating system and processor architecture

The `Category` column of the releases table tells you which system the release targets. (in case of GNU/Linux system, the distribution is in brackets)

If no release has been made for you, fall back to the manual installation below.

## Manual installation

To install Dunitrust manually, you must first [install Rust](https://www.rust-lang.org/tools/install).

Then install Dunitrust dependencies. Here is how to do on Debian based systems:

    apt-get install pkg-config libssl-dev # install required packages
    git clone https://git.duniter.org/nodes/rust/duniter-rs.git # clone the Dunitrust repository

Change your current directory to the folder correspondig to the variant you want to build:

* For `dunitrust-server`, go into `bin/dunitrust-server`

    cd bin/dunitrust-server

* For `durs-desktop`, go into `bin/dunitrust-desktop`

    cd bin/dunitrust-desktop

Then build Dunitrust with the command:

    cargo build --release --features ssl

In case of problem with `openssl`, you can try building without the `ssl` feature:

    cargo build --release

This just means that your node will not be able to contact the WS2P endpoints that are behind an SSL/TLS layer.  
Your node should still work normally if there is enough unencrypted WS2P endpoints.

If the build succeeds, your binary will lay in `duniter-rs/target/release` with the name `durs` or `durs-desktop`.
You can place it anywhere and run it without other requirement.
