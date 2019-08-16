# Tests automatisés de Durs

Date: 2019-05-18
Authors: elois

Tout programme Rust peut mettre en place 3 type's de tests automatisés :

1. Les tests unitaires (souvent nommés `TU`).
2. Les tests d'intégration (souvent nommés `TI`).
3. Les tests de performance (souvent nommmés `benchs`).

Les TU se trouvent dans le même fichier que le code source testé, ils testent généralement une fonvtion publique d'un module rust.
Les TI se trouvent dans un dossier `tests` a la racine de la crate testée. Chaque TI est compilé comme une crate a part. Les TI vont tester une fonctionnalité précise en simulant un contexte d'éxécution pour la crate testée.

Pour plus de précisions sur les TU et TI en Rust reportez vous au [chapitre dédiée dans le rust book](https://doc.rust-lang.org/book/ch11-03-test-organization.html).

Les `benchs` se trouvent dans un dossier `benchs` a la racine de la crate. Tout les détails se trouvent dans [ce chapitre d'une ancienne* version du rust book](https://doc.rust-lang.org/1.6.0/book/benchmark-tests.html).

*Les informations sur les `benchs` y sont toujours valables.

Le projet durs a surtout mis en place beaucoup de TU. De nombreux TI seront implémentez dnas les mois a venir, cela fait partit des chantiers en cours.

## Lancez les tests d'une crate en particulier

Pour exécutez les tests (TU+TI) d'une crate en particulier :

    cargo test --package CRATE_NAME

Par exemple pour exécuter les tests (TU+TI) de la crate dubp-user-docs:

    cargo test --package dubp-user-docs

Le nom d'une crate est indiqué dans l'attribut `name` du fichier `Cargo.toml` situé a la racine de la crate en question.

Par exemple pour la crate située dans `lib/tools/user-docs`, il faut regarder le fichier `lib/tools/user-docs/Cargo.toml`.

## Lancer tout les tests du projet

    cargo test --all

Attention c'est long !

Pour gagner du temps vous pouvez vous contenter de lancer les tests des seules crates que vous avez modifier.
De toute façon la CI Gitlab lancera tout les tests du projet, donc en cas de regression vous vous en rendrez compte après avoir pushé.

De plus, le Gitlab est configuré de tel façon a ce qu'il soit impossible d'accepter votre Merge Request si tout les tests ne passent pas, vous avez donc la garantie de ne pas intégrer de regression dans la branche principale (sauf si la regression correspond a un cas non couvert par les tests automatisés).

## Vérifier la couverture des tests

Afin d'éviter les regressions, les tests doivent couvrir le plus de cas possible (idéalement tous mais il est impossible de penser a tout les cas).
Pour vérifier la couverture des tests d'une crate, **(en cours de rédaction)**.

L'intégration d'un outils de coverage au projets est un chantier en cours, les outils Rust dédiés au coverage arrivent tout juste a maturité et devrait donc etre intégrés au projet Dunitrust dans les prochains mois.
