import collections

file_path = r'C:\PROJETOS\TraderLogPro\2.csv'
try:
    with open(file_path, 'rb') as f:
        content = f.read()
    
    decoded = content.decode('windows-1252')
    lines = decoded.splitlines()
    
    header_idx = -1
    for i, line in enumerate(lines):
        if line.strip().lower().startswith('ativo;'):
            header_idx = i
            break
            
    if header_idx == -1:
        print("Header not found")
        sys.exit(0)
        
    data_lines = lines[header_idx+1:]
    groups = collections.defaultdict(list)
    
    for line in data_lines:
        fields = line.split(';')
        if len(fields) < 17:
            continue
        symbol = fields[0].strip().upper()
        entry_date = fields[1].strip()
        side = fields[6].strip()
        quantity = fields[4].strip()
        
        if not symbol or not entry_date:
            continue
            
        # This is exactly what the Rust code does for grouping
        key = (symbol, entry_date, side)
        groups[key].append(line)
        
    found_group = False
    for key, matches in groups.items():
        if len(matches) > 1:
            found_group = True
            print(f"Group found in 2.csv: {key} -> {len(matches)} lines")
            for m in matches:
                print(f"  {m}")
    
    if not found_group:
        print("No groups (partials) found in 2.csv using current logic.")
                
except Exception as e:
    print(f"Error: {e}")
