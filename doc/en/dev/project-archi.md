## General architecture

## Root folders

The Dunitrust repository is composed of two types of crates: binaries and libraries.

Binary crates are located in the `bin` folder and library crates are located in the `lib` folder.

However, the repository does not only contain Rust code, here is what each folder is about:

`.github` : contains templates for the forms that create isues and PR on github to indicate that it is only a mirror repository.
`.gitlab` : contains python scripts to automate the publication of releases on a gitlab wiki page.
`bin` : contains the binary crates, more details in the section dedicated to this folder.
`doc` : contains documentation for users and testers as well as high-level documentation for developers (There is also low-level documentation that is self-generated from the source code).
`images` : contains the images of the project. Some are integrated in the binaries (such as the logo for example), others are integrated on pages of the documentation.
`lib` : contains library crates, more details in the section dedicated to this folder.
`release` : contains the scripts to build and package the deliverables.

## `bin` folder

The binary crates are grouped in the `bin` folder and are two in number:

* dunitrust-server : produces a Dunitrust command-line executable, which can be installed on a server.
* dunitrust-desktop : produces a Dunitrust executable as a desktop graphics application (does not yet exist).

## `lib` folder

The `lib` folder is organized into 8 sub-folders corresponding to 8 types of libraries:

1. `core`: the structuring libraries of the core and interphase with the modules
2. `crypto`: libraries providing cryptographic functionalities.
3. `dubp`: libraries defining the structures of the DU**B**P protocol (DUniter **Blockchain** Protocol) as well as methods to manipulate these structures.
4. `dunp`: libraries defining the structures of the DU**N**P protocol (DUniter **Network** Protocol) as well as methods to manipulate these structures.
5. `modules`: libraries representing a Dunitrust module.
6. `modules-lib`: libraries dedicated only to certain modules.
7. `tests-tools`: libraries providing code used only by automated tests.
8. `tools`: tool libraries, which can potentially be used for all crates. These libraries must not include any Duniter/Dunitrust specific functional logic (i.e. they must be theoretically externalisable).
