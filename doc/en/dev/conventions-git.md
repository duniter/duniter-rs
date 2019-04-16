# DURS' git conventions

## Branch naming

### Branch created by Gitlab

Most of the time, you'll use the "create a merge request" button and
Gitlab will name your branch. In that case, please prefix the name of
your branch with your Gitlab username and a slash, for example:

    elois/issue-test

### Branch created manually

On all cases anyway, your branch must start by your gitlab account's
username, so than everybody knows who's working on it. Also add it its
type, following this model:

    username/type/description

username := your Gitlab username.
type := see "Commit types" below.
description := short summary in present form, 3 to 4 words maximum, no articles.

Example:

    elois/ref/rename_trait_module

## Naming commits

Every commit must follow this convention:

    [type] crate: action subject

The **type** must be a keyword of the "Commit types" list below.

The **crate** must be the name of the crate in question, without the "durs-" prefix.

The **action** must be a verb in imperative form, the **subject** a noun.

For example, we rename the trait `Foo` to `Fii` in the `durs-crate` crate:

    [ref] mycrate: rename Foo -> Fii

### Commit types

* `build`: Changes in the scripts of build, packaging or publication of releases.
* `ci` :  Changes in the Continuous Integration pipeline.
* `deps` : Changes in dependencies without changes into the code. This can be for update or deletion of third-party libraries.
* `docs` : Changes in documentation (both for translation and new content).
* `feat` : Development of a new feature.
* `fix` : Bug fixing.
* `opti` :  Optimisation: better performances, decrease in memory or disk usage.
* `pub` : commit about the publication of a crate on [crates.io](https://crates.io).
* `ref` : Refactoring. This commit doesn't change the functionnality.
* `style` : Style modification (usually `fmt` and `clippy`).
* `tests` : Changes in tests or new tests.

If you have a new need, please contact the main developers to add a type together.


## Update strategy

We only use **rebases**, *merges* are strictly fordbidden !

Every time the `dev` branch is updated, you must rebase each of your working branch on it. For each of them:

1. Go on your branch
2. Run a rebase on dev:

    git rebase dev

3. If you see conflicts, fix them by editing the sources. Once it is done, you must:
   a. commit the files that were in conflict
   b. continue the rebase with `git rebase --continue`
   c. Do 3. again for each commit that will be in conflict.

4. When you don't have any conflict anymore after `git rebase --continue`, then the rebase succeeded. Then rebase a remaning branch.

## When to push

Ideally, you should push when you are about to shut down your computer, so about once a day.

You must prefix your commit with `wip:` when it is a work in progress.

> But why push if I am not done ?

Pushing is no big deal and prevents you from loosing work in case of
any problem with your material.

## How to merge

When you finished developing, run `fmt` and `clippy` and run all tests:

    cargo +nightly fmt
    cargo +nightly clippy
    cargo test --all

Then commit everything.

In case you had a `wip:` prefix, you can remove it.

If you have a pile of commits, use the useful interactive rebase to clean up your branch history and create atomic ones:

    git rebase -i dev

There you can rename the `wip:` commits, you can "fixup" commits that go together, you can rename and re-order commits,...

After an interactive rebase, your local git history is different that yours in Gitlab, so you need a force push to make it to Gitlab:

    git push -f

Now is time to go to Gitlab and re-check your commits.

Wait for the Continuous Integration pipeline to finish (it lasts ±20min), and at last when it is done you can remove the "WIP" mention of your Merge Request and mention (with "@name") the lead developers to ask for a code review.
