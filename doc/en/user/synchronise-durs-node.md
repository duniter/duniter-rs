# Synchronise your Dunitrust node

## Synchronise from the network

This feature is not yet available in Dunitrust.

## Synchronise from a local Duniter node

Make sure you have a Duniter node (duniter-ts) syncronised on the same computer.

You must tell Dunitrust the path of the raw blockchain in JSON format.
It is usually located in  `~/.config/duniter/<profile>/<currency>`.

Example:

for a GNU/Linux system, with a duniter user called `user`, the path is:

    home/user/.config/duniter/duniter-default/g1

append it to the `--local` option of the `sync` command like this:

    durs sync --local ts home/user/.config/duniter/duniter-default/g1
