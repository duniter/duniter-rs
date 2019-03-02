# Installer DURS sur votre ordinateur

## Installation simple

Dans tout les cas vous aurez 3 choix a faire :

1. Choisir entre durs-server ou durs-desktop
2. Choisir la version de Durs que vous souhaitez installer
3. Choisir le livrable correspondant votre système d'exploitation et votre processeur.

### `durs-server` ou `durs-desktop`

`durs-desktop` est destiné aux utilisateurs souhaitant installer Durs sur leur ordinateur personnel et administrer leur noeud Durs via une interface graphique.

`durs-server` est beaucoup plus léger et se manipule via la ligne de commande. Il est nottament utile dans les cas suivants :

* Installation de durs sur serveur dédié
* Installation de durs sur micro pc (raspberry pi, brique internet, etc)
* Pour les utilisateurs avancé qui préfèrent la ligne de commande.

Notez bien : il est possible d'administrer `durs-server` a distance via une interface graphique (voir [administrer un noeud durs a distance]).

### Choisir la version de Durs a installer

<s>Rendez vous sur [le site officiel de Durs](durs.info), vous y trouverez un lien direct vers la dernière version stable.</s>

Le site web de Durs n'existe pas encore, en attendant vous devrez vous renseigenr sur le [forum duniter](forum.duniter.org) pour savoir quelle version installer.

Vous trouverez toute les versions disponibles au téléchargement sur [cette page du gitlab](https://git.duniter.org/nodes/rust/duniter-rs/tags).

Il y a 4 types de versions :

* **Version alpha** : tout juste sortie de la phase de développement, les versions alpha peuvent présenter de nombreuses instabilités et sont destinées aux testeurs les plus aggéris (nommés alpha-testeurs).
* **Version beta** : version toujours réservée au test mais globalement fonctionnelle. Ouverte a tout les testeurs.
* **Version RC**: signifie "release candidate". Version pouvant être installée par tout les utilisateurs avancés, y compris non-testeurs, demande de suivre de près les mises a jours en cas de bug critique découvert.
* **Version stable** : Cette version n'est pas annotée. Elle est destinée a tout les utilisateurs.

### Choisir le livrable correspondant votre système d'exploitation et votre processeur

La colonne `Category` du tableau des livrables vous indique le système d'exploitation pour lequel est destiné chaque livrable. (dans le cas de linux, la distribution est indiquée entre parenthèse).

S'il n'y a pas de livrable pour votre configuration, vous pouvez installer durs manuellement (voir ci-dessous)

## Installation manuelle

Pour installer durs manuellement vous devez d'abord [installer Rust](https://www.rust-lang.org/tools/install).

Ensuite vous devez installer les dépendances nécessaire a la compilation de Durs, les indications qui suivent sont pour Debian/Ubuntu, pour les autres distributions ce sera a vous de trouver les équivalents.

Installer les paquets suivants :

    apt-get install pkg-config libssl-dev

Ensuite, clonez le dépot git :

    git clone https://git.duniter.org/nodes/rust/duniter-rs.git

Rendez vous dans le dossier `duniter-rs` ainsi créer puis dnas le sous-dossier correspondant a la variante que vous souhaitez installer :

* Pour `durs-server`, rendez-vous dans `bin/durs-server`

    cd bin/durs-server

* Pour `durs-desktop`, rendez-vous dans `bin/durs-desktop`

    cd bin/durs-desktop

Enfin lancez la compilation de Durs avec la commande suivante :

    cargo build --release --features ssl

Si vous avez des problèmes avec `openssl` lors de la compilation, vous pouvez essayer de compiler sans la feature `ssl` :

    cargo build --release

Cela implique juste que votre noeud ne pourra pas contacter les endpoint WS2P qui sont derrière une couche SSL/TLS.  
Votre noeud devrait tout de même fonctionner normalement s'il ya suffisamment de endpoint WS2P accesibles en clair.

Si la compilation réussie, votre éxécutable se trouve dans `duniter-rs/target/release` et se nomme `durs` ou  `durs-desktop`. Vous pouvez le déplacer ou bon vous semble sur votre disque puis l'éxécuter directement.