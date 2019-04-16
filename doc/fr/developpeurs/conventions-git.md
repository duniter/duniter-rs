# Conventions git du projet Durs

## Nommage des branches

### Branche créée par Gitlab

Le plus souvent, votre branche sera nommée automatiquement par Gitlab puisque vous êtes censé créer votre branche en cliquant sur le bouton "Create a merge request" sur l'issue liée.
Dans ce cas vous devez préfixer la branche par votre pseudo Gitlab suivi d'un slash, exemple :

    elois/2-test-de-ticket

### Branche créée manuellement

Dans tous les autres cas, votre branche doit impérativement commencer par le pseudo de votre compte Gitlab afin que tout un chacun sache qui travaille sur cette branche. Voici la convention à respecter pour les branches que vous créez manuellement :

    pseudo/type/description

pseudo := pseudo de votre compte Gitlab.
type := voir "Liste des types de commit".
description := courte description en anglais à l'impératif présent, 3 à 4 mots maximum, pas d'articles.

Exemple :

    elois/ref/rename_module_trait

## Nommage des commits

Chaque commit doit suivre la convention suivante :

    [type] crate: action subject

Le type doit être un mot clé de type parmi la liste des types de commit.

La crate doit être le nom de la crate concernée par le commit sans le préfixe `durs-`.

L'action doit être un verbe à l'impératif et le sujet un nom.

Exemple, renommage d'un trait `Toto` en `Titi` dans la crate `durs-bidule` :

    [ref] bidule: rename Toto -> Titi

### Liste des types de commit

* `build` : Modification des scripts de build, de packaging ou/et de publication des livrables.
* `ci` : Modification de la chaîne d'intégration continue.
* `deps` : Modification des dépendances sans modification du code : ce peut être pour mettre à jour des dépendances tierces ou pour supprimer des dépendances tierces qui ne sont plus utilisées.
* `docs` : Modification de la documentation (y compris traduction et création de nouveau contenu).
* `feat` : Développement d'une nouvelle fonctionnalité.
* `fix` : Correction d'un bug
* `opti` :  Optimisation : amélioration des performances ou/et réduction de l'espace mémoire/disque utilisé.
* `pub` : commit lié a la publication d'une crate sur [crates.io](https://crates.io).
* `ref` : Changement du code qui ne change rien au fonctionnement (refactoring en anglais).
* `style` : Modification du style du code (fmt et clippy).
* `tests` : Modification des tests existants ou/et création de nouveaux tests.

Si vous avez besoin d'effectuer une action qui ne rentre dans aucun de ses types, contactez les principaux développeurs du projet pour discuter de l'ajout d'un nouveau type de commit dans cette liste.

## Stratégie de mise à jour

On met à jour uniquement avec des rebase, les merge sont strictement interdits !

Chaque fois que la branche `dev` est mise à jour, vous devez rebaser chacune de vos branches de travail sur dev. Pour chaque branche :

1. Placez-vous sur votre branche
2. Lancez un rebase sur dev

    git rebase dev

3. Réglez les conflits s'il y en a. Une fois les conflits résolus vous devez :
    a. commiter les fichiers qui étaient en conflit
    b. Continuer le rebase avec la commande `git rebase --continue`
    c. Refaire 3. pour chaque commit où il y a des conflits

4. Vous n'avez plus de conflits après un `git rebase --continue`, c'est que le rebase est terminé. Passez à la branche suivante.

Si quelque chose s'est mal passé et que vous ne savez plus où vous en êtes, vous pouvez annuler votre rebase et reprendre de zéro avec la commande `git rebase --abort`.

Il se peut que vous n'ayez pas de conflits du tout, dans ce cas vous sautez directement de l'étape 2. à 4. sans passer par 3.

## Quand pusher

Idéalement à chaque fois que vous êtes sur le point d'éteindre votre ordinateur, soit environ une fois par jour (uniquement pour les jours où vous codez sur le projet bien sûr).

Pensez bien à préfixer votre commit par `wip:` pour indiquer que c'est un "work in progress".

> Pourquoi pusher alors que je n'ai pas fini ?

Si votre ordinateur rencontre un problème (panne, perte de données, reformatage, etc), pusher vous permet de vous assurer d'avoir toujours une copie de votre travail quelque part sur les internets.

## Comment merger ma contribution

Lorsque vous avez fini votre développement, exécutez `fmt` et `clippy` pour être sûr que votre code est propre puis exécutez tous les tests pour être sûr qu'ils passent :

    cargo +nightly fmt
    cargo +nightly clippy
    cargo test --all

Puis commitez le tout, sans le préfix wip- cette fois ci.

Ensuite nettoyez l'historique de votre branche avec un rebase interactif :

    git rebase -i dev

Renommez notamment les commits `wip:` et fusionnez les commits liés à fmt ou à clippy afin de simplifier l'historique.

Enfin faites un `push force` sur le dépot distant :

    git push -f

Puis, rendez-vous sur le Gitlab et vérifiez que le code sur votre branche distante est bien celui censé s'y trouver.

Attendez 20 minutes que la chaîne d'intégration continue puisse vérifier votre code, et si elle réussit vous pouvez alors supprimer la mention WIP de votre Merge Request et tagger des développeurs expérimentés pour demander une revue de code.
