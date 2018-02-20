# Contributing

When contributing to this repository, please first discuss the change you wish to make via issue and
via the [forum](https://forum.duniter.org) before making a change.

Please note we have a specific workflow, please follow it in all your interactions with the project.

## Workflow

- You must create an issue for each feature you wish to develop, with a precise title and a
  description of your feature. Then, assign it to yourself and click on the button
  **`Create a merge request`**. GitLab will create a branch dedicated to the issue as well as a
  *Work in Progress* merge request of this branch into the main branch (`dev`).
- Please use tags to specify feature domains and concerned crates.
- Never contribute to a branch whose issue has not been assigned to you! If the contributor make a 
  `git rebase` your commit will be lost !
- Before you push your commit: 
  1.  Apply `rustfmt`. Only formatted code will be accepted, and gitlab-ci will make sure it is ok.
      Some exceptions are accepted such as long raw strings. 

      `rustfmt` is a tool applying Rust idiomatic code style to your files automatically.

      If you're using rust 1.24 or greater :

      ```bash
      # Install rustfmt through rustup
      rustup component add rustfmt
      # Run rustfmt
      cargo fmt
      ```

      If you're using rust 1.23 or lower :

      ```bash
      # Install nightly toolchain
      rustup install nightly
      # Install rustfmt through cargo
      cargo +nightly install rustfmt
      # Run rustfmt
      cargo +nightly fmt
      ```

      > If you switch from 1.23 to 1.24, uninstall `rustfmt` from cargo and install it back with
      > `rustup`

  1.  Use `clippy`.

      `clippy` is a linting tool scanning your code to find common mistakes or bad code.

      Currenctly `clippy` is only available on the `nigthly` toolchain.

      ```bash
      # Install nightly toolchain
      rustup install nightly
      # Install clippy through cargo
      cargo +nightly install clippy
      # Run clippy
      cargo +nightly clippy
      ```
  
  1.  Document your code and avoid any unsafe feature.

      Each create should contain in its root file

      ```rust
      #![deny(missing_docs, missing_debug_implementations, missing_copy_implementations,
        trivial_casts, trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
        unused_qualifications)]
      ```

      It forces you to write documentation for any public code and prevent you from using unstable
      or unsafe code.
      
  1.  Write unit tests, and verify that they **all** pass.
  1.  Made a git rebase to make your history clean.

      ```bash
      # Make an alias
      git config --global alias.tidy "rebase -i @{upstream}"
      # Rebase your last commits since last push
      git tidy
      ```

## Merge Process

1. Ensure any install or build dependencies are removed before the end of the layer when doing a 
   build.
2. Update the README.md with details of changes to the interface, this includes new environment 
   variables, exposed ports, useful file locations and container parameters.
3. Increase the version numbers in any examples files and the README.md to the new version that this
   Pull Request would represent. The versioning scheme we use is [SemVer](http://semver.org/).
4. You may merge the Merge Request in once you have the sign-off of two other developers, or if you 
   do not have permission to do that, you may request the second reviewer to merge it for you.