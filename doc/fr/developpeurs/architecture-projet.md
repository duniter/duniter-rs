# Architecture générale du projet

## Dossiers racine

Le dépôt Dunitrust est constitué de deux types de crates : les binaires et les bibliothèques (nommées librairies par abus de langage).
Les crates de type binaire se trouvent dans le dossier `bin` et les crates de type bibliothèque se trouvent dans le dossier `lib`.

Toutefois, le dépôt ne contient pas que du code Rust, voici a quoi sert chaque dossier :

`.github` : contient des modèles pour les formulaires de création d'isues et de PR sur github afin d'indiquer qu'il ne s'agit que d'un dépot mirroir.
`.gitlab` : contient des script python pour automatiser la publication des releases sur une page wiki du gitlab.
`bin` : contient les crates binaires, plus de détail dans la section dédiée a ce dossier.
`doc` : contient la documentation pour les utilisateur et les testeurs ainsi qu'une documentation haut-niveau pour les développeurs (Il existe aussi une documentation bas-niveau auto-générée a partir du code source).
`images` : contient les images du projet. Certaines sont intégrées dans les binaires (comme le logo par exemple), d'autres sont intégrés sur des pages de la documentation.
`lib` : contient les crates bibliothèques, plus de détail dans la section dédiée a ce dossier.
`release` : contient les scripts permettant de construire et empaqueter les livrables.

## Dossier `bin`

Les crates binaires sont regroupés dans le dossier `bin` et sont au nombre de deux :

* dunitrust-server : produit un exécutable de Dunitrust en ligne de commande, donc installable sur un serveur.
* dunitrust-desktop : produit un exécutable de Dunitrust en application graphique de bureau (n'existe pas encore).

## Dossier `lib`

Le dossier `lib` est organisé en 8 sous-dossiers correspondant à 8 types de bibliothèques :

1. `core` :  les bibliothèques structurantes du cœur et de l'interphasage avec les modules.
2. `crypto` : les bibliothèques fournissant les fonctionnalités cryptographiques.
3. `dubp` : les bibliothèques définissant les structures du protocole DU**B**P (DUniter **Blockchain** Protocol) ainsi que des méthodes pour manipuler ces structures.
4. `dunp` : les bibliothèques définissant les structures du protocole DU**N**P (DUniter **Network** Protocol) ainsi que des méthodes pour manipuler ces structures.
5. `modules` : les bibliothèques représentant un module Dunitrust.
6. `modules-lib` : les bibliothèques dédiées uniquement à certains modules.
7. `tests-tools` : les bibliothèques fournissant du code utilisé uniquement par les tests automatisés.
8. `tools` : les bibliothèques outils, pouvant potentiellement servir à toutes les crates. Ces bibliothèques ne doivent comporter aucune logique métier spécifique à Duniter/Dunitrust (dit autrement elles doivent être théoriquement externalisables).
