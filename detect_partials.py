import collections

file_path = r'C:\PROJETOS\TraderLogPro\2.csv'
try:
    with open(file_path, 'rb') as f:
        content = f.read()
    
    decoded = content.decode('windows-1252')
    lines = decoded.splitlines()
    
    # Find header
    header_idx = -1
    for i, line in enumerate(lines):
        if line.strip().lower().startswith('ativo;'):
            header_idx = i
            break
            
    if header_idx == -1:
        print("Header not found")
        sys.exit(1)
        
    data_lines = lines[header_idx+1:]
    groups = collections.defaultdict(list)
    
    for line in data_lines:
        fields = line.split(';')
        if len(fields) < 2:
            continue
        ativo = fields[0].strip()
        abertura = fields[1].strip()
        if not ativo or not abertura:
            continue
        groups[(ativo, abertura)].append(line)
        
    for key, matching_lines in groups.items():
        if len(matching_lines) > 1:
            print(f"Group found: {key}")
            for ml in matching_lines:
                print(f"  {ml}")
                
except Exception as e:
    print(f"Error: {e}")
