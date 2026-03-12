import sys

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
        sys.exit(1)
        
    header = lines[header_idx].split(';')
    data_lines = lines[header_idx+1:]
    
    print(" | ".join([f"{i}:{h}" for i, h in enumerate(header)]))
    print("-" * 100)
    
    for line in data_lines[:20]:
        fields = line.split(';')
        print(" | ".join([f"{i}:{f}" for i, f in enumerate(fields)]))
                
except Exception as e:
    print(f"Error: {e}")
