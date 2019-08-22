# Installer Dunitrust sur votre ordinateur

## Installation simple

Dans tout les cas vous aurez 3 choix a faire :

1. Choisir entre dunitrust-server ou dunitrust-desktop
2. Choisir la version de Dunitrust que vous souhaitez installer
3. Choisir le livrable correspondant à votre système d'exploitation et votre processeur.

### `dunitrust-server` ou `dunitrust-desktop`

`dunitrust-desktop` est destiné aux utilisateurs souhaitant installer Dunitrust sur leur ordinateur personnel et administrer leur noeud Dunitrust via une interface graphique.

`dunitrust-server` est beaucoup plus léger et se manipule via la ligne de commande. Il est notamment utile dans les cas suivants :

* Installation de durs sur serveur dédié
* Installation de durs sur micro pc (raspberry pi, brique internet, etc)
* Pour les utilisateurs avancé qui préfèrent la ligne de commande.

Notez bien : il est possible d'administrer `dunitrust-server` a distance via une interface graphique (voir [administrer un noeud durs a distance]).

### Choisir la version de Dunitrust a installer

<s>Rendez vous sur [le site officiel de Dunitrust](dunitrust.org), vous y trouverez un lien direct vers la dernière version stable.</s>

Le site web de Dunitrust n'existe pas encore, en attendant vous devrez vous renseigner sur le [forum duniter](https://forum.duniter.org) pour savoir quelle version installer.

Vous trouverez toute les versions disponibles au téléchargement sur [cette page du gitlab](https://git.duniter.org/nodes/rust/duniter-rs/tags).

Il y a 4 types de versions :

* **Version alpha** : tout juste sortie de la phase de développement, les versions alpha peuvent présenter de nombreuses instabilités et sont destinées aux testeurs les plus aggéris (nommés alpha-testeurs).
* **Version beta** : version toujours réservée au test mais globalement fonctionnelle. Ouverte a tout les testeurs.
* **Version RC**: signifie "release candidate". Version pouvant être installée par tout les utilisateurs avancés, y compris non-testeurs, demande de suivre de près les mises a jours en cas de bug critique découvert.
* **Version stable** : Cette version n'est pas annotée. Elle est destinée a tout les utilisateurs.

### Choisir le livrable correspondant votre système d'exploitation et votre processeur

La colonne `Category` du tableau des livrables vous indique le système d'exploitation pour lequel est destiné chaque livrable. (dans le cas de linux, la distribution est indiquée entre parenthèse).

S'il n'y a pas de livrable pour votre configuration, vous pouvez installer durs manuellement (voir ci-dessous)

## Installation via Docker

Téléchargez l'image docker de durs :

    docker pull dunitrust/dunitrust

Sans préciser de tag, vous obtiendrez la dernière version stable.

*Note temporaire (05-2019) : Il n'existe pas encore de version stable. Rajoutez le tag `dev` pour télécharger la dernière version de développement*

Pour les alpha-testeurs, vous pouvez télécharger la dernière version de développement avec le tag `dev` :

    docker pull dunitrust/dunitrust:dev

Ensuite configurez votre noeud durs via un fichier de variables d'environnement

Vous devrez nottament définir la variable d'environnement DURS_SYNC_URL qui indiquera a durs sur quel url il devra se synchroniser au démarrage.

Enfin lancez votre conteneur Dunitrust comme suit :

    docker run -it --env-file path/to/your/env/file --name durs registry.duniter.org/nodes/rust/duniter-rs:TAG

### Externaliser les données utilisateur (config, bases de données, logs, trousseaux de clés)

Vous pouvez externaliser les données utilisateurs en montant un volume dans /var/lib/dunitrust, via l'option `-v`  de `docker run`.

L'option `-v` de la commande `docker run` indique quel dossier de la machine hôte doit etre monté dans le conteneur et a quel endroint. La syntaxe générale est `-v HOST_PATH:CONTAINER_PATH`.

il faut alors indiquer a durs que vous souhaitez stocker les données dans `/var/lib/dunitrust` via l'option `--profiles-path`

Exemple, pour stocker les donnes dans le dossier `/home/you/dunitrust-datas` de votre machien hôte :

    docker run -it -v /home/you/dunitrust-datas:/var/lib/dunitrust registry.duniter.org/nodes/rust/duniter-rs:TAG durs --profiles-path /var/lib/dunitrust

Astuce : vous pourrez alors injecter un trousseau de clé personnalisé dans `/home/you/dunitrust-datas/default/keypairs.json`.

### Docker secrets

Pour les utilisateurs souhaitent injecter leur trousseau de clé dans le conteneur via un secret docker, utilisez l'option `--keypairs-file` pour indiquer a durs ou aller chercher le secret. A noté que votre secret devra etre une chaine de caractère JSON du même format que le fichier `keypairs.json`.

## Installation manuelle

Pour installer durs manuellement vous devez d'abord [installer Rust](https://www.rust-lang.org/tools/install).

Ensuite vous devez installer les dépendances nécessaire a la compilation de Durs, les indications qui suivent sont pour Debian/Ubuntu, pour les autres distributions ce sera a vous de trouver les équivalents.

Installer les paquets suivants :

    apt-get install pkg-config libssl-dev

Ensuite, clonez le dépot git :

    git clone https://git.duniter.org/nodes/rust/duniter-rs.git

Rendez vous dans le dossier `duniter-rs` ainsi créé puis dans le sous-dossier correspondant à la variante que vous souhaitez installer :

* Pour `dunitrust-server`, rendez-vous dans `bin/dunitrust-server`

    cd bin/dunitrust-server

* Pour `dunitrust-desktop`, rendez-vous dans `bin/dunitrust-desktop`

    cd bin/dunitrust-desktop

Enfin lancez la compilation de Dunitrust avec la commande suivante :

    cargo build --release --features ssl

Si vous avez des problèmes avec `openssl` lors de la compilation, vous pouvez essayer de compiler sans la feature `ssl` :

    cargo build --release

Cela implique juste que votre noeud ne pourra pas contacter les endpoint WS2P qui sont derrière une couche SSL/TLS.  
Votre noeud devrait tout de même fonctionner normalement s'il ya suffisamment de endpoint WS2P accesibles en clair.

Si la compilation réussie, votre exécutable se trouve dans `duniter-rs/target/release` et se nomme `durs` ou `dunitrust-desktop`.
Vous pouvez le déplacer ou bon vous semble sur votre disque puis l'exécuter directement.
