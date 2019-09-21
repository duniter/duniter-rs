# Processus de livraisons

Le processus de livraisons des binaires officiels du projet Dunitrust suis des conventions très précise decrite ci-dessous.

Ce présent guide a deux objectifs :

1. Permettre aux contributeurs du projet de mieux comprendre les différentes étapes et les points de vigilence a avoir.
2. Permettre a un contributeur qui voudrais livrer Dunitrust (nécessite les droits `Maintainer`) de pouvoir le faire correctement.

## Prérequis

Avoir le rôle `Maintainer` ou `Owner` sur le projet gitlab.
Avoir l'autorisation explicite du chef de projet (ou a défaut du bureau d'Axiom-Team).

Installer le générateur de changelog :

    cargo install --force --git https://github.com/librelois/clog-cli

## Gel du code

Le moment du gel est toujours un choix difficile et important, a décider en concertation avec les principaux contributeurs.
Dans un monde idéal, le gel est effectué lorsque toutes les fonctionnalités d'un jalon ont été développées.
Mais si l'on souhaite livrer le jalon a la date annoncée, le gel doit avoir lieu a minima 3 semaines avant la date de livraison prévisionnelle du jalon.
Le choix de la date du gel est souvent un compromis entre la durée de retard acceptable et les fonctionalités reportables a un jalon suivant.

Une fois la date du gel décicée, voici comment effectuer le gel du jalon `x.y`:

    ```bash
    git checkout dev
    git checkout -b release/x.y
    # mettre à jours la version de toute les crates en -dev (remplacer par -alpha)
    clog --setversion vx.y.0-alpha
    git tag vx.y.0-alpha
    git checkout dev
    # mettre à jours la version de toute les crates en x.y.0-dev (remplacer par le jalon suivant en gardantg le suffixe -dev)
    ```

## Livraison de la alpha

La version `alpha` du jalon est toujours la version du code au moment du gel, elle est réservée aux alpha-testeurs.
Une fois le gel effectué, il faut pusher la branche du jalon (`relaase/x.y`) et la tag puis vérifier que la CI/CD se passe bien.
En cas d'echec de la CI/CD, il faut corriger le problème dés que possible puis pusher le correctif sur la branche du jalon puis pusher un nouveau tag (vx.y.0-alpha2).
On incrémente ainsi le nombre après "alpha" jusqu'a obtenir une CI/CD qui réussie et donc obtenir des binaires livrables.

## Annonce de la alpha

Le cahier de tests alpha doit être publié sur un ticket gitlab.

Il faut annoncer la alpha sur la page du site web dédiée aux testeurs (site web en construction), puis notifier par tout moyen les alpha-testeurs connus.
L'annonce sur le site web doit comporter un lien direct vers le ticket du cahier des tests.

Les versions de type alpha ne fonctionne que sur les monnaies de test connues (leur utilisation sur une monnaie de prod ou sur une monnaie inconnue ne fonctionnera pas).
Notez toutefois qu'il est possible de créer une nouvelle monnaie avec une version alpha (afin de tester cette fonctionnalité).

Tout les tickets remontés par les testeurs ne doivent pas forcément être corrigés immédiatement, seul les tickets bloquants doivent l'être.
Un ticket est considéré comme bloquant si :

- L'anomalie empeche de poursuivre le déroulement des tests et il n'existe pas de solution de contournement pour l'utilisateur avancé.
- L'anomalie provoque un arret inopiné (=crash) de Dunitrust dans au moisn 1 cas d'utilisation normale et il n'existe pas de solution de contournement pour l'utilisateur avancé.

Une fois que tout les tests du cahier des tests ont été joués, et que tout les tickets bloquants ou été corrigés, ont peut passer a la phase "beta", même si tout les tests se sont pas un succès.

Cette phase "alpha" peut être très rapide si les alpha-testeurs sont réactifs/efficaces/nombreux, il n'y a pas de délai minimal imposé, les deux seules contraintes sont :

1. Tout les tests du cahier des tests ont été joués intégralement au moins une fois.
2. Il n'exixte plus de ticket bloquant connu.

## Livraison de la beta

Lorsque les conditions permettant de clore la phase "alpha" sont atteintes, un `Maintainer` ou `Owner` du projet peut liver la 1ère version beta a condition d'avoir obtenu l'autorisation explicite du chef de projet (ou à défaut du bureau d'axiom-Team).

Pour livrer la 1ère version beta :

    ```bash
    git checkout release/x.y
    clog --setversion vx.y.0-beta
    git tag vx.y.0-beta
    git push --tags
    ```

Vérifier que la CI/CD se déroule avec succès et jusqu'a bout (ça peut être long).
Lorsque la CI/CD s'est intégralement terminée avec succès, l'annonce de la version beta peut être faite sur le site web de Dunitrust ainsi que sur le forum technique.

En phase "beta", tout les tests du cahier des tests doivent réussir. Lorsqu'un test échoue, un ticket doit être ouvert par le testeur.
Le test en échec doit être intégralement réjoué par un beta-testeur après livraisons du correctif.

Inutile de relivrer une beta a chaque ticket corrigé, autant attendre la correction d'une 1ère vague de tickets, il ne faut toutefois pas trop attendre, car les testeurs ont besoin des correctifs pour rejouer les tests en échec.

Les nouvelles versions beta seront nommées `x.y-beta2`, `x.y-beta3`, etc.

Lorsque tout les cas de test du cahier des tests sont vert, on peut passer a la phase "release candidate".
Il est possible de passer a cette phase même si tout les tests ne sont pas vert, car il peut être décidé de reporter le correctif d'anomalies mineures à un jalon suivant, mais dans tout les cas il faut l'autorisation explicite du chef de projet (ou a défaut du bureau d'Axiom-Team).

## Livraison de la rc (release candidate)

Pour livrer la 1ère version rc :

    ```bash
    git checkout release/x.y
    clog --setversion vx.y.0-rc
    git tag vx.y.0-rc
    git push --tags
    ```
Vérifier que la CI/CD se déroule avec succès et jusqu'a bout (ça peut être long).
Lorsque la CI/CD s'est intégralement terminée avec succès, l'annonce de la version rc doit être faite sur le site web de Dunitrust ainsi que sur le forum technique.

Cette annonce doit invitée tout les utilisateurs avancés a se mettre à jours (la version rc n'est pas réservée aux testeurs).
La phase "rc" peut être cloturée dés lors qu'il se déroule 7 jours consécutifs sans découverte de nouvelles anomalies bloquantes ou majeures (les faux positifs ne comptent pas).

Attention: le fonctionnel du projet étant très complexe, beaucoup d'utilisateur ont tendance a créer des tickets d'anomalies a tord. Ce qu'ils considèrent comme une anomalie est en fait conforme a l'attendu, leurs tickets doivent dans ce cas être considéré comme une demande de nouvelle fonctionnalite, qui sera peut-être traitée dans un futur jalon.

## Livraison de la stable

Pour livrer la 1ère version réputée stable du jalon, il faut que la dernière version rc est été publiée il y a plus de 7 jours et qu'aucune nouvelle anomalie bloquante ou majeure n'est été détectée depuis.

Pour livrer la 1ère version stable :

    ```bash
    git checkout release/x.y
    clog --setversion vx.y.0
    git tag vx.y.0
    git push --tags
    ```

Vérifier que la CI/CD se déroule avec succès et jusqu'a bout (ça peut être long).

Lorsque la CI/CD s'est intégralement terminée avec succès, l'annonce de la version stable doit être faite sur le site web de Dunitrust ainsi que sur le forum technique.
L'annonce de cette version stable doit alors être relayée le plus largement possible, tout les utilisateurs de dunitrust sont invités a se mettre a jours.

## Hotfix

Une fois la version stable livrée, seules les anomalies bloquantes seront corrigées, on réalise a lors ce que l'on nomme un "hotfix", les correctifs sur une version stable sont réservés aux contributeurs les plus expérimentés du projet.

Le correctif doit impérativement être testé et approuvé par un alpha-testeur expérimenté.

Pour livrer le 1er hotfix :

    ```bash
    git checkout release/x.y
    clog --setversion vx.y.1
    git tag vx.y.1
    git push --tags
    ```

On incrémente alors le dernier nombre à chaque hotfix.

Les anomalies non-bloquantes ne doivent pas être corrigés sur une version stable, leur correction se fera dans le prochain jalon. Plus généralement, une version stable doit modifiée le moins possible.

Une anomalie est considérée comme bloquante dans 2 cas :

- L'Anomalie empeche l'utilisation d'une fonctionnalité essentielle de Dunitrust et il n'existe aucune solution de contournement.
- L'Anomalie constitue une faille de sécurité grave.
