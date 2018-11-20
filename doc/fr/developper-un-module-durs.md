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

Pour développer votre module, vous devez créer une crate dans le dossier `modules/{YOUR_MODULE_NAME}`.  
Le nom de votre crate tel que décris dans le Cargo.toml devra etre préfixée par `durs-`. En revanche, le dossier dans lequel se trouvera votre module aura le nom de votre module sans préfixe.

Exemple : si vous souhaitez créer un module nommé `toto`, vous placerez la crate contenant le code de votre module dans le dossier `libs/modules/toto` et dans le Cargo.toml de votre crate vous indiquerez comme nom `durs-toto`.

### Découper un module en plusieurs crates

Si vous souhaitez découper votre module en plusieurs crates, les crates supplémentaires doivent être placés dans `modules-lib/{YOUR_MODULE_NAME}/`.  
La crate qui restera dans `modules/` doit être celle qui sera importée par les crates binaires et elle doit exposer une structure publique qui implémentera le trait `DursModule` (plus de détail plus bas).

### Développer une lib pour plusieurs modules

Si vous souhaitez développer une bibliothèque commune a plusieurs modules et utilisée exclusivement par ceux-ci, vous devrez ranger cette bibliothèque commune dans le dossier `modules-common`, ce dossier n'existe pas encore car il n'existe pas encore de groupe de modules partageant des bibliothèques communes, n'hésitez pas a le créer si le cas se présente pour vous.

Le dossier `tools/` ne doit contenir que des bibliothèques utilisées (aussi) par le coeur.

Pour résumer :

* Une bibliothèque est utilisée par le coeur et éventuellement par des modules : dossier `tools`.
* Une bibliopthèque est utilisée exclusivement par des modules : dossier `modules-common`.
* Une bibliopthèque est utilisée exclusivement par un seul module : dossier `modules-lib/{MODULE_NAME}`.

## Le module skeleton

Pour vous aider, le projet comporte un module nommé `skeleton-module` qui comporte tout ce qu'il faut a un module durs pour fonctionner.
Dans la suite de ce tutoriel, nous allons vous guider pas a pas pour créer votre module a partir d'une copie du module `skeleton-module`.

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