# Développer votre module Durs

Date: 2018-11-20
Authors: elois

Dans ce tutoriel nous allons voir comment développer un module pour [Durs](https://forum.duniter.org/t/etat-davancement-de-durs-dividende-universel-rust/4777), l'implémentation [Rust](https://www.rust-lang.org) de [Duniter](https://duniter.org).

Si ce n'est pas déjà fait, vous devez au préalable [préparer votre environnement de développement](installer-son-environnement-de-dev.md).

## Architecture générale du dépôt durs

Le dépôt durs est constitué de deux types de crates : les binaires et les bibliothèques (nommées librairies par abus de language).

Les crates binaires sont regroupés dans le dossier `bin` et sont au nomdre de deux :

* durs-server : produit un éxécutable de durs en ligne de commande, donc installable sur un serveur.
* durs-desktop : produit un éxécutable de durs en application graphique de bureau (n'existe pas encore).

Les modules durs sont des crates de type bibliothèques, vous devez donc placer la crate de votre module dans le dossier `lib`.

Le dossier `lib` est organisé en 4 sous-dossiers correspondant à 4 types de bibliothèques :

1. `tools` : les bibliothèques outils, pouvant potentiellement servir a toutes les crates.
2. `modules` : les bibliothèques représentant un module durs.
3. `modules-lib` : les bibliothèques dédiés uniquement a certain modules.
4. `core` :  les bibliothèques structurantes du coeur et de l'interphasage avec les modules.

Pour développer votre module, vous devez créer une crate dans le dossier `modules/{your-module-name}`.  
Le nom de votre crate tel que décris dans le Cargo.toml devra etre préfixée par `durs-`. En revanche, le dossier dans lequel se trouvera votre module aura le nom de votre module **sans le préfixe** `durs-`.

Exemple : si vous souhaitez créer un module nommé `toto`, vous placerez la crate contenant le code de votre module dans le dossier `lib/modules/toto` et dans le Cargo.toml de votre crate vous indiquerez comme nom `durs-toto`.

### Découper un module en plusieurs crates

Si vous souhaitez découper votre module en plusieurs crates, le dossier de votre crate principale* doit être `lib/modules/{your-module-name}/your-module-name}`. Les crates supplémentaires doivent avoir pour dossier `modules/{your-module-name}/{crate-name}` et leur nom doit être préfixé par `{your-module-name}-`.

Exemple : vous souhaitez déplacer une partie du code de votre module toto dans une nouvelle crate `tata`. Vous devrez déplacer votre module `toto` dans `lib/modules/toto/toto` et créer votre module tata dans `lib/modules/toto/toto-tata`. De plus, votre nouvelle doit déclarer dans sont Cargo.toml le nom `durs-toto-tata`.

Régle générale : le dossier d'une crate doit avoir le même nom que la crate mais **sans le préfixe** `durs-`.

\* La crate principale doit être celle qui sera importée par les crates binaires et elle doit exposer une structure publique qui implémentera le trait `DursModule` (plus de détail plus bas).

### Développer une lib pour plusieurs modules

Si vous souhaitez développer une bibliothèque commune a plusieurs modules et utilisée exclusivement par ceux-ci, vous devrez ranger cette bibliothèque commune dans le dossier `modules-common`, ce dossier n'existe pas encore car il n'existe pas encore de groupe de modules partageant des bibliothèques communes, n'hésitez pas a le créer si le cas se présente pour vous.

Le dossier `tools/` ne doit contenir que des bibliothèques utilisées (aussi) par le coeur.

Pour résumer :

* Une bibliothèque est utilisée par le coeur et éventuellement par des modules : dossier `tools`.
* Une bibliopthèque est utilisée exclusivement par des modules : dossier `modules-common`.
* Une bibliopthèque est utilisée exclusivement par un seul module : dossier `modules-lib/{MODULE_NAME}`.

## Le module skeleton

Pour vous aider, le projet comporte un module nommé `skeleton` qui comporte tout ce qu'il faut a un module durs pour fonctionner.
Dans la suite de ce tutoriel, nous allons vous guider pas a pas pour créer votre module a partir d'une copie du module `skeleton`.

Le module skeleton contient de morceaux de code permettant d'exemplifier les différentes usages courant, comme modifier la configuration de votre modules ou découper votre module en plusieurs threads, certains usages ne vous serviront peut-être pas et vous pourrez donc purement supprimer le code afférant dans votre copie.

Evidemment, le module skeleton contient également quelques ingrédients indispensable au fonctionnement de tout module durs, et c'est par ceux-ci que nous allons commencer.

## Le trait `DursModule`

La seule obligation que vous devez respecter pour que votre module soit reconnu par le coeur est d'exposer une structure publique qui implémente le trait `DursModule`.

Ensuite vous n'avez plus qu'a modifier les crates binaires pour qu'elle importent votre structure qui implémente le trait `DursModule`. (La modification des binaire ssera détaillée plus loin).

Les traits sont au coeur du langage Rust, on les utilises partout et tout le temps. Ils ressemblent au concept d'interfaces que l'on peut trouver dans d'autres langages.
Un trait défini un **comportement**, il expose effectivement des méthodes un peu comme les interfaces ainsi que des traits parents rapellant le concept d'héritage mais un trait peut également exposer des types, et c'est d'ailleurs le cas du trait `DursModule`.

Le trait `DursModule` expose 2 types que vous devrez donc définir dans votre module : `ModuleConf` et `ModuleOpt`.

### Le type `ModuleConf`

Type représentant la configuration de votre module. Si votre module n'a pas de configuration vous pouvez créer un type structure vide.

Exemple de structure vide :

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Skeleton Module Configuration
pub struct YourModuleConf {}

impl Default for YourModuleConf {
    fn default() -> Self {
        YourModuleConf {}
    }
}
```

Le type `ModuleConf` doit lui même implémenter toute une série de trait permettant la gestion automatisé de votre configuration par le coeur. Heureusement beaucoup de traits peuvent être implémentés automatiquement par le compilateur gràçe a la macro `#[derive(Trait1, ..., TraitN)]`.  
Le seul trait que vous devrez implémenter manuellement est le trait `Default`, il expose une seule fonction `default()` qui sera utilisée par le coeur pour générer la configuration par défaut de votre module.

Le module skeleton donne un exemple de configuration avec un champ de type `String` nommé `test_fake_conf_field`. Ce champ permet d'alimenter le code d'exemple de modification de la configuration.

### Le type `ModuleOpt`

Type représentant les options de ligne de commande pour votre module. Si votre module n'a pas de commande cli (ou une commande cli sans aucun option) vous pouvez créer un type structure vide.

Exemple de structure vide :

```rust
#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "skeleton",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
/// YourModule subcommand options
pub struct YourModuleOpt {}
```

Le module skeleton donne un exemple de `ModuleOpt` avec un champ, cela afin de montrer comment fonctionne l'éxécution d'une sous-commande injectée par un module.

### Les fonctions du trait `DursModule`

Le trait `DursModule` expose 6 fonctions dont 4 doivent obligatoirement être implémentés par votre modules. Les fonctions optionnelles sont celles disposant d'une implémentation par défaut, vous pouvez évidemment les réimplémenter si besoin.

Voici un résumé des 6 fonctions avant de nous plogner dans le détail de chacune d'elles.

Les 4 fonctions obligatoires :

* `name` : Dopit retourner le nom de votre module
* `priority` : doit indiquer si votre module est obligatoire ou optionnel et s'il est activé par défaut.
* `ask_required_keys` : doit indiquer de quelles clés cryptographiques votre module a besoin.
* `start` : Fonction appelée dans un thread dédié quand le noeud durs est lancé (seulement si votre module est activé). C'est un quelque sorte le "main" de votre module.

Les 2 fonctions optionelles concernent uniquement la ligne de commande :

* `have_subcommand` : retourne un booléen indiquant si votre module doit injecter une sous commande a la commande durs. L'implémentation par défaut retourne `false`.
* `exec_subcommand` : fonction appelée quand l'utilisateur a saisi a sous-commande de votre module. L'implémentation par défaut ne fait rien.

#### La fonction `name`

Déclaration :

```rust
    /// Returns the module name
    fn name() -> ModuleStaticName;
```

`ModuleStaticName` est une tuple struct contenant un seul élément de type `&'static str`. C'est une pratique courante en Rust que d'encapsuler des types standards dans des tuple struct pour manipuler des types pus expressifs. Notez bien qu'il s'agit d'une abstraction sans cout car le compilateur désencapsulera tout vos types dans le binaire final, donc vous pouvez abuser des tuples struct a volonté, c'est considéré comme une bonne pratique. Si vous ne connaissez pas le type `&'static str` je vous renvoie au [Rust Book](https://doc.rust-lang.org/book/second-edition/ch10-03-lifetime-syntax.html#the-static-lifetime).

Dans la pratique, le type `ModuleStaticName` est vraiment très simple a utiliser, si votre module se nomme `toto` vous pouvez écrire :

```rust
    fn name() -> ModuleStaticName {
        ModuleStaticName("toto")
    }
```

Toutefois, vous aurez probablement besoin du nom de votre module a plusieurs endroits dans le code, la bonne pratique consiste donc a créer une constante globale :

```rust
static MODULE_NAME: &'static str = "toto";
```

Puis a remplacer dans l'implémentation de la fonction `name` :

```rust
    fn name() -> ModuleStaticName {
        ModuleStaticName(MODULE_NAME)
    }
```

#### La fonction `priority`

Déclaration :

```rust
    /// Returns the module priority
    fn priority() -> ModulePriority;
```

Les différents niveaux de priorité possibles sont présentés dans la [documentation auto-générée](https://nodes.duniter.io/rust/duniter-rs/duniter_module/enum.ModulePriority.html).

Il suffit de choisir la variante de l'énumération qui vous conviens puis de la retournée. Par exemple si votre module est optionel et désactivé par défaut :

```rust
    fn priority() -> ModulePriority {
        ModulePriority::Optional()
    }
```

#### La fonction `ask_required_keys`

Déclaration :

```rust
    /// Indicates which keys the module needs
    fn ask_required_keys() -> RequiredKeys;
```

Toutes les variantes de l'énumération `RequiredKeys` sont présentés dans la [documentation auto-générée](https://nodes.duniter.io/rust/duniter-rs/duniter_module/enum.RequiredKeys.html).

Il suffit de choisir la variante de l'énumération qui vous conviens puis de la retournée. PAr exemple si vous n'avez besoin d'aucune clé :

```rust
    fn ask_required_keys() -> RequiredKeys {
        RequiredKeys::None()
    }
```

#### La fonction `have_subcommand`

Déclaration :

```rust
    /// Define if module have a cli subcommand
    fn have_subcommand() -> bool {
        false
    }
```

L'implémentation par défaut retourne `false`. Si vous avez une sous commande il vous suffit de réimplémenter la fonction en retournant true.

#### La fonction `exec_subcommand`

Déclaration :

```rust
    /// Execute injected subcommand
    fn exec_subcommand(
        soft_meta_datas: &SoftwareMetaDatas<DC>,
        keys: RequiredKeysContent,
        module_conf: Self::ModuleConf,
        subcommand_args: Self::ModuleOpt,
    ) {
    }
```

Tutoriel en cours de rédaction...

#### La fonction `start`

Déclaration :

```rust
    /// Launch the module
    fn start(
        soft_meta_datas: &SoftwareMetaDatas<DC>,
        keys: RequiredKeysContent,
        module_conf: Self::ModuleConf,
        main_sender: mpsc::Sender<RouterThreadMessage<M>>,
        load_conf_only: bool,
    ) -> Result<(), ModuleInitError>;
```

Dans le cas d'un module qui ne servirait qu'a ajouter une sous-commande a la ligne de commande Durs, l'implémentation de la fonction start reste oblogatoire et le module doit absolument s'enregistrer auprès du router quand même et garder son thread actif jusqu'a la fin du programme. J'ai ouvert [un ticket](https://git.duniter.org/nodes/rust/duniter-rs/issues/112) pour améliorer cela.

La 1ère chose a faire dans votre fonction start est de vérifier l'intégrité et la cohérence de la configuration de votre module.  
Si vous détectez la moindre erreur dans la configuration de votrre module, vous devez interrompre le programme avec un message d'erreur indiquand clairement le nom de votre module et le fait que la configuration est incorrecte.

Ensuite si `load_conf_only` vaut `true` vous n'avez plus rien a faire, retournez `Ok(())`.
En revanche, si `load_conf_only` vaut `false` c'est qu'il vous faut réellement lancer votre module, cela se fera en plusieurs étapes détaillées plus bas :

1. Créez votre channel

2. Créer vos endpoint s'il y a lieu

3. Enregistrez votre module auprès du router

4. Faites les traitements que vous devez faire avant votre main loop (ça peut être rien si votre module est petit).

5. Lancez votre main loop au sein de laquelle vous écouterez les messages arrivant dans votre channel.

Si jamais le router n'a pas reçu l'enregistrement de tout les modules au bout de 20 secondes, il interromp de programme.  
Le plus important est donc d'enregistrer votre module auprès du router AVANT tout traitement lourd ou couteux.
20 secondes peut vous sembler énorme, mais garder en tête que Durs peut être amené a s'éxcuter dans n'importequel contexte, y compris sur un micro-pc aux performances très très réduites. De plus, Durs n'est pas seul sur la machine de l'utilisateur final, le délai de 20 secondes doit être respecté même dans le pire des scénarios (micro-pc déjà très occupé a d'autres taches).

Si vous prévoyez de réaliser des traitements lours ou/et couteux dans votre module, il peut être pertinent de ne pas l'inclure dans la release pour micro-pc (architecture arm), n'hésitez pas a poser la question aux développeurs principaux du projet en cas de doute.  
En gros, lorsque votre poste de développement ne fait rien de couteux en même temps, votre module doit s'être enregistré en moins de 3 secondes, si ça dépasse c'est que vous faites trop de choses a l'initialisation.

## Injecter votre module dans les crates binaires

Tout d'abord, il faut ajouter votre module aux dépendances des crates binaires. Les dépendances d'une crate sont déclarées dans son fichier `Cargo.toml`.

Par exemple, pour ajouter le module `toto` a la crate binaire `durs-server` il faut ajouter la ligne suivante dans la section `[dependencies]` du fichier `bin/durs-server/Cargo.toml` :

    durs-toto = { path = "../../lib/modules/toto" }

Vous pouvez modifier une copie de la ligne du module skeleton pour être sûr de ne pas vous tromper.

### Injecter votre module dans `durs-server`

Une fois que vous avez ajouter votre module en dépendance dans le Carego.toml de `durs-server`, il vas falloir utiliser votre module dans le main.rs :

1. Utilisez votre structure implémentant le trait DursModule :

    pub use durs_toto::TotoModule;

2. Ajouter votre module en paramètre de la macro `durs_plug!` :

    durs_plug!([WS2PModule], [TuiModule, .., TotoModule])

    Notez que `durs_plug!` prend en paramètre 2 tableaux de modules, le 1er correspond aux modules de type réseau inter-noeuds, tout les autres modules doivent se trouver dans le 2ème tableau.

3. Si votre module doit injecter une sous-commande dans la ligne de commande `durs`, ajoutez le également a la macro `durs_inject_cli!` :

    durs_inject_cli![WS2PModule, .., TotoModule],

    La macro `durs_inject_cli!` n'accepte qu'un seul tableau qui doit comporter tout les modules injectant une sous-commande, pas de distinction ici.

    Notez que votre module doit DANS TOUT LES CAS être ajouté a la macro `durs_plug!` sinon sa sous-commande ne fonctionnera pas.

Tutoriel en cours de rédaction...