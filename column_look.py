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
    data = lines[header_idx+1:]
    
    for line in data:
        if 'WINJ26' in line:
            fields = line.split(';')
            for i, (h, f) in enumerate(zip(header, fields)):
                print(f"{i:2}: {h:<20} | {f}")
            break
                
except Exception as e:
    print(f"Error: {e}")
