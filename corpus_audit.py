
import os, glob, re, json
from collections import defaultdict, Counter

base_dir = r'C:\Users\Sonali\Desktop\projectwork\business_Dev\LiteCowork\docs'
active_files = {}

for root, dirs, files in os.walk(base_dir):
    if 'archive' in root.split(os.sep):
        continue
    for f in files:
        path = os.path.join(root, f)
        rel = os.path.relpath(path, base_dir).replace(chr(92), '/')
        with open(path, 'r', encoding='utf-8', errors='ignore') as fp:
            active_files[rel] = fp.readlines()

print('Total files read:', len(active_files))
total_lines = sum(len(lines) for lines in active_files.values())
print('Total lines read:', total_lines)

# 1. ADR audit
adrs = []
for k in sorted(active_files.keys()):
    if k.startswith('adr/') and k.endswith('.md') and k != 'adr/README.md':
        lines = active_files[k]
        title = lines[0].strip('# ').strip() if lines else k
        status = 'UNKNOWN'
        for l in lines[:25]:
            if 'Status' in l:
                status = l.strip().replace('*', '').strip()
                break
        adrs.append((k, title, status, len(lines)))

# 2. Schemas & Migrations audit
schemas = []
for k in sorted(active_files.keys()):
    if k.startswith('schemas/'):
        schemas.append((k, len(active_files[k])))

# 3. Contract & Ledger audit
ledgers = [k for k in active_files if 'LEDGER' in k or 'CONTRACT' in k or 'MATRIX' in k]

# 4. Cross reference graph
ref_counts = Counter()
for src, lines in active_files.items():
    text = ''.join(lines)
    for target in active_files.keys():
        tname = target.split('/')[-1]
        if tname != src.split('/')[-1] and tname in text:
            ref_counts[target] += 1

# 5. Contradiction & Alignment points
contradictions = []
if 'CONTRADICTION-AND-ALIGNMENT-REGISTER.md' in active_files:
    c_lines = active_files['CONTRADICTION-AND-ALIGNMENT-REGISTER.md']
    for l in c_lines:
        if l.startswith('## ') or l.startswith('### '):
            contradictions.append(l.strip())

print(json.dumps({
    'file_count': len(active_files),
    'line_count': total_lines,
    'adr_count': len(adrs),
    'adrs': adrs,
    'schema_count': len(schemas),
    'schemas': schemas,
    'ledger_count': len(ledgers),
    'top_referenced': ref_counts.most_common(15),
    'contradiction_sections': contradictions[:10]
}, indent=2))
