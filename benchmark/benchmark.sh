#!/bin/bash
#SBATCH --job-name=bench_fdedup
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=6
#SBATCH --mem=32G
#SBATCH --time=48:00:00
#SBATCH --partition=cpu-dedicated
#SBATCH --account=dedicated-cpu@cirad-normal
set -e

# Configuration
BIN_DIR="./bin"
FDEDUP="$BIN_DIR/fdedup"
DATA_DIR="./data"
RESULTS_CSV="benchmark_results.csv"
GENOME_FA="$DATA_DIR/hg38.fa"

# Estimation : 150bp SE FASTQ ~ 320 lectures par Mo (environ)
# Donc pour 1 Go non compressé, il faut environ 3.2 Millions de lectures.
READS_PER_GB=3200000 

mkdir -p "$DATA_DIR"

if [ ! -x "$FDEDUP" ]; then
    echo "Erreur : L'exécutable $FDEDUP n'est pas présent ou n'a pas les droits d'exécution."
    exit 1
fi

# 1. Téléchargement et préparation du génome de référence (si nécessaire)
if [ ! -f "$GENOME_FA" ]; then
    echo "Téléchargement du génome humain (GRCh38)..."
    wget -qO- "https://ftp.ensembl.org/pub/release-111/fasta/homo_sapiens/dna/Homo_sapiens.GRCh38.dna.primary_assembly.fa.gz" | gunzip > "$GENOME_FA"
fi

# Initialisation du fichier de résultats
echo "Taille_Cible_Go,Taille_Reelle_Go,Temp_Ecoule,RAM_Max_Mo" > "$RESULTS_CSV"

# 2. Boucle de benchmark (de 5 à 100 Go)
for size_gb in {5,10,15,20,25,30,40,50,60,70,80,90,100}; do
    
    echo "=================================================="
    echo "Palier ${size_gb}Go : Génération et ajout de données..."
    
    # Calcul du nombre de lectures nécessaire pour atteindre la taille cible (non compressée)
    # Note: Cette estimation dépend de la longueur des headers. 
    # On ajuste le nombre de lectures (-N) pour approcher la taille demandée.
    TARGET_READS=$((size_gb * READS_PER_GB))
    
    echo "Nombre de lectures estimé : ${TARGET_READS}"
    
    # Génération d'un nouveau fichier avec wgsim (seed = $i pour la variabilité)
    # On génère du single-end ici (on ignore le read2) pour simplifier l'input de fdedup
    # Le fichier est généré en .fq (non compressé)
    
    OUT_TMP="$DATA_DIR/tmp_${size_gb}GB.fq"
    wgsim -N $TARGET_READS -1 150 -2 150 -S ${size_gb} "$GENOME_FA" "$OUT_TMP" /dev/null > /dev/null
    
    # Mesure de la taille réelle du fichier généré (non compressé)
    ACTUAL_SIZE_BYTES=$(wc -c < "$OUT_TMP")
    ACTUAL_SIZE_GB=$(echo "scale=2; $ACTUAL_SIZE_BYTES / 1073741824" | bc)
    
    echo "Taille réelle générée : ${ACTUAL_SIZE_GB} Go (Cible: ${size_gb} Go)"
    
    if [ "$ACTUAL_SIZE_GB" -lt "$((size_gb * 1))" ]; then
        echo "Avertissement : Le fichier est beaucoup plus petit que la cible. Vérifiez l'espace disque."
    fi
    
    echo "Lancement de fdedup..."
    
    # 3. Exécution et profilage sur le fichier non compressé (.fq)
    OUT_FILE="$DATA_DIR/out_${size_gb}GB.fastq.gz"
    LOG_FILE="$DATA_DIR/time_${size_gb}.log"
    
    /usr/bin/env time -v "$FDEDUP" --forcer "$OUT_TMP" "$OUT_FILE" 2> "$LOG_FILE"
    
    # 4. Extraction des métriques
    # Le temps au format (h:mm:ss ou m:ss)
    WALL_TIME=$(grep "Elapsed (wall clock) time" "$LOG_FILE" | awk '{print $NF}')
    
    # Max RSS en Kilo-octets, converti en Méga-octets
    MAX_RAM_KB=$(grep "Maximum resident set size" "$LOG_FILE" | awk '{print $NF}')
    MAX_RAM_MB=$(echo "scale=2; $MAX_RAM_KB / 1024" | bc)
    
    echo "-> Temps : $WALL_TIME | RAM Max : $MAX_RAM_MB Mo"
    
    # Sauvegarde dans le CSV (Taille cible, Taille réelle, Temps, RAM)
    echo "${size_gb},${ACTUAL_SIZE_GB},${WALL_TIME},${MAX_RAM_MB}" >> "$RESULTS_CSV"
    
    # Nettoyage des fichiers lourds de l'itération pour préserver le disque
    rm -f "$OUT_TMP" "$DATA_DIR/tmp_${size_gb}GB.fq"
done

echo "=================================================="
echo "Benchmark terminé ! Résultats sauvegardés dans $RESULTS_CSV"
