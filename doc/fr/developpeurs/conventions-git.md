# Conventions git du projet Durs

## TL;DR résumé de cette page, instructions pour bien travailler

Les points abordés dans cette page sont résumés ici pour vous permettre d'avoir un aperçu global des règles à suivre avant de les lire en détail.

- pour une branche créée suite à une issue, préfixer le nom de la branche par votre pseudo
- pour une branche créée manuellement, respecter le format `pseudo/type/description`
- le travail "au brouillon" doit être signalé par un "WIP" (Work In Progress)
- le nommage des commits finaux doit respecter le format `[type] crate: action subject`
- communiquer avec les développeurs via les espaces dédiés
- l'intégration d'une contribution se fait uniquement par rebase (merge interdit) et une fois les critères suivants remplis
    - branche à jour sur dev
    - formatage canonique du code, tests automatisés passés avec succès
    - historique des commits propre, compréhensible et concis
    - contribution approuvée par un reviewer


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

Une merge request ne doit concerner qu'un seul sujet.

## Nommage des commits

Chaque commit doit suivre la convention suivante :

    [type] crate: action subject

Le **type** doit être un mot clé de type parmi la liste des types de commit.

La **crate** doit être le nom de la crate concernée par le commit sans le préfixe `durs-`.

L'**action** doit être un verbe à l'impératif et le sujet un nom.

Exemple, renommage d'un trait `Toto` en `Titi` dans la crate `durs-bidule` :

    [ref] bidule: rename Toto -> Titi

Le nom de commit doit être entièrement en minuscules. Le nom du commit est de préférence en anglais. Dans le cas de commits en français l'utilisation d'accents et à éviter (certains moteurs git les gèrent mal, notamment sous windows).

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

Le nom du commit doit être parlant dans le seul contexte de la lecture de l'historique, il ne doit donc pas faire référence à une MR ou à des discussions en particulier.
L'historique des commits est notamment utilisé pour publier le changelog entre deux versions ainsi que pour faire le bilan du travail accompli, c'est pourquoi il doit se suffire a lui-même.
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


## À vérifier avant de demander la relecture de votre "merge request"

Après avoir rempli les critères ci-dessus dans vos commits, vous devez vérifier que votre branche est bien à jour par rapport à la branche cible (`dev` dans cet exemple). Comme cette branche avance fréquemment, il est possible que de nouveaux commits aient eu lieu pendant que vous travailliez sur votre branche (nommée VOTRE_BRANCHE, ici). Si c'est le cas ou en cas de doute, pour mettre à jour votre branche par rapport à `dev`, faites comme suit :

  git checkout dev          # basculer sur la branche dev
  git pull                  # mettre à jour la branche dev par rapport au dépôt distant
  git checkout VOTRE_BRANCHE # basculer à nouveau sur votre branche
  git rebase dev            # prendre dev comme nouvelle base pour votre branche

En cas de conflits pendant le rebase que vous n'arrivez pas à résoudre, il faut contacter un lead dev en lui indiquant le hash du commit sur lequel se base VOTRE_BRANCHE au moment du rebase pour qu'il puisse reproduire le rebase et voir les conflits en question. En attendant sa réponse, vous pouvez annuler le rebase et travailler sur VOTRE_BRANCHE sans se mettre a jour :

  git rebase --abort

Il est préférable de prendre son temps avant d'intégrer une nouvelle contribution car l'historique de la branche dev n'est pas modifiable : c'est une branche protégée. Chaque commit sur cette branche y reste donc *ad vitam aeternam* c'est pourquoi l'on veille à garder un historique des commits clair et compréhensible.

## Discussion dans une merge request

Sur Gitlab, une discussion est ouverte pour chaque merge request. Elle vous permettra de discuter des changements que vous avez faits. N'hésitez pas à identifier quelqu'un en écrivant @pseudo pour qu'il soit notifié de votre demande. Ne vous impatientez pas, la relecture de votre contribution peut prendre plus ou moins de temps en fonction de son contenu !

La discussion générale sert à commenter la merge request dans son ensemble, par exemple pour tagger un développeur pour une demande de relecture. Quand il s'agit de discuter un changement précis dans le code, il faut se rendre dans l'onglet "Changes" de la merge request et commenter sous l'extrait de code impliqué. Cela permet de découper plus facilement la résolution des problèmes soulevés par la merge request via la fonctionnalité de "résolution de commentaire". Chaque segment peut être marqué comme résolu, mais seul le reviewer est autorisé à le faire !

## Merger une contribution dans `dev`

Lorsque vous avez fini votre développement et que votre "merge request" est prête comme décrit précédemment, exécutez `fmt` et `clippy` pour être sûr que votre code est propre puis exécutez tous les tests pour être sûr qu'ils passent :

    cargo fmt
    cargo clippy
    cargo test --all

Puis commitez le tout, sans le préfix wip- cette fois ci.

Ensuite nettoyez l'historique de votre branche avec un rebase interactif :

    git rebase -i dev

Renommez notamment les commits `wip:` et fusionnez les commits liés à fmt ou à clippy afin de simplifier l'historique.

Enfin faites un `push force` sur le dépôt distant :

    git push -f

Puis, rendez-vous sur le Gitlab et vérifiez que le code sur votre branche distante est bien celui censé s'y trouver.

Attendez 20 minutes que la chaîne d'intégration continue puisse vérifier votre code, et si elle réussit vous pouvez alors supprimer la mention WIP de votre Merge Request et tagger des développeurs expérimentés pour demander une revue de code.
