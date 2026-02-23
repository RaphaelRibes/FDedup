# Architecture du Déduplicateur de PCR par Hachage


## 1. Le Concept (Le Problème et la Solution)

Lors du séquençage, l'amplification PCR génère des lectures (reads) artificiellement dupliquées. L'approche classique d'alignement complet pour les identifier est extrêmement coûteuse en temps et en calcul ou nécessite un tri lourd.

L'approche proposée ici transforme ce problème d'alignement génomique en un problème de **recherche d'entiers en mémoire**. En hachant chaque read, on le réduit à une simple empreinte numérique. Stocker et interroger ces empreintes dans une structure de données succincte permet de traiter le flux en avec une empreinte mémoire minimale et statique.

---

## 2. Flux d'Exécution Principal

Le cœur du programme repose sur une boucle de traitement continue et ultra-rapide.

1. **Lecture :** Extraction d'un read depuis le fichier source (FASTQ/FASTA).
2. **Hachage :** Passage de la séquence nucléotidique dans l'algorithme **xxHash (XXH3)**.
3. **Interrogation et Routage :**
    * Vérification de la présence du hash dans la structure succincte en mémoire.
    * **Si absent (Nouveau read) :** Bascule du bit/ajout dans la structure pour marquer sa présence.
        * Écriture immédiate du read complet dans le fichier de sortie (`output.fastq`).
    * **Si présent (Duplicata) :** Le read est purement et simplement ignoré. On passe au suivant.



---

## 3. Tolérance aux Pannes et Reprise (Crash Recovery)

L'élégance de ce système réside dans le fait que le fichier de sortie sert lui-même de point de sauvegarde (checkpoint). Il n'est pas nécessaire de dumper l'état de la mémoire sur le disque.

**Procédure de reprise en cas d'interruption :**
1. **Initialisation de secours :** Ouverture du fichier `output.fastq` partiel existant.
2. **Reconstruction de l'état :** Lecture séquentielle rapide de ce fichier, hachage de chaque read, et repopulation silencieuse de la structure de données en mémoire.
3. **Reprise du flux :** Ouverture du fichier source original et reprise du flux d'exécution normal (Étape 1 du flux principal).

---

## 4. Gestion des Entrées/Sorties : Modes de Lecture

Les performances I/O (Entrées/Sorties) sont souvent le goulot d'étranglement de ce type d'outil bioinformatique. Deux stratégies sont exposées via les arguments de ligne de commande :

| Mode                      | Description                                                                        | Cas d'usage idéal                                 | Avantages                                                                                            |
|:--------------------------|:-----------------------------------------------------------------------------------|:--------------------------------------------------|:-----------------------------------------------------------------------------------------------------|
| **Streaming (Défaut)**    | Lecture et traitement par paquets (chunks) ou ligne par ligne.                     | Fichiers massifs (plusieurs dizaines de Go).      | Empreinte mémoire I/O quasi nulle ; pas de limite de taille de fichier.                              |
| **In-Memory (Optionnel)** | Chargement de l'intégralité du fichier source dans un buffer RAM avant traitement. | Petits fichiers (ex: séquençages ciblés, panels). | Supprime les changements de contexte système (context switches) ; vitesse maximale d'exécution pure. |

---

## 5. Environnement et Dépendances

Pour garantir des performances optimales et une reproductibilité totale sans s'encombrer de conflits de versions, l'outil s'appuiera sur un environnement isolé embarquant directement les compilateurs et les bibliothèques nécessaires (comme la lib `xxhash` officielle ou les parseurs FASTQ optimisés). L'utilisation d'un gestionnaire natif garantira de toujours compiler avec les optimisations les plus récentes (ex: AVX512 pour XXH3).

# Algo

```text
Algorithme FDedup(chemin_vers_fichier):
    initialiser_structure_succincte()
    ouvrir_fichier_source(chemin_vers_fichier)
    ouvrir_fichier_sortie("output.fastq", mode="append")

    pour chaque read dans fichier_source:
        hash = XXH3(read.sequence)
        
        si hash non présent dans structure_succincte:
            ajouter hash à structure_succincte
            écrire read dans fichier_sortie
        sinon:
            ignorer read

    fermer_fichier_source()
    fermer_fichier_sortie()
```