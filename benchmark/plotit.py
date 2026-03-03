import pandas as pd
import matplotlib.pyplot as plt

def parse_time_to_minutes(time_str):
    """Converts MM:SS.ms to total minutes for plotting."""
    if pd.isna(time_str):
        return 0
    parts = str(time_str).split(':')
    if len(parts) == 2:
        mins = float(parts[0])
        secs = float(parts[1])
        return mins + (secs / 60)
    return 0

# 1. Load Data
df = pd.read_csv('benchmark_results.csv')

# 2. Process Data
df['Temp_Ecoule_Min'] = df['Temp_Ecoule'].apply(parse_time_to_minutes)

# 3. Create Subplots (Sharing the X-axis for Target Size)
fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(10, 8), sharex=True)

# Top Plot: Elapsed Time vs. Target Size
ax1.plot(df['Taille_Cible_Go'], df['Temp_Ecoule_Min'], marker='o', color='#1f77b4', linewidth=2)
ax1.set_ylabel('Elapsed Time (Minutes)', fontsize=11)
ax1.set_title('Benchmark Performance vs FASTX input file size', fontsize=14, pad=15)
ax1.grid(True, linestyle='--', alpha=0.7)

# Bottom Plot: Max RAM vs. Target Size
ax2.plot(df['Taille_Cible_Go'], df['RAM_Max_Mo'], marker='s', color='#ff7f0e', linewidth=2)
ax2.set_xlabel('Input Size (GB)', fontsize=11)
ax2.set_ylabel('Max RAM (MB)', fontsize=11)
ax2.grid(True, linestyle='--', alpha=0.7)

# 4. Final Formatting
plt.tight_layout()
plt.savefig('benchmark_graphs.svg')
print("Plot successfully generated and saved as 'benchmark_graphs.svg'")
