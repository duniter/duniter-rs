# How to write your Durs module

Date: 2018-11-20
Authors: elois

In this tutorial we'll see how to develop a new module for Durs, the Rust implementation of [Duniter](https://duniter.org).

You are expected to have [setup your development environment](setup-your-dev-environment.md).

## General architecture

The Durs repository is composed of two types of crates: binaries and libraries.

There are two binary crates in the `bin/` directory:

* `durs-server`: builds an executable for the command line, targetting a server install,
* `durs-desktop`: builds a Graphical User Interface (GUI), for the desktop, also in the form of one executable. This one doesn't exist yet.

A Durs module is a library crate. You must then create your crate in the `lib/` directory.

The `lib/` directory has 4 sub-directories which correspond to 4 types of libraries:

1. `tools`: utilities that could be useful to all crates.
2. `modules`: libraries forming a Durs module.
3. `modules-lib`: libraries dedicated to a subset of modules.
4. `core`: libraries structuring the architecture, linking modules between them.

As such, create a new crate in a directory called `modules/{your-module-name}`.
The name of your crate to put in the `Cargo.toml` file must be prefixed by `durs-`. The folder in which lies your module doesn't have this prefix.

For example: you create a new module named "toto". You put the crate, which contains your module's code, int `lib/modules/toto`. In `Cargo.toml` you add `durs-toto`.

### How to split a module in several crates

If you want to decouple your module in several crates, the directory of your main crate must be `lib/modules/{your-module-name}/{your-module-name}`.
The additional crates must be in `modules/{your-module-name}/{crate-name}`, where `crate-name` must be prefixed by `{your-module-name}`.

For example: you want to move some of your "toto" code into a new "tata" crate. You must move `toto` into `lib/modules/toto/toto` and create the tata module into `lib/modules/toto/toto-tata`. In addition, your new crate must declare, into its `Cargo.toml`, the name `durs-toto-tata`.

In general: the folder of a crate must have the same name of the crate but **without the durs- prefix**.

### How to develop a lib for several modules

If you want to write a library to be used by several modules and by them only, you'll have to put it into `modules-common`. Note that this folder doesn't exist yet because we didn't have the need yet. Go ahead and create it if you need.

The `tools/` directory must only contain libraries that are also use by the core.

Summary:

* if a library is used by the core and maybe by some modules: into `tools/`.
* if it is used only by modules: into `modules-common/`.
* when used by only one module: into `/modules-lib/{MODULE-NAME}/`.

## The skeleton module

To help bootstrapping, the project has a `skeleton` module that has everything a module needs to work. From now on, we'll guide you step by step to create your own module from a copy of the `skeleton`.

You might want to delete some parts of the code. Indeed, the skeleton shows common uses, like how to change your module's configuration or how to split it up in threads, but you might not need them all.

Of course, the skeleton contains compulsory bits, and we'll start just by them.

## The `DursModule` trait

The only thing you must do for your module to be recognized by the core is to expose a public structure that implements the `DursModule` trait.

Then you just have to change the binary crates for them to import your structure that implements the `DursModule` trait (see below).

Traits are a core feature of Rust, we use them all the time. They ressemble somewhat to interfaces that we find in other languages.
A trait defines a **behavior**. It exposes methods (like interfaces), parent traits (as in class inheritance), but it can also expose types, and this is something `DursModule` does.

The `DursModule` trait exposes 2 types that you'll have to define in your module: `ModuleConf` and `ModuleOpt`.

### The `ModuleConf` type

This is a type representing your module's configuration. If it doesn't have configuration, you can create a type with a void structure for it:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Skeleton Module Configuration
pub struct YourModuleConf {}

impl Default for YourModuleConf {
    fn default() -> Self {
        YourModuleConf {}
    }
}
```

The `ModuleConf` type must implement different traits for the core to handle it. Many of them can be done automatically by the compiler, and we ask it with the `#[derive(…)` macro. You only have one left to implement, the `Default` given above, used by the core to generate the default configuration.

The skeleton module gives a configuration example with a `String` field named `test_fake_conf_field`.

### The `ModuleOpt` type

This type represents your module's command line arguments. If it doesn't use any, you can again do with a void structure:

```rust
#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "skeleton",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
/// YourModule subcommand options
pub struct YourModuleOpt {}
```

In the skeleton you'll find a complete example with one field.

### The `DursModule` functions

This trait exposes 6 functions. 4 are mandatory to implement. The 2 optional have a default implementation.

The 4 mandatory are:

* `name`: must return your module's name
* `priority`: must say if the module is mandatory or optional and if it is activated by default.
* `ask_required_keys`: must list the cryptographic keys your module needs.
* `start`: a function called in a dedicated thread when the durs core is run (and only if your module is activated). Kind of the "main".

The 2 optional functions are:

* `have_subcommand`: returns a boolean telling if your module must inject a subcommand to the `durs` command. The default is `false`.
* `exec_subcommand`: function called when the user has called your subcommand. The default implementation does nothing.

Now let's see them in details.

#### The `name` function (mandatory)

Declaration:

```rust
    /// Returns the module name
    fn name() -> ModuleStaticName;
```

`ModuleStaticName` is a tuple struct which contains one single element of type `&'static str`. It is common practice in Rust to encapsulate standard types in tuple structs in order to manipulate more expressive types. This abstraction has zero cost, since the compiler de-encapsulates the types for the final binary. Using tuple structs is considered good practice. To learn about the `&'static str`, see the [Rust Book](https://doc.rust-lang.org/book/second-edition/ch10-03-lifetime-syntax.html#the-static-lifetime).

In practice, the type `ModuleStaticName` is trivial to use. If your module is named `toto`, you can write:

```rust
    fn name() -> ModuleStaticName {
        ModuleStaticName("toto")
    }
```

You'll probably need your module's name at different places, so we can create a global constant:

```rust
static MODULE_NAME: &'static str = "toto";
```

and use it into `name`:

```rust
    fn name() -> ModuleStaticName {
        ModuleStaticName(MODULE_NAME)
    }
```

#### The `priority` function

Its declaration:

```rust
    /// Returns the module priority
    fn priority() -> ModulePriority;
```

There are different priority levels in Duniter: see https://nodes.duniter.io/rust/duniter-rs/durs_module/enum.ModulePriority.html

You just have to choose one of the three variants and return it. For example, if your module is optional and not activated by default:

```rust
    fn priority() -> ModulePriority {
        ModulePriority::Optional()
    }
```

#### The `ask_required_keys` function

Its declaration:

```rust
    /// Indicates which keys the module needs
    fn ask_required_keys() -> RequiredKeys;
```

The `RequiredKeys` enum is presented on the documentation: https://nodes.duniter.io/rust/duniter-rs/durs_module/enum.RequiredKeys.html.

As above, you just have to return a variant. For example, if you don't need any key:

```rust
    fn ask_required_keys() -> RequiredKeys {
        RequiredKeys::None()
    }
```

#### The `have_subcommand` function

It looks like:

```rust
    /// Define if module have a cli subcommand
    fn have_subcommand() -> bool {
        false
    }
```

If you do have a subcommand, simply return `true`.

#### The `exec_subcommand` function

```rust
    /// Execute injected subcommand
    fn exec_subcommand(
        soft_meta_datas: &SoftwareMetaDatas<DC>,
        keys: RequiredKeysContent,
        module_conf: Self::ModuleConf,
        module_user_conf: Option<Self::ModuleUserConf>,
        subcommand_args: Self::ModuleOpt,
    ) -> Option<Self::ModuleUserConf> {
    }
```

ping us if you need this documentation :)

#### The `start` function

```rust
    /// Launch the module
    fn start(
        soft_meta_datas: &SoftwareMetaDatas<DC>,
        keys: RequiredKeysContent,
        module_conf: Self::ModuleConf,
        main_sender: mpsc::Sender<RouterThreadMessage<M>>,
        load_conf_only: bool,
    ) -> Result<(), ModuleInitError>;
```

Note: even in the case of a simple module that only declares a subcommand, it must implement `start`. I opened an issue to improve this, see https://git.duniter.org/nodes/rust/duniter-rs/issues/112

The first thing to do in start is to check your module's integrity.
If you notice any error in its configuration, you must stop the program with an explicit error message.

Then, if `load_conf_only` is `true` you don't have anything more to do, simply return `Ok(())`.
If it is `false`, you must run your module, and this is done in the following steps:

1. create your channel
2. create your endpoint, if necessary
3. register your module to the router
4. do stuff before the main loop
5. run the main loop, in which you'll listen to the incoming messages to your channel

If the router doesn't receive all the modules' registration in less than 20 seconds, it stops the program. You must then register it before any intensive action.
If you think 20 seconds is large anyway, keep in mind that Durs is aimed to run on low-performance micro-computers, that could even busy with other tasks. It is possible to not include your module into the arm build.

In short: if your module takes more than 3 seconds to register on your pc, it's already too much.

## Injecting your module in binary crates

First, you must add your module to the binary crates' dependencies. They are declared in the `Cargo.toml`.

To add `toto`, add the following line to `bin/durs-server/Cargo.toml`:

    durs-toto = { path = "../../lib/modules/toto" }

## Injecting your module into `durs-server`

Once you added your module in `durs-server` dependencies as shown above, you'll want to use it from the main.rs:

1. use the struct that implements `DursModule`:

    pub use durs_toto::TotoModule;

2. Add your module to the macro `durs_plug!` :

    durs_plug!([WS2Pv1Module], [TuiModule, .., TotoModule])

Its first argument is a list of modules of type network inter-node. Other modules go into the second list.

3. If your module injects a subcommand, add it to the `durs_inject_cli!` macro:

    durs_inject_cli![WS2Pv1Module, .., TotoModule],

And now ping us if you want more tutorial…
