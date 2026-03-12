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
        
    data = lines[header_idx+1:]
    
    print(f"{'Ativo':<10} | {'Abertura':<20} | {'Lado':<5} | {'Qtd':<5}")
    print("-" * 50)
    
    all_rows = []
    for line in data:
        fields = line.split(';')
        if len(fields) >= 7:
            ativo = fields[0].strip()
            abertura = fields[1].strip()
            lado = fields[6].strip()
            qtd = fields[4].strip()
            all_rows.append((ativo, abertura, lado, qtd))
            print(f"{ativo:<10} | {abertura:<20} | {lado:<5} | {qtd:<5}")
            
    # Check for duplicates in (ativo, abertura, lado)
    import collections
    counts = collections.Counter([(r[0], r[1], r[2]) for r in all_rows])
    duplicates = [k for k, v in counts.items() if v > 1]
    
    if duplicates:
        print("\nDUPLICATES FOUND (SHOULD BE GROUPED):")
        for d in duplicates:
            print(f"  {d}")
    else:
        print("\nNO EXACT DUPLICATES FOUND IN (ATIVO, ABERTURA, LADO).")
        
except Exception as e:
    print(f"Error: {e}")
