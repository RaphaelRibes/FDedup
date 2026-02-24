import random
import sys

def create_random_genome(output_path, length):
    bases = ['A', 'C', 'G', 'T']
    sequence = ''.join(random.choices(bases, k=int(length)))
    
    with open(output_path, 'w') as f:
        f.write(">synthetic_genome\n")
        # On écrit par blocs de 80 caractères pour respecter le format FASTA
        for i in range(0, len(sequence), 80):
            f.write(sequence[i:i+80] + "\n")
    print(f"Génome synthétique de {length} bp créé dans {output_path}")

if __name__ == "__main__":
    create_random_genome(sys.argv[1], sys.argv[2])
