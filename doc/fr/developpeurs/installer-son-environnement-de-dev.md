# Installer son environnement de Développement

Date: 2018-05-11
Authors: elois

Dans ce tutoriel nous allons voir comment installer un environnement [Rust](https://www.rust-lang.org) complet.
Cela vous servira pour vos propres projets Rust, ou pour contribuer a Duniter-rs, ou pour faire du binding NodeJS-Rust.

## Installation de la toolchain stable

Installez la toolchain stable de Rust :

    curl https://sh.rustup.rs -sSf | sh

Ajoutez ~/.cargo/bin à votre variable d'environnement PATH :

    export PATH="$HOME/.cargo/bin:$PATH"

Je vous recommande vivement d'ajouter cette ligne dans le fichier de configuration de votre terminal pour ne pas avoir à la recopier a chaque fois, si vous ne savez pas de quoi je parle alors vous utilisez très probablement le shell par défaut (bash) et le fichier auquel vous devez ajouter cette ligne est `~/.bashrc`

## Fmt : le formateur de code

Je vous recommande vivement d'installer l'indispensable formateur automatique de code, d'autant qu'il est maintenu par l'équipe officielle du langage Rust donc vous avez la garantie que votre code compilera toujours (et aura toujours le même comportement) après le passage du formateur.

Pour installer `fmt` :

    rustup component add rustfmt-preview

Pour formater automatiquement votre code, placez vous à la racine de votre projet et exécutez la commande suivante :

    cargo fmt

Je vous recommande fortement de créer un alias dans la configuration de votre shell (~/.bashrc si vous utilisez bash). À titre d'exemple j'ai créé l'alias `fmt="cargo +nightly fmt"`.

## Clippy : le linteur

Si vous contribuez à l'implémentation Rust de Duniter vous devrez également utiliser le linteur Clippy. Et dans tous les cas il est vivement recommandé aux débutants en Rust de l'utiliser, en effet clippy est très pédagogique et va beaucoup vous aider à apprendre comment il convient de coder en Rust.

Exécutez la commande suivante pour installer clippy :

    rustup component add clippy-preview

Pour lancer clippy, rendez-vous à la racine de votre projet puis éxécutez la commande suivante :

    cargo clippy --all

Clippy va alors vous signaler de façon très pédagogique tout ce qu'il convient de modifier dans votre code pour être plus dans "l'esprit rust".

## IDE/Editeur

Vous aurez aussi besoin d'un environnement de développement intégré.

Rust étant un langage très récent, il n'a pas d'Environnement de Développement Intégré (IDE) dédié.
Heureusement, plusieurs IDE existants intègrent Rust via des plugins, nous vous recommandons VSCode ou IntelliJ.

Vous pouvez également développer en Rust avec les IDE/Editeurs suivants :

* VSCode
* IntelliJ Rust
* Eclipse/Corrosion
* Emacs
* VIM/Rust.vim
* Geany
* Neovim

Et bien d'autres..

## Intellij

Intellij est un excellent IDE pour Rust, si vous n'avez pas de licence vous pouvez télécharger la version communautaire gratuite : [IntelliJ IDEA Community Edition].

Ensuite extrayez l'archive dans le dossier contenant vos programmes (/opt pour les linuxiens).

    sudo tar xf idea*.tar.gz -C /opt/

Puis exécutez le script `idea.sh` dans le dossier `/bin` :

    cd /opt/idea*/bin/
    ./idea.sh

Enfin installez le plugin pour rust en tapant "Rust" dans le moteur de recherche des plugins dans file -> settings -> plugins.

[IntelliJ IDEA Community Edition]: https://www.jetbrains.com/idea/

## Vscode

[Installation de vscode pour debian/ubuntu](https://code.visualstudio.com/docs/setup/linux#_debian-and-ubuntu-based-distributions).

Une fois vscode installé nous aurons besoin des 3 plugins suivants :

* [BetterTOML](https://github.com/bungcip/better-toml)
* [CodeLLDB](https://github.com/vadimcn/vscode-lldb)
* [Rust](https://github.com/editor-rs/vscode-rust) (attention, pas "Rust (rls)")

Configuration du plugin Rust:

Dans les paramètres de l'extension Rust, vous avez le choix entre modifier les paramètres dans l'interface graphique ou directement dans le fichier `.json`. Nous indiquons ici les lignes à ajouter dans le fichier `settings.json` (qui se trouve par défaut dans `~/.config/Code/User/settings.json`).

1. Passez en mode legacy pour désactiver RLS (=Rust Language Server) qui ne fonctionne pas sur le projet durs (il monopolise inutilement 100% du CPU).

```json
"rust.mode": "legacy",
```

2. Installez racer (pour l'auto-complétion) et sym (pour aller à la définition via Ctrl+clic).

```bash
cargo +nightly install racer
cargo install rustsym
```

3. Indiquez dans le fichier `settings.json` le chemin vers racer et rustsym :

```json
"rust.racerPath": "/home/YOUR_USERuser/.cargo/bin/racer",
"rust.rustsymPath": "/home/YOUR_USERuser/.cargo/bin/rustsym"
```

4. Sauvegardez le fichier `settings.json` puis redémarrez vscode pour appliquer les changements et arrêter rls.

### VSCode : Débuggeur LLDB

[Instructions d'installation de LLDB pour vscode](https://github.com/vadimcn/vscode-lldb/wiki/Installing-on-Linux)

Sélectionnez "LLDB adapter type: native" dans les paramètres LLDB ou ajoutez dans le fichier `settings.json` 

```json
"lldb.adapterType": "native",
```

Pour paramétrer et démarrer le débugger, consultez [la doc vscode](https://code.visualstudio.com/docs/editor/debugging#_launch-configurations). Voici un exemple de fichier `launch.json` pour VSCode :

```json
{
    // Utilisez IntelliSense pour en savoir plus sur les attributs possibles.
    // Pointez pour afficher la description des attributs existants.
    // Pour plus d'informations, visitez : https://go.microsoft.com/fwlink/?linkid=830387
    "version": "0.2.0",
    "configurations": [
        {
            "name": "Debug",
            "type": "lldb",
            "request": "launch",
            "program": "${workspaceFolder}/target/debug/dunitrust",
            "cwd": "${workspaceRoot}",
            "terminal": "integrated",
            "args": ["start"],
            "env": {
                "RUST_BACKTRACE": "1"
            }
        }
    ]
}
```

### Vscode: navigation à la souris comme dans Intellij

Intellij permet par défaut de naviguer dans le code avec les touches suivant/précédent de la souris, une fonctionnalité ô combien indispensable et hélas pas implémentée dans vscode. Il existe cependant un moyen de contournement pour obtenir cette fonctionnalité.

1. Dans vscode, définissez des raccourcis clavier pour les actions `navigate back` et `navigate forward` (par exemple Ctrl+Left et Ctrl+Right).
2. Installez `xbindkeys` et `xdotool`.
3. Créez le fichier de configuration de xbindkeys à la racine de votre home avec les commandes suivantes

    cd
    xbindkeys --defaults > .xbindkeysrc

4. Ajoutez les lignes suivantes dans le fichier `~/.xbindkeysrc`

    ```bash
    ## Navigate back
    "xdotool key ctrl+Left"
            b:8

    ## Navigate forward
    "xdotool key ctrl+Right"
            b:9
    ```

5. Pour vérifier que tout est bien configuré, lancez la commande `xbindkeys -v` puis cliquez sur les touches suivant/précédent de votre souris. Vous devez voir se printer dans la console la commande correspondant au bouton sur lequel vous cliquez (`xdotool key ctrl+Left` ou `xdotool key ctrl+Right`). Vérifiez que vscode réagit comme attendu.

6. Configurez votre système pour lancer la commande `xbindkeys` au démarrage.

## Paquets supplémentaires pour compiler durs

Bien que cela soit de plus en plus rare, certaines crates rust dépendent encore de bibliothèques C/C++ et celles-ci doivent être installées sur votre ordinateur lors de la compilation. Sous Debian et dérivés, vous devez avoir `pkg-config` d'installé car le compilateur rust s'en sert pour trouver les bibliothèques C/C++ installées sur votre système.

    sudo apt-get install pkg-config

### Pour compiler la feature `ssl`

En Rust, les "features" sont des options de compilation.

Dunitrust peut être compilé avec la feature `ssl`, cela lui permet de contacter les endpoints WS2P en ws**s**://.
Par défaut les endpoints WS2P sont accesible en ws://, mais certains utilisateurs choississent de placer un reverse proxy avec une couche TLS devant leur endpoint.
Pour compiler Dunitrust avec la feature `ssl`, vous aurez besoin du paquet supplémentaire suivant :

    sudo apt-get install libssl-dev

## Tester son environnement avec un "Hello, World !"

    mkdir hello-world
    cd hello-world
    cargo init --bin

L'option `--bin` indique que vous souhaitez créer un binaire, par défaut c'est une bibliothèque qui sera créée.

Vous devriez avoir le contenu suivant dans le dossier `hello-world` :

    $ tree
    .
    ├── Cargo.toml
    ├── src
    │   └── main.rs

C'est le contenu minimal de tout projet binaire, le code source se trouve dans `main.rs`.
Tout projet Rust (binaire ou bibliothèque) doit contenir un fichier nommé Cargo.toml à la racine du projet, c'est en quelque sorte l'équivalent du `package.json` de NodeJs.

Le fichier `main.rs` contient déjà par défaut un code permettant de réaliser le traditionnel "Hello, world!" :

    fn main() {
        println!("Hello, world!");
    }

Cette syntaxe doit vous rappeler furieusement le C/C++ pour ceux qui connaissent, et c'est bien normal car Rust est conçu pour être l'un des successeurs potentiel du C++. On peut toutefois déjà noter trois différences majeures avec le C/C++ :

1. La fonction main() ne prend aucun paramètre en entrée. Les arguments cli sont capturés d'une autre façon via une utilisation de la bibliothèque standard.
2. println! n'est pas une fonction, c'est une macro. En Rust toutes les macros sont de la forme `macro_name!(params)`, c'est donc au `!` qu'on les reconnaît. Alors pourquoi une macro juste pour printer une chaîne de caractères ? Et bien parce que en Rust toute fonction doit avoir un nombre fini de paramètres et chaque paramètre doit avoir un type explicitement défini. Pour outrepasser cette limite on utilise une macro qui va créer la fonction souhaitée lors de la compilation.
3. La fonction main() ne retourne aucune valeur, lorsque votre programme se termine, Rust envoi par défaut le code EXIT_SUCCESS a l'OS. Pour interrompre votre programme en envoyant un autre code de sortie, il existe des macro comme par exemple `panic!(err_message)`

Avant de modifier le code, assurez-vous déjà que le code par défaut compile correctement :

    $ cargo build
    Compiling hello-world v0.1.0 (file:///home/elois/dev/hello-world)
    Finished dev [unoptimized + debuginfo] target(s) in 0.91 secs

Cargo est l'équivalent de npm pour Rust, il va chercher toutes les dépendances des crates (=bibliothèques) que vous installez. Oui en Rust on parle de crates pour désigner une dépendance, ça peut être une bibliothèque ou un paquet.

Si vous obtenez bien un `Finished dev [unoptimized + debuginfo] target(s) in x.xx secs`, félicitations vous venez de compiler votre premier programme Rust :)

Si vous obtenez une erreur c'est que votre environnement Rust n'est pas correctement installé, dans ce cas je vous invite à tout désinstaller et à reprendre ce tutoriel de zéro.

> Chez moi ça compile, Comment j’exécute mon programme maintenant ?

Comme ça :

    $ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.0 secs
    Running `target/debug/hello-world`
    Hello, world!

Comme indiqué, cargo run exécute votre binaire qui se trouve en réalité dans `target/debug/`

Il existe plusieurs profils de compilation, et vous pouvez même créer les vôtres, deux profils pré-configurés sont à connaître absolument :

1. Le profil `debug` : c'est le profil par défaut, le compilateur n'effectue aucune optimisation et intègre au binaire les points d'entrée permettant à un débogueur de fonctionner.
2. Le profil `release` : le compilateur effectue le maximum d'optimisation possibles et n'intègre aucun point d'entrée pour le débogueur.

Rust est réputé pour être ultra-rapide, c'est en grande partie grâce aux optimisations poussées effectuées lors d'une compilation en profil `release`, mais réaliser ces optimisations demande du temps, la compilation en mode `release` est donc bien plus longue qu'en mode `debug`.

Pour compiler en mode `release` :

    cargo build --release

Votre binaire final se trouve alors dans `target/release/`.

Pour aller plus loin, je vous invite a lire l'excellent [tutoriel Rust de Guillaume Gomez](https://blog.guillaume-gomez.fr/Rust).

Et si vous savez lire l'anglais, la référence des références que vous devez absolument lire c'est évidemment le sacro-sain [Rust Book](https://doc.rust-lang.org/book/).

Le Rust Book part vraiment de zéro et se lit très facilement même avec un faible niveau en anglais.

## Exécuter les tests automatisés de Durs

Référez vous a la section [tests automatisés](tests-auto.md).

## Alias utiles pour gagner en éfficacité

Personnellement j'utilise les alias suivants :

    alias cc="cargo fmt && cargo check"
    alias cddr="cd ~/dev/duniter/nodes/rust/duniter-rs"
    alias clip="cargo clippy"
    alias cbrf="cargo fmt && cargo build --release --manifest-path bin/dunitrust-server/Cargo.toml --features ssl"
    alias fmt="cargo fmt"
    alias tc="cargo fmt && cargo test --package"
    alias ta="cargo fmt && cargo test --all"
    alias rsup="rustup update && cargo install-update -a"
    alias dursd="./target/release/dunitrust"

Si vous utilisez bash ses alias sont a placer dans votre fichier `~/.bash_aliases` il vous faudra également décomenter la ligne incluant ce fichier dans votre `~/.bashrc`. Si vous utilisez un autre sheel, référez vous a la documentation de votre shell.

Vous pouvez évidemment renommer ces alias comme bon vous semble tant que vous vous y retrouvez.

### `cc="cargo fmt && cargo check"`

Pour scanner votre code sans builder. C'est LA commande que j'utilise le plus en développement. Elle remonte toutes les erreurs de compilation mais effectuer le build, c'est donc considérablement plus rapide que `cargo build`.
Exécutez toujours fmt avant de lancer le compilateur ! En effet il peut arriver que le compilateurt vous retourne plusieurs dizaines d'erreurs incompréhensibles juste a cause d'une simple erreur de syntaxe a un endroit qui fait que le code a été interprété d'une façon inattendu. Plutot que de perdre des heurs a chercher des erreurs qui n'existent pas, exécuter systématiquement fmt vous assurera que le compilateur reçoit toujours un code sans erreur de syntaxe et qu'il l'interprétera donc correctement.

### `cddr="cd ~/chemin/vers/le/depot/duniter-rs"`

Adaptez cet alias en fonction de la ou se trouve votre dépot sur votre poste de dev.

### clip="cargo clippy"

C'est juste plus court a taper.
Attention clippy ne vas pas rechecker les crates déjà parcourues par cargo check, vous devez modifier uen crate bas niveau (par exemple `durs-common-tools`) pour vous assurer que clippy check toutes les crates.
Lancez toujours clippy avant de pusher et corrigez tout les warning, en cas de souci avec un warning contactez un lead dev, dans certains cas très exceptionnels le lead dev pourra décider de skipper explicitement le warning en question, mais la plupart du temps il faudra le résoudre.
Rassurez vous, la CI (Continious integration) de Gitlab passera clippy sur tout le projet dans tout les cas, donc en cas d'oubli vous vous en rendrez compte.

### cbrf="cargo fmt && cargo build --release --manifest-path bin/dunitrust-server/Cargo.toml --features ssl"

Commande pour builder `dunitrust-server`. Le dépot contiendra plusieurs binaires a terme (nottament la variante durs-desktop mais pas que). Il faut donc indiquer a cargo quel binaire builder avec l'option `--manifest-path`.

De plus, pour utiliser durs vous aurez besoin de compiler en mode release, c'est long donc ne le fait que lorsqu'un `cargo check` ne vous retourne plus aucune erreur. Théoriquemetn il devrait etre possible d'utiliser durs en mode debug, c'est un probleme connu et qui sera réglé a terme ([#136](https://git.duniter.org/nodes/rust/duniter-rs/issues/136)).

Enfin, vous aurez besoin d'activer la feature ssl, elle est nécessaire pour que votre neoud durs puisse contacter les endpoint WS2P en `wss://` (l'équivalent de `https://` mais pour le protocole websocket).
La compilation de cette feature `ssl` nécessitera que vous ayez la lib opensssl pour développeurs sur votre machine.

### tc="cargo fmt && cargo test --package"

Pour exécuter les tests d'une crate en particulier. Par exemple pour exécuter les tests de la crate `dubp-user-docs` sasissez la commande suivante :

    tc dubp-user-docs

Le nom d'une crate est indiqué dans l'attribut `name` du fichier `Cargo.toml` situé a la racine de la crate en question.

Par exemple pour la crate située dans `lib/tools/user-docs`, il faut regarder le fichier `lib/tools/user-docs/Cargo.toml`.

### ta="cargo fmt && cargo test --all"

Exécute tout les tests de toutes les crates, attention c'est long !

### rsup="rustup update && cargo install-update -a"

Permet de mettre a jours toutes vos toolchains rust ainsi que tout les binaires que vous avez installer via `cargo install`.
Nécessite d'avoir installé au préalable [cargo-update](https://github.com/nabijaczleweli/cargo-update).

### dursd="./target/release/dunitrust"

Lorsque vous avez compilé `dunitrust-server` avec l'alias `cbrf`, le binaire final est un fichier exécutable qui se nomme `durs` et il se trouve dans le dossier `target/release`. Plutot que de volus déplacer dans ce dossier a chaque fois que vous souhaitez faire des tests manuels, cet alias vous permet de lancer durs en restant a la racine du dépot.

Vous pouvez évidemment renommer ces alias comme bon vous semble tant que vous vous y retrouvez.
