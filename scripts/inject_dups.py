import random
import sys
from pathlib import Path

def inject_pcr_duplicates(r1_path, r2_path, rate, shuffle=False):
    def read_fastq(path):
        print(f"Lecture de {path}...")
        with open(path, 'r') as f:
            lines = f.readlines()
            return [lines[i:i+4] for i in range(0, len(lines), 4)]

    r1_reads = read_fastq(r1_path)
    r2_reads = read_fastq(r2_path)

    num_dups = int(len(r1_reads) * float(rate))
    print(f"--- Injection de {num_dups} duplicats PCR ({float(rate)*100}%) ---")

    indices = random.sample(range(len(r1_reads)), num_dups)

    final_r1 = r1_reads + [r1_reads[i] for i in indices]
    final_r2 = r2_reads + [r2_reads[i] for i in indices]

    if shuffle:
        print("--- Mélange aléatoire (shuffling) ---")
        combined = list(zip(final_r1, final_r2))
        random.shuffle(combined)
        final_r1, final_r2 = zip(*combined)

    # Écriture des fichiers finaux (en .fastq brut)
    for path, data in [(r1_path, final_r1), (r2_path, final_r2)]:
        output_name = Path(path).parent / f"final_{Path(path).name}"
        print(f"Écriture de {output_name}...")
        with open(output_name, 'w') as f:
            for read in data:
                f.writelines(read)

if __name__ == "__main__":
    # Vérification basique des arguments
    r1, r2, rate = sys.argv[1], sys.argv[2], sys.argv[3]
    do_shuffle = "--shuffle" in sys.argv
    inject_pcr_duplicates(r1, r2, rate, shuffle=do_shuffle)