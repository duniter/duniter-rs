# Set up your dev environment

Date: 2018-11-19
Authors: elois

In this tutorial we will see how to install a complete [Rust](https://www.rust-lang.org) environment.  
This will be useful for your own Rust projects, or to contribute to Durs, or to do NodeJS/Rust binding.

## Stable toolchain installation

Install Rust's stable toolchain :

    curl https://sh.rustup.rs -sSf | sh

Add ~/.cargo/bin to your PATH environment variable :

    export PATH="$HOME/.cargo/bin:$PATH"

I strongly recommend that you add this line to your terminal configuration file so that you don't have to copy it every time, if you don't know what I'm talking about then you most probably use the default shell (bash) and the file to which you need to add this line is `~/.bashrc`.

You will also need an integrated development environment, I recommend Visual Studio Code because it supports both NodeJs and Rust :)
You can find instructions on how to install vscode for your system on the Internet.

You can also develop in Rust with the following IDE/editors :

* IntelliJ Rust
* Eclipse/Corrosion
* Emacs
* VIM/Rust.vim
* Geany
* Neovim

 And many others..

## Fmt : le formateur de code

I strongly recommend that you install the essential automatic code formatter, especially since it is maintained by the official Rust language team so you have the guarantee that your code will always compile (and will always have the same behavior) after the formatter's pass.

Install `fmt` :

    rustup component add rustfmt-preview

To automatically format your code, go to the root of your project and execute the following command :

    cargo fmt

I strongly recommend that you create an alias in your shell configuration (~/.bashrc if you use bash). As an example I created the alias `fmt="cargo +nightly fmt"`.

## Clippy: the rust linter

If you contribute to Duniter's Rust implementation you will also need to use the Clippy linter. And in any case it's strongly recommended to beginners in Rust to use it, indeed clippy is very educational and will help you learn a lot how to code in Rust.

Run the following command to install Clippy :

    rustup component add clippy-preview

To launch clippy, go to the root of your project and execute the following command :

    cargo clippy --all

Clippy will then inform you in a very educational way about everything that needs to be modified in your code to be more in the "rust spirit" (We say then that your code is more "rusty").

## Vscode

[https://code.visualstudio.com/docs/setup/linux#_debian-and-ubuntu-based-distributions](https://code.visualstudio.com/docs/setup/linux#_debian-and-ubuntu-based-distributions)

Once vscode is installed we will need the following 3 plugins :

* [BetterTOML](https://github.com/bungcip/better-toml)
* [CodeLLDB](https://github.com/vadimcn/vscode-lldb)
* [Rust](https://github.com/editor-rs/vscode-rust) (attention, not "Rust (rls)")

Configuration of the Rust plugin:

In the parameters of the Rust extension, you have the choice between modifying the parameters in the GUI or manually in the `.json` file. We provide here the ligns to be added in the `settings.json` file (which is by default in `~/.config/Code/User/settings.json`).

1. Switch to legacy mode to deactivate RLS (=Rust Language Server) which does not work on the durs project (it requires 100% of CPU).

```json
"rust.mode": "legacy",
```

1. Install the racer (for auto-completion) and sym (to get "go-to-definition" via Ctrl+clic).

```bash
cargo +nightly install racer
cargo install rustsym
```

1. In the `settings.json` file, provide the racer and rustsym paths:

```json
"rust.racerPath": "/home/YOUR_USERuser/.cargo/bin/racer",
"rust.rustsymPath": "/home/YOUR_USERuser/.cargo/bin/rustsym"
```

1. Save the `settings.json` file and restart vscode to apply the changes and definitely stop rls.

### VSCode: LLDB debugger

[Instructions to install LLDB for vscode](https://github.com/vadimcn/vscode-lldb/wiki/Installing-on-Linux)

Select "LLDB adapter type: native" in the LLDB parameters or add the following in the `settings.json` file: 

```json
"lldb.adapterType": "native",
```

To setup and start the debugger, refer to [the vscode doc](https://code.visualstudio.com/docs/editor/debugging#_launch-configurations). Here is a `launch.json` file example for VSCode:

```json
{
    // https://go.microsoft.com/fwlink/?linkid=830387
    "version": "0.2.0",
    "configurations": [
        {
            "name": "Debug",
            "type": "lldb",
            "request": "launch",
            "program": "${workspaceFolder}/target/debug/dunitrust",
            "cwd": "${workspaceRoot}",
            "terminal": "integrated",
            "args": ["start"],
            "env": {
                "RUST_BACKTRACE": "1"
            }
        }
    ]
}
```

### Vscode: mouse navigation like Intellij

Intellij allows to navigate in the code with the next/previous keys of the mouse. Here is how to replicate this behavior in vscode:

1. In vscode, define the keyboard shortcuts for the `navigate back` and `navigate forward` actions (Ctrl+Left and Ctrl+Right for example).
2. Install `xbindkeys` and `xdotool`.
3. Create the xbindkeys configuration file at the root of your home as follows:

```
cd ~
xbindkeys --defaults > .xbindkeysrc
```

4. Add the following lines in the `~/.xbindkeysrc` file:

```bash
## Navigate back
"xdotool key ctrl+Left"
        b:8

## Navigate forward
"xdotool key ctrl+Right"
        b:9
```

1. To verify that everything is well configured, launch the `xbindkeys -v` command and click on your mouse next/previous buttons. The command corresponding to the button you clicked should appear in the console (`xdotool key ctrl+Left` or `xdotool key ctrl+Right`). Check that vscode reacts as expected.

2. Configure your system to start `xbindkeys` on startup.

## Additional packages to compile durs

Although this is becoming increasingly rare, some rust crates still depend on C/C++ libraries and these must be installed on your computer at compile time. On Debian and derivatives, you must have `pkg-config` installed because the rust compiler uses it to find the C/C++ libraries installed on your system.

    sudo apt-get install pkg-config

## Test your environment with a traditional "Hello, World!"

    mkdir hello-world
    cd hello-world
    cargo init --bin

The `--bin` option indicates that you want to create a binary, by default cargo create a library project.

You should have the following content in the `hello-world` folder:

    $ tree
    .
    ├── Cargo.toml
    ├── src
    │   └── main.rs

This is the minimum content of any binary project, the source code is found in `main.rs`.
Any Rust project (binary or library) must contain a file named `Cargo.toml` at the root of the project, it is somehow the equivalent of the `package.json` of npm.

The `main.rs` file already contains by default a code to perform the traditional "Hello, world !":

    fn main() {
        println!("Hello, world!");
    }

This syntax must remind you furiously of C/C++ for those who know it, and that's normal because Rust is designed to be one of the potential successors of C++. However, three major differences can already be noted with C/C++ :

1. The main() function does not take any input parameters. Command line arguments are captured in a different way using the standard library.
2. println! is not a function, it's a macro. In Rust all macros are of the form `macro_name!(params)`, so it is to `!` that they are recognized. So why a macro just to print a string? Well because in Rust any function must have a finite number of parameters and each parameter must have an explicitly defined type. To exceed this limit we use a macro that will create the desired function during compilation.
3. The main() function doesn't return any value, when your program ends, Rust sends by default the EXIT_SUCCESS code to the OS. To interrupt your program by sending another exit code, there are macro such as `panic!(err_message)`.

Before changing the code, make sure that the default code compiles correctly:

    $ cargo build
    Compiling hello-world v0.1.0 (file:///home/elois/dev/hello-world)
    Finished dev [unoptimized + debuginfo] target(s) in 0.91 secs

Cargo is the equivalent of npm for Rust, it will look for all the dependencies of the crates you install. In Rust a "crate" refers to any library or package. It's comparable to Python wheels, Java archive, Ruby gems...

If you get a `Finished dev[unoptimized + debuginfo] target(s) in x.xx secs`, congratulations you just compiled your first Rust program :)

If you get an error it's because your Rust environment is not correctly installed, in this case I invite you to uninstall everything and restart this tutorial from scratch.

> It compiles for me, How do I run my program now ?

Like that :

    $ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.0 secs
    Running `target/debug/hello-world`
    Hello, world!

As indicated, `cargo run` executes your binary which is actually in `target/debug/`.

There are several compilation profiles, and you can even create your own, two pre-configured profiles are to be known absolutely :

1. The `debug` profile: this is the default profile, the compiler does not perform any optimization and integrates into the binary the entry points allowing a debugger to work.
2. The `release` profile: the compiler performs as much optimization as possible and does not integrate any entry point for the debugger.

Rust is known to be very fast, this is largely due to the extensive optimizations made during a `release` compilation, but making these optimizations takes time, so the `release` compilation is much longer than the `debug` mode.

To compile in `release` mode:

    cargo build --release

Your final binary is then in `target/release/`.

To go further, the reference of the references you must absolutely read is obviously the sacred [Rust Book](https://doc.rust-lang.org/book/).
