# Synchroniser son noeud Durs

## Synchronisation depuis le réseau

Cette fonctionnalitée n'est pas encore intégrée à Durs.

## Synchronisation depuis un noeud Duniter local

Assurez vous d'avoir un noeud Duniter synchronisé sur la même machine.

Vous devez indiquer a Durs le chemin vers le répertoire contenant la blockchain brute sous forme de fichiers JSON. Elle se trouve dans `~/.config/duniter/<profile>/<currency>`.

Exemple:

si vous êtez sous Linux, que l'utilisateur de duniter est `user`, que vous utilisez le profil duniter par défaut et que vous souhaitez vous synchroniser sur la g1; le chemin vers la blockchain brute est :

    home/user/.config/duniter/duniter-default/g1

Vous devez saisir ce chemin dans l'option `--local` de la commande sync comme suis :

    durs sync --local home/user/.config/duniter/duniter-default/g1
