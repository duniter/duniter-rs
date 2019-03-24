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

Clippy va alors vous signaler de façopn très pédagogique tout ce qu'il convient de modifier dans votre code pour être plus dans "l'esprit rust".

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

Puis éxécutez le script `idea.sh` dans le dossier `/bin` :

    cd /opt/idea*/bin/
    ./idea.sh

Enfin installez le plugin pour rust en tapant "Rust" dans le moteur de recherche des plugins dans file -> settings -> plugins.

[IntelliJ IDEA Community Edition]: https://www.jetbrains.com/idea/

## Vscode

[Installation de vscode pour debian/ubuntu](https://code.visualstudio.com/docs/setup/linux#_debian-and-ubuntu-based-distributions).

Une fois vscode installé nous aurons besoin des 3 plugins suivants :

* BetterTOML
* CodeLLDB
* Rust (rls)

Après avoir installé les plugins, relancez votre IDE, il devrait vous proposer spontanément d'installer RLS, dites oui.
Si cela échoue pour RLS, vous devrez l'installer manuellement avec la commande suivante :

    rustup component add rls-preview rust-analysis rust-src

### Débugger LLDB pour VSCode

[Instructions d'installation de LLDB pour vscode](https://github.com/vadimcn/vscode-lldb/wiki/Installing-on-Linux)

Ensuite relancez votre IDE.

Un exemple de fichier `launch.conf` pour VSCode :

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
            "program": "${workspaceFolder}/target/debug/durs",
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

## Paquets supplémentaires pour compiler duniter-rs

Bien que cela soit de plus en plus rare, certaines crates rust dépendent encore de bibliothèques C/C++ et celles-ci doivent être installées sur votre ordinateur lors de la compilation. Sous Debian et dérivés, vous devez avoir `pkg-config` d'installé car le compilateur rust s'en sert pour trouver les bibliothèques C/C++ installées sur votre système.

    sudo apt-get install pkg-config

### Pour compiler la feature `ssl`

En Rust, les "features" sont des options de compilation.

Durs peut être compilé avec la feature `ssl`, cela lui permet de contacter les endpoints WS2P en ws**s**://.
Par défaut les endpoints WS2P sont accesible en ws://, mais certains utilisateurs choississent de placer un reverse proxy avec une couche TLS devant leur endpoint.
Pour compiler Durs avec la feature `ssl`, vous aurez besoin du paquet supplémentaire suivant :

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
