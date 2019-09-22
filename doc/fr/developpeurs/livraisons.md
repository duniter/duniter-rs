# Processus de livraison

Le processus de livraison des binaires officiels du projet Dunitrust suit des conventions très précises décrites ci-dessous.

Ce présent guide a deux objectifs :

1. Permettre aux contributeurs du projet de mieux comprendre les différentes étapes et les points de vigilance à avoir.
2. Permettre à un contributeur qui voudrait livrer Dunitrust (nécessite les droits `Maintainer`) de pouvoir le faire correctement.

## Prérequis

Avoir le rôle `Maintainer` ou `Owner` sur le projet gitlab.
Avoir l'autorisation explicite du chef de projet (ou à défaut du bureau d'Axiom-Team).

Installer le générateur de changelog :

    cargo install --force --git https://github.com/librelois/clog-cli

## Gel du code

Le moment du gel est toujours un choix difficile et important, à décider en concertation avec les principaux contributeurs.
Dans un monde idéal, le gel est effectué lorsque toutes les fonctionnalités d'un jalon ont été développées.
Mais si l'on souhaite livrer le jalon à la date annoncée, le gel doit avoir lieu *a minima* 3 semaines avant la date de livraison prévisionnelle du jalon.
Le choix de la date du gel est souvent un compromis entre la durée de retard acceptable et les fonctionnalités qui peuvent être reportées au jalon suivant.

Une fois la date du gel décidée, voici comment effectuer le gel du jalon `x.y`:

    ```bash
    git checkout dev
    git checkout -b release/x.y
    # mettre à jour la version de toutes les crates en -dev (remplacer par -alpha)
    clog --setversion vx.y.0-alpha
    git tag vx.y.0-alpha
    git checkout dev
    # mettre à jour la version de toutes les crates en x.y.0-dev (remplacer par le jalon suivant en gardant le suffixe -dev)
    ```

## Livraison de la version alpha

La version `alpha` du jalon est toujours la version du code au moment du gel, elle est réservée aux alpha-testeurs.
Une fois le gel effectué, il faut pusher la branche du jalon (`release/x.y`) et la tag puis vérifier que la CI/CD se passe bien.
En cas d'échec de la CI/CD, il faut corriger le problème dès que possible puis pusher le correctif sur la branche du jalon puis pusher un nouveau tag (`vx.y.0-alpha2`).
On incrémente ainsi le nombre après "alpha" jusqu'à obtenir une CI/CD qui réussit et donc obtenir des binaires livrables.

## Annonce de la version alpha

Le cahier de tests alpha doit être publié sur un ticket gitlab.

Il faut annoncer la version alpha sur la page du site web dédiée aux testeurs (site web en construction), puis notifier par tout moyen les alpha-testeurs connus.
L'annonce sur le site web doit comporter un lien direct vers le ticket du cahier des tests.

Les versions de type alpha ne fonctionnent que sur les monnaies de test connues (leur utilisation sur une monnaie de prod ou sur une monnaie inconnue ne fonctionnera pas).
Notez toutefois qu'il est possible de créer une nouvelle monnaie avec une version alpha (afin de tester cette fonctionnalité).

Tous les tickets remontés par les testeurs ne doivent pas forcément être corrigés immédiatement, seul les tickets bloquants doivent l'être.
Un ticket est considéré comme bloquant si :

- L'anomalie empêche de poursuivre le déroulement des tests et il n'existe pas de solution de contournement pour l'utilisateur avancé.
- L'anomalie provoque un arrêt inopiné (=crash) de Dunitrust dans au moins 1 cas d'utilisation normale et il n'existe pas de solution de contournement pour l'utilisateur avancé.

Une fois que tous les tests du cahier des tests ont été joués, et que tous les tickets bloquants ont été corrigés, on peut passer à la phase "bêta", même si tous les tests ne sont pas un succès.

Cette phase "alpha" peut être très rapide si les alpha-testeurs sont réactifs/efficaces/nombreux, il n'y a pas de délai minimal imposé, les deux seules contraintes sont :

1. Tous les tests du cahier des tests ont été joués intégralement au moins une fois.
2. Il n'existe plus de ticket bloquant connu.

## Livraison de la bêta

Lorsque les conditions permettant de clore la phase "alpha" sont atteintes, un `Maintainer` ou `Owner` du projet peut livrer la 1ère version bêta à condition d'avoir obtenu l'autorisation explicite du chef de projet (ou à défaut du bureau d'Axiom-Team).

Pour livrer la 1ère version bêta :

    ```bash
    git checkout release/x.y
    clog --setversion vx.y.0-beta
    git tag vx.y.0-beta
    git push --tags
    ```

Vérifier que la CI/CD se déroule avec succès et jusqu'au bout (ça peut être long).
Lorsque la CI/CD s'est intégralement terminée avec succès, l'annonce de la version bêta peut être faite sur le site web de Dunitrust ainsi que sur le forum technique.

En phase "bêta", tout les tests du cahier des tests doivent réussir. Lorsqu'un test échoue, un ticket doit être ouvert par le testeur.
Le test en échec doit être intégralement rejoué par un bêta-testeur après livraison du correctif.

Inutile de re-livrer une bêta à chaque ticket corrigé, autant attendre la correction d'une 1ère vague de tickets, il ne faut toutefois pas trop attendre, car les testeurs ont besoin des correctifs pour rejouer les tests en échec.

Les nouvelles versions bêta seront nommées `x.y-beta2`, `x.y-beta3`, etc.

Lorsque tous les cas de test du cahier des tests sont verts, on peut passer à la phase "release candidate".
Il est possible de passer à cette phase même si tous les tests ne sont pas verts, car il peut être décidé de reporter le correctif d'anomalies mineures à un jalon suivant, mais dans tous les cas il faut l'autorisation explicite du chef de projet (ou à défaut du bureau d'Axiom-Team).

## Livraison de la rc (release candidate)

Pour livrer la 1ère version rc :

    ```bash
    git checkout release/x.y
    clog --setversion vx.y.0-rc
    git tag vx.y.0-rc
    git push --tags
    ```
Vérifier que la CI/CD se déroule avec succès et jusqu'au bout (ça peut être long).
Lorsque la CI/CD s'est intégralement terminée avec succès, l'annonce de la version rc doit être faite sur le site web de Dunitrust ainsi que sur le forum technique.

Cette annonce doit invitée tout les utilisateurs avancés à se mettre à jour (la version rc n'est pas réservée aux testeurs).
La phase "rc" peut être clôturée dès lors qu'il se déroule 7 jours consécutifs sans découverte de nouvelles anomalies bloquantes ou majeures (les faux positifs ne comptent pas).

Attention: le fonctionnel du projet étant très complexe, beaucoup d'utilisateurs ont tendance à créer des tickets d'anomalies à tort. Ce qu'ils considèrent comme une anomalie est en fait conforme à l'attendu, leurs tickets doivent dans ce cas être considérés comme une demande de nouvelle fonctionnalité, qui sera peut-être traitée dans un futur jalon.

## Livraison de la stable

Pour livrer la 1ère version réputée stable du jalon, il faut que la dernière version rc est été publiée il y a plus de 7 jours et qu'aucune nouvelle anomalie bloquante ou majeure n'ait été détectée depuis.

Pour livrer la 1ère version stable :

    ```bash
    git checkout release/x.y
    clog --setversion vx.y.0
    git tag vx.y.0
    git push --tags
    ```

Vérifier que la CI/CD se déroule avec succès et jusqu'au bout (ça peut être long).

Lorsque la CI/CD s'est intégralement terminée avec succès, l'annonce de la version stable doit être faite sur le site web de Dunitrust ainsi que sur le forum technique.
L'annonce de cette version stable doit alors être relayée le plus largement possible, tout les utilisateurs de Dunitrust sont invités à se mettre a jour.

## Hotfix

Une fois la version stable livrée, seules les anomalies bloquantes seront corrigées, on réalise alors ce que l'on nomme un "hotfix", les correctifs sur une version stable sont réservés aux contributeurs les plus expérimentés du projet.

Le correctif doit impérativement être testé et approuvé par un alpha-testeur expérimenté.

Pour livrer le 1er hotfix :

    ```bash
    git checkout release/x.y
    clog --setversion vx.y.1
    git tag vx.y.1
    git push --tags
    ```

On incrémente alors le dernier nombre à chaque hotfix.

Les anomalies non-bloquantes ne doivent pas être corrigées sur une version stable, leur correction se fera dans le prochain jalon. Plus généralement, une version stable doit modifiée le moins possible.

Une anomalie est considérée comme bloquante dans 2 cas :

- L'Anomalie empêche l'utilisation d'une fonctionnalité essentielle de Dunitrust et il n'existe aucune solution de contournement.
- L'Anomalie constitue une faille de sécurité grave.
