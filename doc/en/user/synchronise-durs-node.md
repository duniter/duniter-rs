# Synchronise your Durs node

## Synchronise from the network

This feature is not yet available in Durs.

## Synchronise from a local Duniter node

Make sure you have a Duniter node (duniter-ts) syncronised on the same computer.

If you run Durs with the same system user, you only need to use the `--type` option as follows:

    durs sync --type ts

`ts` refers to the fact that Duniter is written in TypeScript.

If you run Durs and Duniter with a different system user, you must tell Durs the path of the raw blockchain in JSON format.
It is usually located in  `~/.config/duniter/<profile>/<currency>`.

Example:

for a GNU/Linux system, with a duniter user called `user`, the path is:

    home/user/.config/duniter/duniter-default/g1

append it to the end of the command like this:

    durs sync --type ts home/user/.config/duniter/duniter-default/g1

/!\ it is only necessary if you run Durs and Duniter with two different system users.
For the same user, Durs should find the blockchain path automatically.
