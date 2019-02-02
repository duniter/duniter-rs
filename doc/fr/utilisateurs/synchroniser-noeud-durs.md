# Synchroniser son noeud Durs

## Synchronisation depuis le réseau

Cette fonctionnalitée n'est pas encore intégrée à Durs.

## Synchronisation depuis un noeud Duniter local

Assurez vous d'avoir un noeud Duniter synchronisé sur la même machine.

Si vous éxécutez Durs avec le même utilisateur système, il suffit d'utiliser l'option `--type` comme suit :

    durs sync --type ts

ou

    durs sync --t ts

`ts` fait référence au fait que Duniter est écrit en TypeScript.

Si vous éxécutez Durs et Duniter avec un utilisateur système différent, vous devrez indiquer a Durs le chemin vers le répertoire contenant la blockchain brute sous forme de fichiers JSON. Elle se trouve dans `~/.config/duniter/<profile>/<currency>`.

Exemple: 

si vous êtez sous Linux, que l'utilisateur de duniter est `user`, que vous utilisez le profil duniter par défaut et que vous souhaitez vous synchronisr sur la g1; le chemin vers lma blockchain brute est :

    home/user/.config/duniter/duniter-default/g1

Vous devez coller ce chemin a la fin de la commande sync comme suis :

    durs sync --type ts home/user/.config/duniter/duniter-default/g1

/!\ Cela n'est nécessaire que dans le cas ou vous éxécutez Durs et Duniter avec deux utilisateurs différents ! Si Durs et Duniter utilisent le même utilisateur, Durs trouvera la blockchain brute tout seul.